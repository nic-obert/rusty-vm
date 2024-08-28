use std::collections::HashMap;
use std::mem;
use std::rc::Rc;

use num_traits::ToBytes;

use rusty_vm_lib::assembly::ByteCode;
use rusty_vm_lib::byte_code::ByteCodes;
use rusty_vm_lib::vm::{Address, ADDRESS_SIZE};
use rusty_vm_lib::registers::{Registers, GENERAL_PURPOSE_REGISTER_COUNT, REGISTER_COUNT, REGISTER_SIZE};

use crate::irc::{IROperator, IRValue, Label, LabelID, TnID, IRIDGenerator};
use crate::lang::data_types::{DataType, LiteralValue, BOOL_SIZE, CHAR_SIZE, F32_SIZE, F64_SIZE, I16_SIZE, I32_SIZE, I64_SIZE, I8_SIZE, ISIZE_SIZE, U16_SIZE, U32_SIZE, U64_SIZE, U8_SIZE, USIZE_SIZE};
use crate::symbol_table::{StaticID, SymbolTable};
use crate::flow_analyzer::FunctionGraph;


type LabelAddressMap = HashMap<LabelID, Address>;
type StaticAddressMap = HashMap<StaticID, Address>;


struct ArgsTable {

}


struct LabelGenerator {
    next_label: LabelID
}

impl LabelGenerator {

    pub fn from_irid_generator(irid_gen: IRIDGenerator) -> Self {
        Self {
            next_label: irid_gen.extract_next_label()
        }
    }


    pub fn next_label(&mut self) -> Label {
        let old = self.next_label;
        self.next_label = LabelID(old.0 + 1);
        Label(old)
    }

}


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

enum TnLocation {

    /// The value of the Tn is stored inside a register
    Register (Registers),
    /// The value of the Tn is stores on the stack at an offset
    Stack (StackOffset)

}


/// Generate static data section, equivalent to .data section in assembly
fn generate_static_data_section(symbol_table: &SymbolTable, static_address_map: &mut StaticAddressMap, bytecode: &mut ByteCode) {

    for (static_id, static_value) in symbol_table.get_statics() {

        let static_size = static_value.data_type.static_size().unwrap_or_else(
            |()| panic!("Could not determine static size of {:?}. This is a bug.", static_value.data_type)
        );

        let byte_repr = static_value.value.as_bytes();

        assert_eq!(static_size, byte_repr.len());

        static_address_map.insert(static_id, bytecode.len());

        bytecode.extend(byte_repr);
    }

}


/// Generate the code to construct the given value in-place on the stack.
/// Return the amount of bytes the stack pointer was pushed to perform this operation.
/// The generated value will be placed at the top of the stack.
/// Based on the data type, a different approach is used to include the value into the bytecode.
/// Some small values can be just be pushed with a PUSH_FROM_CONST instruction, while others need to be constructed on the fly.
fn generate_stack_value(value: &LiteralValue, bc: &mut ByteCode, static_address_map: &StaticAddressMap, stack_frame_offset: &mut StackOffset, unnamed_local_statics: &mut Vec<(Rc<LiteralValue>, Label)>, labels_to_resolve: &mut Vec<Address>, label_generator: &mut LabelGenerator) {
    *stack_frame_offset
    -= match value {

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
            bc.extend(static_addr.to_le_bytes());
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
                generate_stack_value(item, bc, static_address_map, stack_frame_offset, unnamed_local_statics, labels_to_resolve, label_generator);
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
            bc.push_unnamed_static_ref(Rc::clone(target), labels_to_resolve, label_generator, unnamed_local_statics);
            ADDRESS_SIZE
        },
    } as StackOffset;
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

    fn push_unnamed_static_ref(&mut self, static_value: Rc<LiteralValue>, labels_to_resolve: &mut Vec<Address>, label_generator: &mut LabelGenerator, unnamed_local_statics: &mut Vec<(Rc<LiteralValue>, Label)>);

}

impl ByteCodeOutput for ByteCode {

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
        self.extend(offset.to_le_bytes());
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
                // mov r1 sbp
                // mov8 r2 abs(offset)
                self.move_into_reg_from_reg(Registers::R1, Registers::STACK_FRAME_BASE_POINTER);
                self.move_into_reg_from_const(ADDRESS_SIZE as u8, Registers::R2, (*offset).abs());
                if *offset < 0 {
                    // isub
                    self.add_opcode(ByteCodes::INTEGER_SUB);
                } else {
                    // iadd
                    self.add_opcode(ByteCodes::INTEGER_ADD);
                }
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
                // mov r1 sbp
                // mov8 r2 abs(offset)
                self.move_into_reg_from_reg(Registers::R1, Registers::STACK_FRAME_BASE_POINTER);
                self.move_into_reg_from_const(ADDRESS_SIZE as u8, Registers::R2, (*offset).abs());
                if *offset < 0 {
                    // isub
                    self.add_opcode(ByteCodes::INTEGER_SUB);
                } else {
                    // iadd
                    self.add_opcode(ByteCodes::INTEGER_ADD);
                }
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
                    },
                    TnLocation::Stack(_) => {
                        // pop8 r1
                        self.pop8_into_reg(Registers::R1, &mut 0);
                    },
                }
            },
        }
    }


    fn push_unnamed_static_ref(&mut self, static_value: Rc<LiteralValue>, labels_to_resolve: &mut Vec<Address>, label_generator: &mut LabelGenerator, unnamed_local_statics: &mut Vec<(Rc<LiteralValue>, Label)>) {
        // Construct the literal value somewhere alse and push on the stack its memory address
        let label = declare_unnamed_local_static_ref(static_value, label_generator, unnamed_local_statics);

        self.add_opcode(ByteCodes::PUSH_FROM_CONST);
        self.add_byte(ADDRESS_SIZE as u8);
        self.add_placeholder_label(label, labels_to_resolve);
    }

}


fn declare_unnamed_local_static_ref(static_value: Rc<LiteralValue>, label_generator: &mut LabelGenerator, unnamed_local_statics: &mut Vec<(Rc<LiteralValue>, Label)>) -> Label {
    let label = label_generator.next_label();
    unnamed_local_statics.push((static_value, label));
    label
}


/// Generate the code section, equivalent to .text in assembly
fn generate_text_section(function_graphs: Vec<FunctionGraph>, labels_to_resolve: &mut Vec<Address>, bc: &mut ByteCode, label_address_map: &mut LabelAddressMap, static_address_map: &StaticAddressMap, symbol_table: &SymbolTable, label_generator: &mut LabelGenerator) {

    // Stores the unnamed local static values. These are, concretely, constants that are created in-place and passed around as references
    let mut unnamed_local_statics: Vec<(Rc<LiteralValue>, Label)> = Vec::new();

    for function_graph in function_graphs {

        // TODO: we need to load the function arguments, or at least keep track of where they are.
        // defining a calling convention is thus necessary at this point.
        // This function should have access to the function's signature

        let mut reg_table = UsedGeneralPurposeRegisterTable::new();

        // Keeps track of where the actual value of Tns is stored
        let mut tn_locations: HashMap<TnID, TnLocation> = HashMap::new();

        // TODO: initialize the stack frame

        // Keeps a record of the current offset from the stack frame base.
        // This is used to keep track of where Tns' real values are located when pushed onto the stack
        let mut stack_frame_offset: StackOffset = 0;

        for block in function_graph.code_blocks {

            for ir_node in block.borrow().code.iter() {

                match &ir_node.op {

                    IROperator::Add { target, left, right } => {

                        match left {

                            IRValue::Tn(tn) => {

                                let tn_location = tn_locations.get(&tn.id).unwrap();

                                match tn.data_type.as_ref() {

                                    DataType::I8 => bc.load_first_numeric_arg(tn_location, I8_SIZE),
                                    DataType::I16 => bc.load_first_numeric_arg(tn_location, I16_SIZE),
                                    DataType::I32 => bc.load_first_numeric_arg(tn_location, I32_SIZE),
                                    DataType::I64 => bc.load_first_numeric_arg(tn_location, I64_SIZE),
                                    DataType::U8 => bc.load_first_numeric_arg(tn_location, U8_SIZE),
                                    DataType::U16 => bc.load_first_numeric_arg(tn_location, U16_SIZE),
                                    DataType::U32 => bc.load_first_numeric_arg(tn_location, U32_SIZE),
                                    DataType::U64 => bc.load_first_numeric_arg(tn_location, U64_SIZE),
                                    DataType::F32 => bc.load_first_numeric_arg(tn_location, F32_SIZE),
                                    DataType::F64 => bc.load_first_numeric_arg(tn_location, F64_SIZE),
                                    DataType::Usize => bc.load_first_numeric_arg(tn_location, USIZE_SIZE),
                                    DataType::Isize => bc.load_first_numeric_arg(tn_location, ISIZE_SIZE),
                                    // References are usize-sized numbers
                                    DataType::Ref { .. } => bc.load_first_numeric_arg(tn_location, USIZE_SIZE),

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
                            IRValue::Const(v) => {
                                match v.as_ref() {
                                    LiteralValue::Char(ch) => {
                                        // mov1 r1 ch
                                        bc.move_into_reg_from_const(CHAR_SIZE as u8, Registers::R1, *ch as u8);
                                    },
                                    LiteralValue::Numeric(n) => {
                                        // mov(sizeof(n)) r1 n
                                        bc.move_into_reg_from_const(n.data_type().static_size().unwrap() as u8, Registers::R1, n);
                                    },
                                    LiteralValue::Ref { target, .. } => {
                                        let label = declare_unnamed_local_static_ref(Rc::clone(target), label_generator, &mut unnamed_local_statics);
                                        bc.add_placeholder_label(label, labels_to_resolve);
                                    },

                                    LiteralValue::Array { .. } |
                                    LiteralValue::StaticString(_) |
                                    LiteralValue::Bool(_)
                                        => unreachable!("Operation not supported for this type")
                                }
                            },
                        }

                        // Be careful because performing calculations to load the right argument into r2 will invalidate r1, which contains the left argument
                        match right {

                            IRValue::Tn(tn) => {

                                let tn_location = tn_locations.get(&tn.id).unwrap();

                                match tn.data_type.as_ref() {

                                    DataType::I8 => bc.load_second_numeric_arg(tn_location, I8_SIZE, &mut reg_table),
                                    DataType::I16 => bc.load_second_numeric_arg(tn_location, I16_SIZE, &mut reg_table),
                                    DataType::I32 => bc.load_second_numeric_arg(tn_location, I32_SIZE, &mut reg_table),
                                    DataType::I64 => bc.load_second_numeric_arg(tn_location, I64_SIZE, &mut reg_table),
                                    DataType::U8 => bc.load_second_numeric_arg(tn_location, U8_SIZE, &mut reg_table),
                                    DataType::U16 => bc.load_second_numeric_arg(tn_location, U16_SIZE, &mut reg_table),
                                    DataType::U32 => bc.load_second_numeric_arg(tn_location, U32_SIZE, &mut reg_table),
                                    DataType::U64 => bc.load_second_numeric_arg(tn_location, U64_SIZE, &mut reg_table),
                                    DataType::F32 => bc.load_second_numeric_arg(tn_location, F32_SIZE, &mut reg_table),
                                    DataType::F64 => bc.load_second_numeric_arg(tn_location, F64_SIZE, &mut reg_table),
                                    DataType::Usize => bc.load_second_numeric_arg(tn_location, USIZE_SIZE, &mut reg_table),
                                    DataType::Isize => bc.load_second_numeric_arg(tn_location, ISIZE_SIZE, &mut reg_table),
                                    // References are usize-sized numbers
                                    DataType::Ref { .. } => bc.load_second_numeric_arg(tn_location, USIZE_SIZE, &mut reg_table),

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
                            IRValue::Const(v) => {
                                match v.as_ref() {
                                    LiteralValue::Char(ch) => {
                                        // mov1 r2 ch
                                        bc.move_into_reg_from_const(CHAR_SIZE as u8, Registers::R2, *ch as u8);
                                    },
                                    LiteralValue::Numeric(n) => {
                                        // mov(sizeof(n)) r2 n
                                        bc.move_into_reg_from_const(n.data_type().static_size().unwrap() as u8, Registers::R2, n);
                                    },
                                    LiteralValue::Ref { target, .. } => {
                                        let label = declare_unnamed_local_static_ref(Rc::clone(target), label_generator, &mut unnamed_local_statics);
                                        bc.add_placeholder_label(label, labels_to_resolve);
                                    },

                                    LiteralValue::Array { .. } |
                                    LiteralValue::StaticString(_) |
                                    LiteralValue::Bool(_)
                                        => unreachable!("Operation not supported for this type")
                                }
                            },
                        }

                        match target.data_type.as_ref() {

                            DataType::Char |
                            DataType::Ref { .. } |
                            DataType::I8 |
                            DataType::I16 |
                            DataType::I32 |
                            DataType::I64 |
                            DataType::U8 |
                            DataType::U16 |
                            DataType::U32 |
                            DataType::U64 |
                            DataType::Usize |
                            DataType::Isize
                                => bc.add_opcode(ByteCodes::INTEGER_ADD),

                            DataType::F32 |
                            DataType::F64
                                => bc.add_opcode(ByteCodes::FLOAT_ADD),

                            DataType::StringRef { .. } |
                            DataType::RawString { .. } |
                            DataType::String |
                            DataType::Array { .. } |
                            DataType::Bool |
                            DataType::Function { .. } |
                            DataType::Void |
                            DataType::Unspecified
                                => unreachable!("Operation not supported for this type")
                        }

                        match tn_locations.get(&target.id).unwrap() {
                            TnLocation::Register(target_reg) => {
                                // mov target_reg r1
                                bc.move_into_reg_from_reg(*target_reg, Registers::R1);
                            },
                            TnLocation::Stack(target_offset) => {
                                // Save the result value in r1 because it will be overwritten when calculating the target address
                                let r1_store =
                                    if let Some(store_reg) = reg_table.get_first_unused_register_not_r1r2() {
                                        reg_table.set_in_use(store_reg);
                                        // mov store_reg r1
                                        bc.add_opcode(ByteCodes::MOVE_INTO_REG_FROM_REG);
                                        bc.add_reg(store_reg);
                                        bc.add_reg(Registers::R1);
                                        TnLocation::Register(store_reg)
                                    } else {
                                        // push r1
                                        // We don't care about the stack frame offset since we aren't doing any stack operation besides pushing and popping the value of r1
                                        bc.push_from_reg(Registers::R1, &mut (REGISTER_SIZE as StackOffset));
                                        TnLocation::Stack(0)
                                    };

                                // Calculate the stack address of the target
                                // mov r1 sbp
                                // mov8 r2 abs(target_offset)
                                bc.move_into_reg_from_reg(Registers::R1, Registers::STACK_FRAME_BASE_POINTER);
                                if *target_offset < 0 {
                                    bc.move_into_reg_from_const(ADDRESS_SIZE as u8, Registers::R2, (*target_offset).unsigned_abs());
                                    // iadd
                                    bc.add_opcode(ByteCodes::INTEGER_ADD);
                                } else if *target_offset > 0 {
                                    bc.move_into_reg_from_const(ADDRESS_SIZE as u8, Registers::R2, *target_offset);
                                    // isub
                                    bc.add_opcode(ByteCodes::INTEGER_SUB);
                                }
                                // if target_offset is 0, don't perform the operation

                                // Restore the value of r1
                                match r1_store {
                                    TnLocation::Register(store_reg) => {
                                        // mov r1 store_reg
                                        bc.add_opcode(ByteCodes::MOVE_INTO_REG_FROM_REG);
                                        bc.add_reg(store_reg);
                                        bc.add_reg(Registers::R1);
                                    },
                                    TnLocation::Stack(_) => {
                                        // pop8 r1
                                        bc.pop8_into_reg(Registers::R1, &mut 0);
                                    },
                                }
                            },
                        }
                    },
                    IROperator::Sub { target, left, right } => todo!(),
                    IROperator::Mul { target, left, right } => todo!(),
                    IROperator::Div { target, left, right } => todo!(),
                    IROperator::Mod { target, left, right } => todo!(),
                    IROperator::Assign { target, source } => todo!(),
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
                                    generate_stack_value(v, bc, static_address_map, &mut stack_frame_offset, &mut unnamed_local_statics, labels_to_resolve, label_generator);
                                },
                            }

                        }

                        // Clean up after the function has returned (restore previous states)
                        todo!()
                    },

                    IROperator::Return => todo!(),

                    &IROperator::PushScope { bytes } => {
                        bc.move_into_reg_from_const(ADDRESS_SIZE as u8, Registers::R1, bytes);
                        bc.move_into_reg_from_reg(Registers::R2, Registers::STACK_TOP_POINTER);
                        // The stack grows downwards
                        bc.add_opcode(ByteCodes::INTEGER_SUB);
                        bc.move_into_reg_from_reg(Registers::STACK_TOP_POINTER, Registers::R1);
                    },
                    &IROperator::PopScope { bytes } => {
                        bc.move_into_reg_from_const(ADDRESS_SIZE as u8, Registers::R1, bytes);
                        bc.move_into_reg_from_reg(Registers::R2, Registers::STACK_TOP_POINTER);
                        bc.add_opcode(ByteCodes::INTEGER_ADD);
                        bc.move_into_reg_from_reg(Registers::STACK_TOP_POINTER, Registers::R1);
                    },

                    IROperator::Nop => {
                        bc.add_opcode(ByteCodes::NO_OPERATION);
                    },
                }
            }
        }
    }

    // Include the unnamed local static values in the byte code
    for (value, label) in unnamed_local_statics {
        // Construct the value directly in the bytecode and make it available under a label
        let address = bc.len();
        label_address_map.insert(label.0, address);
        value.write_to_bytes(bc);
    }

}


fn resolve_unresolved_addresses(labels_to_resolve: Vec<Address>, label_address_map: LabelAddressMap, bytecode: &mut ByteCode) {

    // Substitute labels with actual addresses
    for label_location in labels_to_resolve {
        let label_id = LabelID(usize::from_le_bytes(
            bytecode[label_location..label_location + mem::size_of::<LabelID>()].try_into().unwrap()
        ));
        let address = label_address_map.get(&label_id).unwrap();
        bytecode[label_location..label_location + mem::size_of::<LabelID>()].copy_from_slice(&address.to_le_bytes());
    }

}


pub fn generate_bytecode(symbol_table: &SymbolTable, function_graphs: Vec<FunctionGraph>, irid_generator: IRIDGenerator) -> ByteCode {
    /*
        Generate a static section for static data
        Generate a text section for the code
        Substitute labels with actual addresses
        Note that, since labels are hardcoded into the binary, the binary cannot be mutated after being generated.
        Inserting or removing instructions would fuck up the jump addresses and labels.
    */

    let mut label_generator = LabelGenerator::from_irid_generator(irid_generator);

    // Map a label to an actual memory address in the bytecode
    let mut label_address_map = LabelAddressMap::new();
    // Maps a static data id to an actual memory address in the bytecode
    let mut static_address_map = StaticAddressMap::new();
    // List of labels that will need to be filled in later, when all label addresses are known.
    let mut labels_to_resolve: Vec<Address> = Vec::new();

    let mut bytecode = ByteCode::new();

    generate_static_data_section(symbol_table, &mut static_address_map, &mut bytecode);

    generate_text_section(function_graphs, &mut labels_to_resolve, &mut bytecode, &mut label_address_map, &static_address_map, symbol_table, &mut label_generator);

    resolve_unresolved_addresses(labels_to_resolve, label_address_map, &mut bytecode);

    // Specify the entry point of the program (main function or __init__ function)
    todo!();

    bytecode
}

/*
    TODO
    Possible optimizations:

    Determine if a function performs any call to other functions. If a function doesn't perform any further function call, there's no need to increment
    the stack pointer, since local stack variables are accessed through an offset from the stack frame base.


*/
