use std::rc::Rc;
use std::mem;
use std::collections::HashMap;

use num_traits::ToBytes;
use rusty_vm_lib::{assembly::ByteCode, byte_code::ByteCodes, registers::{Registers, GENERAL_PURPOSE_REGISTER_COUNT, REGISTER_SIZE}, vm::{Address, ADDRESS_SIZE}};

use crate::{flow_analyzer::FunctionGraph, irc::{IROperator, IRValue, Label, TnID}, lang::data_types::{DataType, LiteralValue, BOOL_SIZE, CHAR_SIZE, F32_SIZE, F64_SIZE, I16_SIZE, I32_SIZE, I64_SIZE, I8_SIZE, ISIZE_SIZE, U16_SIZE, U32_SIZE, U64_SIZE, U8_SIZE, USIZE_SIZE}, symbol_table::SymbolTable};

use super::{LabelAddressMap, LabelGenerator, StaticAddressMap};


const GENERAL_PURPOSE_REGISTER_COUNT_WITHOUT_R1R2: usize = GENERAL_PURPOSE_REGISTER_COUNT - 2;


struct UsedGeneralPurposeRegisterTable {
    registers: [bool; GENERAL_PURPOSE_REGISTER_COUNT_WITHOUT_R1R2]
}

impl UsedGeneralPurposeRegisterTable {

    pub const fn new() -> Self {
        Self {
            registers: [false; GENERAL_PURPOSE_REGISTER_COUNT_WITHOUT_R1R2]
        }
    }


    pub const fn is_in_use(&self, reg: Registers) -> bool {
        self.registers[reg as usize - 2]
    }


    pub fn set_in_use(&mut self, reg: Registers) {
        self.registers[reg as usize - 2] = true;
    }


    pub fn set_unused(&mut self, reg: Registers) {
        self.registers[reg as usize - 2] = false;
    }


    pub fn get_first_unused_register_not_r1r2(&self) -> Option<Registers> {
        self.registers.iter()
            .enumerate()
            .find(|(_, &is_used)| !is_used)
            .map(|(i, _)| Registers::from(i as u8 + 2))
    }

}


/// Signed offset from a memory location on the stack.
/// TODO: brobably this doesn't need to be an isize. A i32 would be good enough because stack frames aren't usually huge
type StackOffset = isize;

#[derive(Clone)]
enum TnLocation {

    /// The value of the Tn is stored inside a register
    Register (Registers),
    /// The value of the Tn is stores on the stack at an offset
    Stack (StackOffset)

}


fn generate_stack_value_at_offset(stack_frame_offset: StackOffset, value: &LiteralValue, bc: &mut ByteCode, static_address_map: &StaticAddressMap, unnamed_local_statics: &mut UnnamedLocalStaticsManager, labels_to_resolve: &mut Vec<Address>) {

    fn internal(stack_frame_offset: StackOffset, value: &LiteralValue, bc: &mut ByteCode, static_address_map: &StaticAddressMap, unnamed_local_statics: &mut UnnamedLocalStaticsManager, labels_to_resolve: &mut Vec<Address>, is_generating_array: bool) {

        if !is_generating_array {
            // Calculate the target absolute address
            bc.calculate_address_from_stack_frame_offset(stack_frame_offset, false);
        }

        match value {

            LiteralValue::Char(ch) => {
                // mov1 [r1] ch
                bc.move_into_addr_in_reg_from_const(CHAR_SIZE as u8, Registers::R1, *ch as u8)
            },

            LiteralValue::Bool(b) => {
                // mov1 [r1] b
                bc.move_into_addr_in_reg_from_const(BOOL_SIZE as u8, Registers::R1, *b as u8);
            },

            LiteralValue::Numeric(n) => {
                // mov(sizeof(n)) [r1] n
                bc.move_into_addr_in_reg_from_const(n.data_type().static_size().unwrap() as u8, Registers::R1, n);
            },

            LiteralValue::Array { element_type, items } => {
                /*
                    Sequentially initialize each array element in-place.
                    Keep the current target address in r1 and increment it each time a new element is initialized.
                    If the element is an array, the generation process of the item array value will increment the target address like described above.
                */
                let element_size = element_type.static_size().unwrap();
                let is_element_array = matches!(element_type.as_ref(), DataType::Array { .. });

                let second_last_index = items.len().checked_sub(2).unwrap_or(0);
                for item in items[0..second_last_index].iter() {

                    internal(0, item, bc, static_address_map, unnamed_local_statics, labels_to_resolve, true);

                    // If the element is an array, there's no need to update the target address because the inner array's generation process already did that
                    // On the other hand, if the element is an array, we need to increment the target address to pass to the next item position in the array
                    if !is_element_array {
                        // r1 is the target address
                        // Increment the target address by the size of the newly initialized element to pass to the next element position
                        // mov8 r2 sizeof(item)
                        bc.move_into_reg_from_const(ADDRESS_SIZE as u8, Registers::R2, element_size);
                        bc.add_opcode(ByteCodes::INTEGER_ADD);
                    }
                }

                if let Some(last_item) = items.last() {
                    internal(0, last_item, bc, static_address_map, unnamed_local_statics, labels_to_resolve, is_generating_array);
                    // Usually, there's no need to increment the target address if the element is the last of the array being generated
                    // However, if this array being generated is an element of an outer array and this element is not an array itself, the target address must be incremented to pass to the next item position in the outer array
                    if is_generating_array && !is_element_array {
                        // r1 is the target address
                        // Increment the target address by the size of the newly initialized element to pass to the next element position in the outer array
                        // mov8 r2 sizeof(item)
                        bc.move_into_reg_from_const(ADDRESS_SIZE as u8, Registers::R2, element_size);
                        bc.add_opcode(ByteCodes::INTEGER_ADD);
                    }
                }
            },

            LiteralValue::Ref { target, .. } => {
                bc.move_unnamed_static_ref_into_addr_in_reg(Registers::R1, Rc::clone(target), labels_to_resolve, unnamed_local_statics);
            },

            LiteralValue::StaticString(strind_id) => {
                // mov8 [r1] address of static string
                let static_addr = *static_address_map.get(strind_id).unwrap();
                bc.move_into_addr_in_reg_from_const(ADDRESS_SIZE as u8, Registers::R1, static_addr);
            },
        }
    }

    internal(stack_frame_offset, value, bc, static_address_map, unnamed_local_statics, labels_to_resolve, false);
}


/// Generate the code to construct the given value in-place on the stack.
/// Return the amount of bytes the stack pointer was pushed to perform this operation.
/// The generated value will be placed at the top of the stack.
/// Based on the data type, a different approach is used to include the value into the bytecode.
/// Some small values can be just be pushed with a PUSH_FROM_CONST instruction, while others need to be constructed on the fly.
fn generate_push_stack_value(value: &LiteralValue, bc: &mut ByteCode, static_address_map: &StaticAddressMap, stack_frame_offset: &mut StackOffset, unnamed_local_statics: &mut UnnamedLocalStaticsManager, labels_to_resolve: &mut Vec<Address>) {
    let offset = match value {

        LiteralValue::Bool(b) => {
            // push1 b
            bc.add_opcode(ByteCodes::PUSH_FROM_CONST);
            bc.add_byte(BOOL_SIZE as u8);
            bc.add_byte(*b as u8);
            BOOL_SIZE
        },

        LiteralValue::Char(ch) => {
            // push1 ch
            bc.add_opcode(ByteCodes::PUSH_FROM_CONST);
            bc.add_byte(CHAR_SIZE as u8);
            bc.add_byte(*ch as u8);
            CHAR_SIZE
        },

        LiteralValue::StaticString(string_id) => {
            // push8 address of static string
            bc.add_opcode(ByteCodes::PUSH_FROM_CONST);
            bc.add_byte(ADDRESS_SIZE as u8);
            let static_addr = *static_address_map.get(string_id).unwrap();
            bc.add_const_usize(static_addr);
            ADDRESS_SIZE
        },

        LiteralValue::Array { items, .. } => {
            /*
                Sequentially push every element of the array onto the stack
                Push the elements in reverse order to maintain the correct indices
                This could also be done like gcc does, by moving each element onto the stack at the correct index.
                However, calculating the element index can be more expensive because it requires loading the operands into r1 and r2.
                Also, values larger than 8 bytes cannot be directly moved, so this approach is more generic
            */
            for item in items.iter().rev() {
                generate_push_stack_value(item, bc, static_address_map, stack_frame_offset, unnamed_local_statics, labels_to_resolve);
            }

            // Don't modify the stack frame offset because it was already modified when pushing the array elements
            0
        },

        LiteralValue::Numeric(n) => {
            // push(sizeof(n)) n
            bc.add_opcode(ByteCodes::PUSH_FROM_CONST);
            let number_size = n.data_type().static_size().unwrap();
            bc.add_byte(number_size as u8);
            bc.extend(n.to_le_bytes());
            number_size
        },

        LiteralValue::Ref { target, .. } => {
            bc.push_unnamed_static_ref(Rc::clone(target), labels_to_resolve, unnamed_local_statics);
            ADDRESS_SIZE
        },
    } as StackOffset;

    *stack_frame_offset -= offset;
}


trait ByteCodeOutput {

    fn add_byte(&mut self, byte: u8);

    fn add_reg(&mut self, reg: Registers);

    fn add_opcode(&mut self, opcode: ByteCodes);

    fn add_placeholder_label(&mut self, label: Label, labels_to_resolve: &mut Vec<Address>);

    fn add_const_usize(&mut self, value: usize);

    /// Generate the bytecode to push from the given register.
    /// Update the stack frame offset to account for the pushed value
    fn push_from_reg(&mut self, reg: Registers, stack_frame_offset: &mut StackOffset);

    fn pop8_into_reg(&mut self, reg: Registers, stack_frame_offset: &mut StackOffset);

    fn move_into_reg_from_reg(&mut self, dest: Registers, source: Registers);

    fn move_into_reg_from_const<T>(&mut self, handled_size: u8, reg: Registers, value: T) where T: ToBytes;

    fn move_into_reg_from_addr_in_reg(&mut self, handled_size: u8, dest: Registers, source: Registers);

    fn push_stack_pointer_const(&mut self, offset: usize, stack_frame_offset: &mut StackOffset);

    fn load_first_numeric_arg(&mut self, tn_location: &TnLocation, size: usize);

    fn load_second_numeric_arg(&mut self, tn_location: &TnLocation, size: usize, reg_table: &mut UsedGeneralPurposeRegisterTable);

    fn push_unnamed_static_ref(&mut self, static_value: Rc<LiteralValue>, labels_to_resolve: &mut Vec<Address>, unnamed_local_statics: &mut UnnamedLocalStaticsManager);

    fn load_first_arg(&mut self, arg: &IRValue, tn_locations: &mut HashMap<TnID, TnLocation>, labels_to_resolve: &mut Vec<Address>, unnamed_local_statics: &mut UnnamedLocalStaticsManager, static_address_map: &StaticAddressMap);

    fn load_const(&mut self, target_reg: Registers, value: &LiteralValue, unnamed_local_statics: &mut UnnamedLocalStaticsManager, labels_to_resolve: &mut Vec<Address>, static_address_map: &StaticAddressMap);

    fn load_second_arg(&mut self, arg: &IRValue, tn_locations: &mut HashMap<TnID, TnLocation>, labels_to_resolve: &mut Vec<Address>, unnamed_local_statics: &mut UnnamedLocalStaticsManager, static_address_map: &StaticAddressMap, reg_table: &mut UsedGeneralPurposeRegisterTable);

    fn store_r1(&mut self, target_tn: TnID, reg_table: &mut UsedGeneralPurposeRegisterTable, tn_locations: &mut HashMap<TnID, TnLocation>);

    fn calculate_address_from_stack_frame_offset(&mut self, offset: StackOffset, r2_already_sbp: bool);

    fn move_into_addr_in_reg_from_reg(&mut self, handled_size: u8, dest: Registers, source: Registers);

    fn move_into_addr_in_reg_from_const<T>(&mut self, handled_size: u8, dest: Registers, value: T) where T: ToBytes;

    fn move_unnamed_static_ref_into_addr_in_reg(&mut self, dest: Registers, static_value: Rc<LiteralValue>, labels_to_resolve: &mut Vec<Address>, unnamed_local_statics: &mut UnnamedLocalStaticsManager);

    fn pop_stack_pointer_const(&mut self, offset: usize, stack_frame_offset: &mut StackOffset);

}

impl ByteCodeOutput for ByteCode {

    fn pop_stack_pointer_const(&mut self, offset: usize, stack_frame_offset: &mut StackOffset) {
        self.add_opcode(ByteCodes::POP_STACK_POINTER_CONST);
        self.add_byte(mem::size_of::<usize>() as u8);
        self.add_const_usize(offset);
        *stack_frame_offset += offset as StackOffset;
    }


    fn move_into_addr_in_reg_from_const<T>(&mut self, handled_size: u8, dest: Registers, value: T)
    where
        T: ToBytes
    {
        debug_assert_eq!(
            AsRef::<[u8]>::as_ref(&value.to_le_bytes()).len(),
            handled_size as usize
        );

        self.add_opcode(ByteCodes::MOVE_INTO_ADDR_IN_REG_FROM_CONST);
        self.add_byte(handled_size);
        self.add_reg(dest);
        self.extend_from_slice(AsRef::<[u8]>::as_ref(&value.to_le_bytes()));
    }

    fn move_into_addr_in_reg_from_reg(&mut self, handled_size: u8, dest: Registers, source: Registers) {
        self.add_opcode(ByteCodes::MOVE_INTO_ADDR_IN_REG_FROM_REG);
        self.add_byte(handled_size);
        self.add_reg(dest);
        self.add_reg(source);
    }


    /// Generate the bytecode for calculating the absolute memory address, given a signed offset from the stack frame base pointer.
    /// The resulting absolute memory address will be stored in r1
    /// The stack frame base pointer will be stored in r2 as a side effect, which can be used to calculate multiple absolute addresses in a row.
    fn calculate_address_from_stack_frame_offset(&mut self, offset: StackOffset, r2_already_sbp: bool) {

        if !r2_already_sbp {
            // mov r2 sbp
            self.move_into_reg_from_reg(Registers::R2, Registers::STACK_FRAME_BASE_POINTER);
        }

        // Use the absolute value of the offset because addresses are unsigned integers.
        // Subtracting the usize representation of an isize may result in an overflow.
        if offset < 0 {
            // mov8 r1 abs(offset)
            // isub
            self.move_into_reg_from_const(ADDRESS_SIZE as u8, Registers::R1, offset.unsigned_abs());
            self.add_opcode(ByteCodes::INTEGER_SUB);
        } else {
            // mov8 r1 offset
            // iadd
            self.move_into_reg_from_const(ADDRESS_SIZE as u8, Registers::R1, offset);
        }
        // if offset is 0, don't perform the operation
    }


    fn add_byte(&mut self, byte: u8) {
        self.push(byte);
    }


    fn add_reg(&mut self, reg: Registers) {
        self.add_byte(reg as u8);
    }


    fn add_opcode(&mut self, opcode: ByteCodes) {
        self.add_byte(opcode as u8);
    }


    fn add_placeholder_label(&mut self, label: Label, labels_to_resolve: &mut Vec<Address>) {
        labels_to_resolve.push(self.len());
        self.extend(label.to_le_bytes());
    }


    fn add_const_usize(&mut self, value: usize) {
        self.extend(value.to_le_bytes());
    }


    fn push_from_reg(&mut self, reg: Registers, stack_frame_offset: &mut StackOffset) {
        self.add_opcode(ByteCodes::PUSH_FROM_REG);
        self.add_reg(reg);
        *stack_frame_offset -= REGISTER_SIZE as StackOffset;
    }


    fn pop8_into_reg(&mut self, reg: Registers, stack_frame_offset: &mut StackOffset) {
        self.add_opcode(ByteCodes::POP_INTO_REG);
        self.add_byte(REGISTER_SIZE as u8);
        self.add_reg(reg);
        *stack_frame_offset += REGISTER_SIZE as StackOffset
    }


    fn move_into_reg_from_reg(&mut self, dest: Registers, source: Registers) {
        self.add_opcode(ByteCodes::MOVE_INTO_REG_FROM_REG);
        self.add_reg(dest);
        self.add_reg(source);
    }


    fn move_into_reg_from_const<T>(&mut self, handled_size: u8, reg: Registers, value: T)
    where
        T: ToBytes
    {
        debug_assert_eq!(
            AsRef::<[u8]>::as_ref(&value.to_le_bytes()).len(),
            handled_size as usize
        );

        self.add_opcode(ByteCodes::MOVE_INTO_REG_FROM_CONST);
        self.add_byte(handled_size);
        self.add_reg(reg);
        self.extend_from_slice(AsRef::<[u8]>::as_ref(&value.to_le_bytes()));
    }


    fn move_into_reg_from_addr_in_reg(&mut self, handled_size: u8, dest: Registers, source: Registers) {
        self.add_opcode(ByteCodes::MOVE_INTO_REG_FROM_ADDR_IN_REG);
        self.add_byte(handled_size);
        self.add_reg(dest);
        self.add_reg(source);
    }


    fn push_stack_pointer_const(&mut self, offset: usize, stack_frame_offset: &mut StackOffset) {
        self.add_opcode(ByteCodes::PUSH_STACK_POINTER_CONST);
        self.add_byte(mem::size_of::<usize>() as u8);
        self.add_const_usize(offset);
        *stack_frame_offset -= offset as StackOffset;
    }


    fn load_first_numeric_arg(&mut self, tn_location: &TnLocation, size: usize) {
        match tn_location {

            TnLocation::Register(reg) => {
                // mov r1 reg
                self.move_into_reg_from_reg(Registers::R1, *reg);
            },

            TnLocation::Stack(offset) => {
                // Calculate the stack address of the operand
                self.calculate_address_from_stack_frame_offset(*offset, false);

                // Load the operand value
                // mov(n) r1 [r1]
                self.move_into_reg_from_addr_in_reg(size as u8, Registers::R1, Registers::R1);
            },
        }
    }


    fn load_second_numeric_arg(&mut self, tn_location: &TnLocation, size: usize, reg_table: &mut UsedGeneralPurposeRegisterTable) {
        match tn_location {
            TnLocation::Register(reg) => {
                // mov r2 reg
                self.move_into_reg_from_reg(Registers::R2, *reg);
            },
            TnLocation::Stack(offset) => {

                // Save the value stored in r1, which would otherwise be overwritten
                let r1_store =
                    if let Some(store_reg) = reg_table.get_first_unused_register_not_r1r2() {
                        reg_table.set_in_use(store_reg);
                        // mov store_reg r1
                        self.add_opcode(ByteCodes::MOVE_INTO_REG_FROM_REG);
                        self.add_reg(store_reg);
                        self.add_reg(Registers::R1);
                        TnLocation::Register(store_reg)
                    } else {
                        // push r1
                        // We don't care about the stack frame offset since we aren't doing any stack operation besides pushing and popping the value of r1
                        self.push_from_reg(Registers::R1, &mut (REGISTER_SIZE as StackOffset));
                        TnLocation::Stack(0)
                    };

                // Calculate the stack address of the operand
                self.calculate_address_from_stack_frame_offset(*offset, false);

                // Load the operand value
                // mov(n) r2 [r1]
                self.move_into_reg_from_addr_in_reg(size as u8, Registers::R2, Registers::R1);

                // Restore the value of r1
                match r1_store {
                    TnLocation::Register(store_reg) => {
                        // mov r1 store_reg
                        self.add_opcode(ByteCodes::MOVE_INTO_REG_FROM_REG);
                        self.add_reg(store_reg);
                        self.add_reg(Registers::R1);
                        reg_table.set_unused(store_reg);
                    },
                    TnLocation::Stack(_) => {
                        // pop8 r1
                        self.pop8_into_reg(Registers::R1, &mut 0);
                    },
                }
            },
        }
    }


    fn move_unnamed_static_ref_into_addr_in_reg(&mut self, dest: Registers, static_value: Rc<LiteralValue>, labels_to_resolve: &mut Vec<Address>, unnamed_local_statics: &mut UnnamedLocalStaticsManager) {

        let label = unnamed_local_statics.declare(static_value);

        self.add_opcode(ByteCodes::MOVE_INTO_ADDR_IN_REG_FROM_CONST);
        self.add_byte(ADDRESS_SIZE as u8);
        self.add_reg(dest);
        self.add_placeholder_label(label, labels_to_resolve);
    }


    fn push_unnamed_static_ref(&mut self, static_value: Rc<LiteralValue>, labels_to_resolve: &mut Vec<Address>, unnamed_local_statics: &mut UnnamedLocalStaticsManager) {
        // Construct the literal value somewhere alse and push on the stack its memory address
        let label = unnamed_local_statics.declare(static_value);

        self.add_opcode(ByteCodes::PUSH_FROM_CONST);
        self.add_byte(ADDRESS_SIZE as u8);
        self.add_placeholder_label(label, labels_to_resolve);
    }


    /// Load the given argument into r1, assuming it's the first argument being loaded
    fn load_first_arg(&mut self, arg: &IRValue, tn_locations: &mut HashMap<TnID, TnLocation>, labels_to_resolve: &mut Vec<Address>, unnamed_local_statics: &mut UnnamedLocalStaticsManager, static_address_map: &StaticAddressMap) {
        match arg {

            IRValue::Tn(tn) => {

                let tn_location = tn_locations.get(&tn.id).unwrap();

                match tn.data_type.as_ref() {

                    DataType::I8 => self.load_first_numeric_arg(tn_location, I8_SIZE),
                    DataType::I16 => self.load_first_numeric_arg(tn_location, I16_SIZE),
                    DataType::I32 => self.load_first_numeric_arg(tn_location, I32_SIZE),
                    DataType::I64 => self.load_first_numeric_arg(tn_location, I64_SIZE),
                    DataType::U8 => self.load_first_numeric_arg(tn_location, U8_SIZE),
                    DataType::U16 => self.load_first_numeric_arg(tn_location, U16_SIZE),
                    DataType::U32 => self.load_first_numeric_arg(tn_location, U32_SIZE),
                    DataType::U64 => self.load_first_numeric_arg(tn_location, U64_SIZE),
                    DataType::F32 => self.load_first_numeric_arg(tn_location, F32_SIZE),
                    DataType::F64 => self.load_first_numeric_arg(tn_location, F64_SIZE),
                    DataType::Usize => self.load_first_numeric_arg(tn_location, USIZE_SIZE),
                    DataType::Isize => self.load_first_numeric_arg(tn_location, ISIZE_SIZE),
                    // References are usize-sized numbers
                    DataType::Ref { .. } => self.load_first_numeric_arg(tn_location, USIZE_SIZE),
                    // Bools are represented as one byte with a binary state
                    DataType::Bool => self.load_first_numeric_arg(tn_location, BOOL_SIZE),
                    DataType::Char => self.load_first_numeric_arg(tn_location, CHAR_SIZE),

                    DataType::String |
                    DataType::Array { .. } |
                    DataType::StringRef { .. } |
                    DataType::RawString { .. } |
                    DataType::Function { .. } |
                    DataType::Void |
                    DataType::Unspecified
                        => unreachable!("Operation not supported for this type"),
                }
            },

            IRValue::Const(v) => self.load_const(Registers::R1, v, unnamed_local_statics, labels_to_resolve, static_address_map)
        }
    }


    fn load_second_arg(&mut self, arg: &IRValue, tn_locations: &mut HashMap<TnID, TnLocation>, labels_to_resolve: &mut Vec<Address>, unnamed_local_statics: &mut UnnamedLocalStaticsManager, static_address_map: &StaticAddressMap, reg_table: &mut UsedGeneralPurposeRegisterTable) {
        // Be careful because performing calculations to load the right argument into r2 will invalidate r1, which contains the left argument
        match arg {

            IRValue::Tn(tn) => {

                let tn_location = tn_locations.get(&tn.id).unwrap();

                match tn.data_type.as_ref() {

                    DataType::I8 => self.load_second_numeric_arg(tn_location, I8_SIZE, reg_table),
                    DataType::I16 => self.load_second_numeric_arg(tn_location, I16_SIZE, reg_table),
                    DataType::I32 => self.load_second_numeric_arg(tn_location, I32_SIZE, reg_table),
                    DataType::I64 => self.load_second_numeric_arg(tn_location, I64_SIZE, reg_table),
                    DataType::U8 => self.load_second_numeric_arg(tn_location, U8_SIZE, reg_table),
                    DataType::U16 => self.load_second_numeric_arg(tn_location, U16_SIZE, reg_table),
                    DataType::U32 => self.load_second_numeric_arg(tn_location, U32_SIZE, reg_table),
                    DataType::U64 => self.load_second_numeric_arg(tn_location, U64_SIZE, reg_table),
                    DataType::F32 => self.load_second_numeric_arg(tn_location, F32_SIZE, reg_table),
                    DataType::F64 => self.load_second_numeric_arg(tn_location, F64_SIZE, reg_table),
                    DataType::Usize => self.load_second_numeric_arg(tn_location, USIZE_SIZE, reg_table),
                    DataType::Isize => self.load_second_numeric_arg(tn_location, ISIZE_SIZE, reg_table),
                    // References are usize-sized numbers
                    DataType::Ref { .. } => self.load_second_numeric_arg(tn_location, USIZE_SIZE, reg_table),

                    DataType::Bool |
                    DataType::Char |
                    DataType::String |
                    DataType::Array { .. } |
                    DataType::StringRef { .. } |
                    DataType::RawString { .. } |
                    DataType::Function { .. } |
                    DataType::Void |
                    DataType::Unspecified
                        => unreachable!("Operation not supported for this type"),
                }
            },

            IRValue::Const(v) => self.load_const(Registers::R2, v, unnamed_local_statics, labels_to_resolve, static_address_map)
        }
    }


    fn load_const(&mut self, target_reg: Registers, value: &LiteralValue, unnamed_local_statics: &mut UnnamedLocalStaticsManager, labels_to_resolve: &mut Vec<Address>, static_address_map: &StaticAddressMap) {
        match value {

            LiteralValue::Char(ch) => {
                // mov1 target_reg ch
                self.move_into_reg_from_const(CHAR_SIZE as u8, target_reg, *ch as u8);
            },

            LiteralValue::Numeric(n) => {
                // mov(sizeof(n)) target_reg n
                self.move_into_reg_from_const(n.data_type().static_size().unwrap() as u8, target_reg, n);
            },

            LiteralValue::Ref { target, .. } => {
                let label = unnamed_local_statics.declare(Rc::clone(target));
                self.add_placeholder_label(label, labels_to_resolve);
            },

            LiteralValue::Bool(b) => {
                // mov1 target_reg b
                self.move_into_reg_from_const(BOOL_SIZE as u8, target_reg, *b as u8);
            },

            LiteralValue::StaticString(static_id) => {
                let address = *static_address_map.get(static_id).unwrap();
                // mov8 target_reg string_address
                self.move_into_reg_from_const(ADDRESS_SIZE as u8, target_reg, address);
            },

            LiteralValue::Array { .. }
                => unreachable!("Operation not supported for this type")
        }
    }


    fn store_r1(&mut self, target_tn: TnID, reg_table: &mut UsedGeneralPurposeRegisterTable, tn_locations: &mut HashMap<TnID, TnLocation>) {
        match tn_locations.get(&target_tn).unwrap() {

            TnLocation::Register(target_reg) => {
                // mov target_reg r1
                self.move_into_reg_from_reg(*target_reg, Registers::R1);
            },

            TnLocation::Stack(target_offset) => {
                // Save the result value in r1 because it will be overwritten when calculating the target address
                let r1_store =
                    if let Some(store_reg) = reg_table.get_first_unused_register_not_r1r2() {
                        reg_table.set_in_use(store_reg);
                        // mov store_reg r1
                        self.add_opcode(ByteCodes::MOVE_INTO_REG_FROM_REG);
                        self.add_reg(store_reg);
                        self.add_reg(Registers::R1);
                        TnLocation::Register(store_reg)
                    } else {
                        // push r1
                        // We don't care about the stack frame offset since we aren't doing any stack operation besides pushing and popping the value of r1
                        self.push_from_reg(Registers::R1, &mut (REGISTER_SIZE as StackOffset));
                        TnLocation::Stack(0)
                    };

                // Calculate the stack address of the target
                self.calculate_address_from_stack_frame_offset(*target_offset, false);

                // Restore the value of r1
                match r1_store {

                    TnLocation::Register(store_reg) => {
                        // mov r1 store_reg
                        self.add_opcode(ByteCodes::MOVE_INTO_REG_FROM_REG);
                        self.add_reg(store_reg);
                        self.add_reg(Registers::R1);
                        reg_table.set_unused(store_reg);
                    },

                    TnLocation::Stack(_) => {
                        // pop8 r1
                        self.pop8_into_reg(Registers::R1, &mut 0);
                    },
                }
            },
        }
    }

}


/// Stores the unnames local static values and the labels to their definition
struct UnnamedLocalStaticsManager {
    local_statics: Vec<(Rc<LiteralValue>, Label)>,
    label_generator: LabelGenerator
}

impl UnnamedLocalStaticsManager {

    pub fn new(label_generator: LabelGenerator) -> Self {
        Self {
            local_statics: Default::default(),
            label_generator
        }
    }


    pub fn declare(&mut self, value: Rc<LiteralValue>) -> Label {

        // Ensure value uniqueness
        // This approach is O(n), but using a HashSet or HashMap would require float fields in Number to implement Eq and Hash
        for (sv, label) in &self.local_statics {
            if sv.equal(&value) {
                return *label;
            }
        }

        // The value is not in the list
        let label = self.label_generator.next_label();
        self.local_statics.push((value, label));
        label
    }


    pub fn into_iter(self) -> impl IntoIterator<Item = (Rc<LiteralValue>, Label)> {
        self.local_statics.into_iter()
    }

}


/// Generate the code section, equivalent to .text in assembly
pub fn generate_text_section(function_graphs: Vec<FunctionGraph>, labels_to_resolve: &mut Vec<Address>, bc: &mut ByteCode, label_address_map: &mut LabelAddressMap, static_address_map: &StaticAddressMap, symbol_table: &SymbolTable, label_generator: LabelGenerator) {

    // Stores the unnamed local static values. These are, concretely, constants that are created in-place and passed around as references
    let mut unnamed_local_statics = UnnamedLocalStaticsManager::new(label_generator);

    for function_graph in function_graphs {

        let mut reg_table = UsedGeneralPurposeRegisterTable::new();

        // Keeps track of where the actual value of Tns is stored
        // This map is populated when writing the code to load function parameters in the function prologue
        let mut tn_locations: HashMap<TnID, TnLocation> = HashMap::new();

        // Save the previous stack frame and load the new one
        // push sbp
        // mov sbp stp
        bc.push_from_reg(Registers::STACK_FRAME_BASE_POINTER, &mut (REGISTER_SIZE as StackOffset));
        bc.move_into_reg_from_reg(Registers::STACK_FRAME_BASE_POINTER, Registers::STACK_TOP_POINTER);

        // Keeps a record of the current offset from the stack frame base.
        // This is used to keep track of where Tns' real values are located when pushed onto the stack
        let mut stack_frame_offset: StackOffset = 0;

        // Make space for the local variables on the stack
        let stack_frame_size = symbol_table.total_scope_size_excluding_parameters(function_graph.function_scope).expect("The function stack frame size must be known by now");
        // pushsp stack_frame_size
        bc.push_stack_pointer_const(stack_frame_size, &mut stack_frame_offset);

        // TODO: we need to load the function arguments, or at least keep track of where they are (stack and registers).
        // This function should have access to the function's signature to determine which parameters go where
        //
        // TODO: we need to assign a stack location to local variables, too.

        for block in function_graph.code_blocks {

            for ir_node in block.borrow().code.iter() {

                match &ir_node.op {

                    IROperator::Add { target, left, right } => {

                        bc.load_first_arg(left, &mut tn_locations, labels_to_resolve, &mut unnamed_local_statics, static_address_map);

                        bc.load_second_arg(right, &mut tn_locations, labels_to_resolve, &mut unnamed_local_statics, static_address_map, &mut reg_table);

                        if target.data_type.is_float() {
                            bc.add_opcode(ByteCodes::FLOAT_ADD);
                        } else {
                            bc.add_opcode(ByteCodes::INTEGER_ADD);
                        }

                        bc.store_r1(target.id, &mut reg_table, &mut tn_locations);
                    },

                    IROperator::Sub { target, left, right } => {

                        bc.load_first_arg(left, &mut tn_locations, labels_to_resolve, &mut unnamed_local_statics, static_address_map);

                        bc.load_second_arg(right, &mut tn_locations, labels_to_resolve, &mut unnamed_local_statics, static_address_map, &mut reg_table);

                        if target.data_type.is_float() {
                            bc.add_opcode(ByteCodes::FLOAT_SUB);
                        } else {
                            bc.add_opcode(ByteCodes::INTEGER_SUB);
                        }

                        bc.store_r1(target.id, &mut reg_table, &mut tn_locations);
                    },

                    IROperator::Mul { target, left, right } => {

                        bc.load_first_arg(left, &mut tn_locations, labels_to_resolve, &mut unnamed_local_statics, static_address_map);

                        bc.load_second_arg(right, &mut tn_locations, labels_to_resolve, &mut unnamed_local_statics, static_address_map, &mut reg_table);

                        if target.data_type.is_float() {
                            bc.add_opcode(ByteCodes::FLOAT_MUL);
                        } else {
                            bc.add_opcode(ByteCodes::INTEGER_MUL);
                        }

                        bc.store_r1(target.id, &mut reg_table, &mut tn_locations);
                    },

                    IROperator::Div { target, left, right } => {

                        bc.load_first_arg(left, &mut tn_locations, labels_to_resolve, &mut unnamed_local_statics, static_address_map);

                        bc.load_second_arg(right, &mut tn_locations, labels_to_resolve, &mut unnamed_local_statics, static_address_map, &mut reg_table);

                        if target.data_type.is_float() {
                            bc.add_opcode(ByteCodes::FLOAT_DIV);
                        } else {
                            bc.add_opcode(ByteCodes::INTEGER_DIV);
                        }

                        bc.store_r1(target.id, &mut reg_table, &mut tn_locations);
                    },

                    IROperator::Mod { target, left, right } => {

                        bc.load_first_arg(left, &mut tn_locations, labels_to_resolve, &mut unnamed_local_statics, static_address_map);

                        bc.load_second_arg(right, &mut tn_locations, labels_to_resolve, &mut unnamed_local_statics, static_address_map, &mut reg_table);

                        if target.data_type.is_float() {
                            bc.add_opcode(ByteCodes::FLOAT_MOD);
                        } else {
                            bc.add_opcode(ByteCodes::INTEGER_MOD);
                        }

                        bc.store_r1(target.id, &mut reg_table, &mut tn_locations);
                    },

                    IROperator::Assign { target, source } => {

                        let target_location = if let Some(target_location) = tn_locations.get(&target.id) {
                            target_location.clone()
                        } else {
                            // If the tn store wasn't already created, make space for it now
                            // TODO: we could also store tns in free registers as long as they are 8 bytes or less in size
                            // pushsp sizeof(target)
                            bc.push_stack_pointer_const(target.data_type.static_size().unwrap(), &mut stack_frame_offset);
                            let location = TnLocation::Stack(stack_frame_offset);
                            // Record that this tn is stored at that specific address
                            tn_locations.insert(target.id, location.clone());
                            location
                        };

                        match (target_location, source) {

                            (TnLocation::Register(target_reg), IRValue::Tn(source_tn)) => {
                                match tn_locations.get(&source_tn.id).unwrap() {
                                    TnLocation::Register(source_reg) => {
                                        // mov target_reg source_reg
                                        bc.move_into_reg_from_reg(target_reg, *source_reg);
                                    },
                                    TnLocation::Stack(source_offset) => {
                                        // Assume r1 and r2 are free
                                        // Calculate the source tn location
                                        bc.calculate_address_from_stack_frame_offset(*source_offset, false);
                                        // mov(sizeof(target)) target_reg [r1]
                                        bc.move_into_reg_from_addr_in_reg(target.data_type.static_size().unwrap() as u8, target_reg, Registers::R1);
                                    },
                                }
                            },

                            (TnLocation::Register(target_reg), IRValue::Const(source_value)) => {
                                bc.load_const(target_reg, source_value, &mut unnamed_local_statics, labels_to_resolve, static_address_map);
                            },

                            (TnLocation::Stack(target_offset), IRValue::Tn(source_tn)) => {
                                match tn_locations.get(&source_tn.id).unwrap() {
                                    TnLocation::Register(source_reg) => {
                                        bc.calculate_address_from_stack_frame_offset(target_offset, false);
                                        // mov(sizeof(target)) [r1] source_reg
                                        bc.move_into_addr_in_reg_from_reg(target.data_type.static_size().unwrap() as u8, Registers::R1, *source_reg);
                                    },
                                    TnLocation::Stack(source_offset) => {
                                        // Calculate the source address
                                        bc.calculate_address_from_stack_frame_offset(*source_offset, false);
                                        // Store the source address on the stack to free r1
                                        // TODO: storing it in a free register would be faster
                                        bc.push_from_reg(Registers::R1, &mut stack_frame_offset);
                                        // Calculate the target address
                                        bc.calculate_address_from_stack_frame_offset(target_offset, true);
                                        // Copy from source to target
                                        // Reload the source address into r2
                                        // pop8 r2
                                        bc.pop8_into_reg(Registers::R2, &mut stack_frame_offset);
                                        // The target address is already in r1
                                        // memcpyb sizeof(target)
                                        bc.add_opcode(ByteCodes::MEM_COPY_BLOCK_CONST);
                                        bc.add_byte(mem::size_of::<usize>() as u8);
                                        bc.add_const_usize(target.data_type.static_size().unwrap());
                                    },
                                }
                            },

                            (TnLocation::Stack(target_offset), IRValue::Const(source_value)) => {
                                generate_stack_value_at_offset(target_offset, source_value, bc, static_address_map, &mut unnamed_local_statics, labels_to_resolve);
                            },
                        }
                    },

                    IROperator::Deref { target, ref_ } => todo!(),
                    IROperator::DerefAssign { target, source } => todo!(),
                    IROperator::Ref { target, ref_ } => todo!(),
                    IROperator::Greater { target, left, right } => todo!(),
                    IROperator::Less { target, left, right } => todo!(),
                    IROperator::GreaterEqual { target, left, right } => todo!(),
                    IROperator::LessEqual { target, left, right } => todo!(),
                    IROperator::Equal { target, left, right } => todo!(),
                    IROperator::NotEqual { target, left, right } => todo!(),
                    IROperator::BitShiftLeft { target, left, right } => todo!(),
                    IROperator::BitShiftRight { target, left, right } => todo!(),
                    IROperator::BitNot { target, operand } => todo!(),
                    IROperator::BitAnd { target, left, right } => todo!(),
                    IROperator::BitOr { target, left, right } => todo!(),
                    IROperator::BitXor { target, left, right } => todo!(),
                    IROperator::Copy { target, source } => todo!(),
                    IROperator::DerefCopy { target, source } => todo!(),

                    IROperator::Jump { target } => {
                        // jmp target
                        bc.add_opcode(ByteCodes::JUMP);
                        bc.add_placeholder_label(*target, labels_to_resolve);
                    },

                    IROperator::JumpIf { condition, target } => todo!(),
                    IROperator::JumpIfNot { condition, target } => todo!(),

                    IROperator::Label { label } => {
                        // Labels are not actual instructions and don't get translated to any bytecode.
                        // Mark the label as pointing to this specific real location in the bytecode
                        label_address_map.insert(label.0, bc.len());
                    },

                    IROperator::Call { return_target, return_label, callable, args } => {

                        let arg_registers = [
                            Registers::R3,
                            Registers::R4,
                            Registers::R5,
                            Registers::R6,
                            Registers::R7,
                            Registers::R8
                        ];

                        let mut arg_register_it = arg_registers.iter();

                        // TODO: this may be more efficient to implement as a stack array with a fixed size of `arg_registers.len()`
                        let mut registers_to_restore: Vec<Registers> = Vec::new();

                        // Args are pushed in reverse order
                        for arg in args.iter().rev() {

                            match arg {

                                IRValue::Tn(tn) => {

                                    let arg_size = tn.data_type.static_size().expect("Size should be known by now");

                                    if arg_size <= REGISTER_SIZE {
                                        if let Some(arg_reg) = arg_register_it.next() {
                                            // The arg will be passed through a register

                                            if reg_table.is_in_use(*arg_reg) {
                                                // If the register is currently in use, save its current state to the stack and restore it after the function has returned
                                                registers_to_restore.push(*arg_reg);
                                                bc.push_from_reg(*arg_reg, &mut stack_frame_offset);
                                                // Keep track of the moved value
                                                tn_locations.insert(tn.id, TnLocation::Stack(stack_frame_offset)).expect("Tn should exist");
                                            }

                                            bc.push_from_reg(*arg_reg, &mut stack_frame_offset);
                                            continue;
                                        }
                                    }

                                    // The arg must be passed on the stack because it's either too large for a register or there aren't enough registers for all args

                                    match tn_locations.get(&tn.id).unwrap() {

                                        &TnLocation::Register(reg) => {
                                            // The value in the register is to be pushed onto the stack
                                            bc.push_from_reg(reg, &mut stack_frame_offset);
                                        },

                                        &TnLocation::Stack(offset) => {
                                            // The value on the stack is to be copied onto the stack

                                            // Calculate the stack address of the argument to copy
                                            // A positive offset is required because addresses are positive and adding the usize representation of an isize to a usize would overflow
                                            // mov8 r1 sbp
                                            // mov8 r2 abs(offset)
                                            bc.move_into_reg_from_reg(Registers::R1, Registers::STACK_FRAME_BASE_POINTER);
                                            if offset < 0 {
                                                bc.move_into_reg_from_const(ADDRESS_SIZE as u8, Registers::R2, (offset).unsigned_abs());
                                                // isub
                                                bc.add_opcode(ByteCodes::INTEGER_SUB);
                                            } else if offset > 0 {
                                                bc.move_into_reg_from_const(ADDRESS_SIZE as u8, Registers::R2, offset);
                                                // iadd
                                                bc.add_opcode(ByteCodes::INTEGER_ADD);
                                            }
                                            // If the offset is exactly 0, don't perform the operation

                                            // Push the stack pointer to make space for the argument. stp will now point to the uninitialized arg
                                            // pushsp sizeof(arg)
                                            bc.push_stack_pointer_const(arg_size, &mut stack_frame_offset);

                                            // Copy the argument value on the stack into its designated place on the stack
                                            // mov r2 r1 (r1 contains the source address of the argument, which was calculated above)
                                            // mov r1 stp
                                            // memcpyb8 sizeof(arg)
                                            bc.move_into_reg_from_reg(Registers::R2, Registers::R1);
                                            bc.move_into_reg_from_reg(Registers::R1, Registers::STACK_TOP_POINTER);
                                            bc.add_opcode(ByteCodes::MEM_COPY_BLOCK_CONST);
                                            bc.add_byte(8);
                                            bc.add_const_usize(arg_size);
                                        },
                                    }
                                },

                                IRValue::Const(v) => {
                                    generate_push_stack_value(v, bc, static_address_map, &mut stack_frame_offset, &mut unnamed_local_statics, labels_to_resolve);
                                },
                            }

                        }

                        // Clean up after the function has returned (restore previous states)
                        // Restore previous register states, pop function arguments from the stack (popsp)
                        // The stack top will then be the returned value, if it was returned on the stack. Otherwise, it will be located in r1
                        todo!()
                    },

                    IROperator::Return => {
                        // Note that this ir instruction is found only once in the function's ir code

                        // "Pop" the function's stack frame
                        // mov stp sbp
                        bc.move_into_reg_from_reg(Registers::STACK_TOP_POINTER, Registers::STACK_FRAME_BASE_POINTER);
                        // Restore the prevous stack frame
                        // pop8 sbp
                        bc.pop8_into_reg(Registers::STACK_FRAME_BASE_POINTER, &mut stack_frame_offset);

                        // TODO: The return value must be returned.
                        // We need access to the return tn

                        // return
                        bc.add_opcode(ByteCodes::RETURN);
                    },

                    IROperator::Nop => {
                        // nop
                        bc.add_opcode(ByteCodes::NO_OPERATION);
                    },
                }
            }

            // The function epilogue is generated by the ir return instruction
        }
    }

    // Include the unnamed local static values in the byte code
    for (value, label) in unnamed_local_statics.into_iter() {
        // Construct the value directly in the bytecode and make it available under a label
        let address = bc.len();
        label_address_map.insert(label.0, address);
        value.write_to_bytes(bc);
    }

}
