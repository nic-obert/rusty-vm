use std::collections::HashMap;
use std::mem;

use num_traits::ToBytes;

use rusty_vm_lib::assembly::ByteCode;
use rusty_vm_lib::byte_code::ByteCodes;
use rusty_vm_lib::vm::{Address, ADDRESS_SIZE};
use rusty_vm_lib::registers::{Registers, REGISTER_COUNT, REGISTER_SIZE};

use crate::irc::{IROperator, IRValue, Label, LabelID, TnID};
use crate::lang::data_types::{DataType, LiteralValue, BOOL_SIZE, CHAR_SIZE, F32_SIZE, F64_SIZE, I16_SIZE, I32_SIZE, I64_SIZE, I8_SIZE, ISIZE_SIZE, U16_SIZE, U32_SIZE, U64_SIZE, U8_SIZE, USIZE_SIZE};
use crate::symbol_table::{StaticID, SymbolTable};
use crate::flow_analyzer::FunctionGraph;


type LabelAddressMap = HashMap<LabelID, Address>;
type StaticAddressMap = HashMap<StaticID, Address>;


struct ArgsTable {

}


struct UsedRegisterTable {
    registers: [bool; REGISTER_COUNT]
}

impl UsedRegisterTable {

    pub const fn new() -> Self {
        Self {
            registers: [false; REGISTER_COUNT]
        }
    }


    pub const fn is_in_use(&self, reg: Registers) -> bool {
        self.registers[reg as usize]
    }


    pub fn set_in_use(&mut self, reg: Registers) {
        self.registers[reg as usize] = true;
    }


    pub fn set_unused(&mut self, reg: Registers) {
        self.registers[reg as usize] = false;
    }

}


type StackOffset = isize;

enum TnLocation {

    /// The value of the Tn is stored inside a register
    Register (Registers),
    /// The value of the Tn is stores on the stack at an offset
    /// TODO: brobably this doesn't need to be an isize. A i32 or i16 would be good enough
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
fn generate_stack_value(value: &LiteralValue, bytecode: &mut ByteCode, static_address_map: &StaticAddressMap) -> usize {
    match value {

        LiteralValue::Bool(b) => {
            // push1 b
            bytecode.push(ByteCodes::PUSH_FROM_CONST as u8);
            bytecode.push(BOOL_SIZE as u8);
            bytecode.push(*b as u8);
            BOOL_SIZE
        },

        LiteralValue::Char(ch) => {
            // push1 ch
            bytecode.push(ByteCodes::PUSH_FROM_CONST as u8);
            bytecode.push(CHAR_SIZE as u8);
            bytecode.push(*ch as u8);
            CHAR_SIZE
        },

        LiteralValue::StaticString(string_id) => {
            // push8 address of static string
            bytecode.push(ByteCodes::PUSH_FROM_CONST as u8);
            bytecode.push(ADDRESS_SIZE as u8);
            let static_addr = *static_address_map.get(string_id).unwrap();
            bytecode.extend(static_addr.to_le_bytes());
            ADDRESS_SIZE
        },

        LiteralValue::Array { element_type, items } => {
            todo!()
        },

        LiteralValue::Numeric(n) => {
            // push(sizeof(n)) n
            bytecode.push(ByteCodes::PUSH_FROM_CONST as u8);
            let number_size = n.data_type().static_size().unwrap();
            bytecode.push(number_size as u8);
            bytecode.extend(n.to_le_bytes());
            number_size
        },

        LiteralValue::Ref { target, .. } => {
            // Construct the literal value in-place on the stack and load its address as the argument

            // push8 address
            bytecode.push(ByteCodes::PUSH_FROM_CONST as u8);
            bytecode.push(ADDRESS_SIZE as u8);
            // bytecode.extend(iter)
            todo!("Need to know what the ref points to")
        },
    }
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

    fn move_into_reg_from_reg(&mut self, dest: Registers, source: Registers);

    fn move_into_reg_from_const<T>(&mut self, handled_size: u8, reg: Registers, value: T) where T: ToBytes;

    fn move_into_reg_from_addr_in_reg(&mut self, handled_size: u8, dest: Registers, source: Registers);

    fn push_stack_pointer_const(&mut self, offset: usize, stack_frame_offset: &mut StackOffset);

    fn load_first_numeric_arg(&mut self, tn_location: &TnLocation, size: usize);

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

}


/// Generate the code section, equivalent to .text in assembly
fn generate_text_section(function_graphs: Vec<FunctionGraph>, labels_to_resolve: &mut Vec<Address>, bc: &mut ByteCode, label_address_map: &mut LabelAddressMap, static_address_map: &StaticAddressMap) {

    for function_graph in function_graphs {

        // TODO: we need to load the function arguments, or at least keep track of where they are.
        // defining a calling convention is thus necessary at this point.
        // This function should have access to the function's signature

        let mut reg_table = UsedRegisterTable::new();

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
                                        // First, construct the referenced literal value in-place on the stack.
                                        // Then, load its address into r1
                                        todo!()
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
                            IRValue::Tn(_) => todo!(),
                            IRValue::Const(_) => todo!(),
                        }

                        todo!("Perform the addition and move into target")
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
                                            bc.move_into_reg_from_const(ADDRESS_SIZE as u8, Registers::R2, (offset).abs());
                                            if offset < 0 {
                                                // iadd
                                                bc.add_opcode(ByteCodes::INTEGER_ADD);
                                            } else {
                                                // isub
                                                bc.add_opcode(ByteCodes::INTEGER_SUB);
                                            }

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
                                    // The literal value has to be included in the static bytecode and copied onto the stack
                                    // Based on the data type, a different approach is used to include the value into the bytecode
                                    // Some small values can be just be pushed with a PUSH_FROM_CONST instruction, while others need to be constructed on the fly
                                    let stack_offset = match v.as_ref() {
                                        LiteralValue::Bool(b) => {
                                            // push1 b
                                            bc.add_opcode(ByteCodes::PUSH_FROM_CONST);
                                            bc.add_byte(BOOL_SIZE as u8);
                                            bc.add_byte(*b as u8);
                                            BOOL_SIZE as isize
                                        },
                                        LiteralValue::Char(ch) => {
                                            // push1 ch
                                            bc.add_opcode(ByteCodes::PUSH_FROM_CONST);
                                            bc.add_byte(CHAR_SIZE as u8);
                                            bc.add_byte(*ch as u8);
                                            CHAR_SIZE as isize
                                        },
                                        LiteralValue::StaticString(string_id) => {
                                            // push8 address of static string
                                            bc.add_opcode(ByteCodes::PUSH_FROM_CONST);
                                            bc.add_byte(ADDRESS_SIZE as u8);
                                            let static_addr = *static_address_map.get(string_id).unwrap();
                                            bc.extend(static_addr.to_le_bytes());
                                            ADDRESS_SIZE as isize
                                        },
                                        LiteralValue::Array { element_type, items } => {
                                            todo!()
                                        },
                                        LiteralValue::Numeric(n) => {
                                            // push(sizeof(n)) n
                                            bc.add_opcode(ByteCodes::PUSH_FROM_CONST);
                                            let number_size = n.data_type().static_size().unwrap();
                                            bc.add_byte(number_size as u8);
                                            bc.extend(n.to_le_bytes());
                                            number_size as isize
                                        },
                                        LiteralValue::Ref { target, .. } => {
                                            // Construct the literal value in-place on the stack and load its address as the argument


                                            // push8 address
                                            bc.add_opcode(ByteCodes::PUSH_FROM_CONST);
                                            bc.add_byte(ADDRESS_SIZE as u8);
                                            // bytecode.extend(iter)
                                            todo!("Need to know what the ref points to")
                                        },
                                    };
                                    stack_frame_offset -= stack_offset;
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


pub fn generate_bytecode(symbol_table: &SymbolTable, function_graphs: Vec<FunctionGraph>) -> ByteCode {
    /*
        Generate a static section for static data
        Generate a text section for the code
        Substitute labels with actual addresses
    */

    // Map a label to an actual memory address in the bytecode
    let mut label_address_map = LabelAddressMap::new();
    // Maps a static data id to an actual memory address in the bytecode
    let mut static_address_map = StaticAddressMap::new();
    // List of labels that will need to be filled in later, when all label addresses are known.
    let mut labels_to_resolve: Vec<Address> = Vec::new();

    let mut bytecode = ByteCode::new();

    generate_static_data_section(symbol_table, &mut static_address_map, &mut bytecode);

    generate_text_section(function_graphs, &mut labels_to_resolve, &mut bytecode, &mut label_address_map, &static_address_map);

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
