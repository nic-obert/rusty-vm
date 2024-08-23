use std::collections::HashMap;
use std::mem;

use rusty_vm_lib::assembly::ByteCode;
use rusty_vm_lib::byte_code::ByteCodes;
use rusty_vm_lib::vm::{Address, ADDRESS_SIZE};
use rusty_vm_lib::registers::{Registers, REGISTER_COUNT, REGISTER_SIZE};

use crate::irc::{IROperator, IRValue, LabelID, TnID};
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


/// Generate the code section, equivalent to .text in assembly
fn generate_text_section(function_graphs: Vec<FunctionGraph>, labels_to_resolve: &mut Vec<Address>, bytecode: &mut ByteCode, label_address_map: &mut LabelAddressMap) {

    for function_graph in function_graphs {

        // TODO: we need to load the function arguments, or at least keep track of where they are.
        // defining a calling convention is thus necessary at this point.
        // This function should have access to the function's signature

        let mut reg_table = UsedRegisterTable::new();

        // Keeps track of where the actual value of Tns is stored
        let mut tn_location: HashMap<TnID, TnLocation> = HashMap::new();

        // TODO: initialize the stack frame

        // Keeps a record of the current offset from the stack frame base.
        // This is used to keep track of where Tns' real values are located when pushed onto the stack
        let mut stack_frame_offset: StackOffset = 0;

        for block in function_graph.code_blocks {

            for ir_node in block.borrow().code.iter() {

                macro_rules! add_byte {
                    ($byte:expr) => {
                        bytecode.push($byte as u8);
                    }
                }

                macro_rules! add_const_usize {
                    ($val:expr) => {
                        bytecode.extend(($val as usize).to_le_bytes());
                    }
                }

                macro_rules! set_reg_const_usize {
                    ($reg:path, $val:expr) => {
                        add_byte!(ByteCodes::MOVE_INTO_REG_FROM_CONST);
                        add_byte!(mem::size_of::<usize>());
                        add_byte!($reg);
                        bytecode.extend(($val as usize).to_le_bytes());
                    }
                }

                macro_rules! move_into_reg_from_reg {
                    ($reg1:path, $reg2:path) => {
                        add_byte!(ByteCodes::MOVE_INTO_REG_FROM_REG);
                        add_byte!($reg1);
                        add_byte!($reg2);
                    }
                }

                macro_rules! push_from_reg {
                    ($reg:expr) => {
                        add_byte!(ByteCodes::PUSH_FROM_REG);
                        add_byte!($reg);
                        stack_frame_offset -= REGISTER_SIZE as StackOffset;
                    }
                }

                macro_rules! push_stack_pointer_const {
                    ($offset:expr) => {
                        add_byte!(ByteCodes::PUSH_STACK_POINTER_CONST);
                        add_byte!(mem::size_of::<usize>());
                        bytecode.extend(($offset as Address).to_le_bytes());
                        stack_frame_offset -= $offset as isize;
                    }
                }

                // macro_rules! push_from_

                macro_rules! placeholder_label {
                    ($label:ident) => {
                        labels_to_resolve.push(bytecode.len());
                        bytecode.extend(($label.0.0).to_le_bytes());
                    }
                }

                match &ir_node.op {

                    IROperator::Add { target, left, right } => {
                        // TODO: we need to keep a record of which Tns map to which memory address in the stack.
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
                        add_byte!(ByteCodes::JUMP);
                        placeholder_label!(target);
                    },

                    IROperator::JumpIf { condition, target } => todo!(),
                    IROperator::JumpIfNot { condition, target } => todo!(),

                    IROperator::Label { label } => {
                        // Labels are not actual instructions and don't get translated to any bytecode.
                        // Mark the label as pointing to this specific real location in the bytecode
                        label_address_map.insert(label.0, bytecode.len());
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
                                                push_from_reg!(*arg_reg);
                                                // Keep track of the moved value
                                                tn_location.insert(tn.id, TnLocation::Stack(stack_frame_offset)).expect("Tn should exist");
                                            }

                                            push_from_reg!(*arg_reg);
                                            continue;
                                        }
                                    }

                                    // The arg must be passed on the stack because it's either too large for a register or there aren't enough registers for all args

                                    match tn_location.get(&tn.id).unwrap() {

                                        &TnLocation::Register(reg) => {
                                            // The value in the register is to be pushed onto the stack
                                            push_from_reg!(reg);
                                        },

                                        &TnLocation::Stack(offset) => {
                                            // The value on the stack is to be copied onto the stack

                                            // Calculate the stack address of the argument to copy
                                            // A positive offset is required because addresses are positive and adding the usize representation of an isize to a usize would overflow
                                            // mov8 r1 sbp
                                            // mov8 r2 abs(offset)
                                            move_into_reg_from_reg!(Registers::R1, Registers::STACK_FRAME_BASE_POINTER);
                                            set_reg_const_usize!(Registers::R2, (offset).abs());
                                            if offset < 0 {
                                                // iadd
                                                add_byte!(ByteCodes::INTEGER_ADD);
                                            } else {
                                                // isub
                                                add_byte!(ByteCodes::INTEGER_SUB);
                                            }

                                            // Push the stack pointer to make space for the argument. stp will now point to the uninitialized arg
                                            // pushsp sizeof(arg)
                                            push_stack_pointer_const!(arg_size);

                                            // Copy the argument value on the stack into its designated place on the stack
                                            // mov r2 r1 (r1 contains the source address of the argument, which was calculated above)
                                            // mov r1 stp
                                            // memcpyb8 sizeof(arg)
                                            move_into_reg_from_reg!(Registers::R2, Registers::R1);
                                            move_into_reg_from_reg!(Registers::R1, Registers::STACK_TOP_POINTER);
                                            add_byte!(ByteCodes::MEM_COPY_BLOCK_CONST);
                                            add_byte!(8);
                                            add_const_usize!(arg_size);
                                        },
                                    }
                                },

                                IRValue::Const(v) => {
                                    // The literal value is to be copied onto the stack
                                    // pushsp8 sizeof(arg)
                                    // mov r1 stp
                                    // mov8 r2
                                    //
                                },
                            }

                        }

                        // Clean up after the function has returned (restore previous states)
                        todo!()
                    },

                    IROperator::Return => todo!(),

                    &IROperator::PushScope { bytes } => {
                        set_reg_const_usize!(Registers::R1, bytes);
                        move_into_reg_from_reg!(Registers::R2, Registers::STACK_TOP_POINTER);
                        // The stack grows downwards
                        add_byte!(ByteCodes::INTEGER_SUB);
                        move_into_reg_from_reg!(Registers::STACK_TOP_POINTER, Registers::R1);
                    },
                    &IROperator::PopScope { bytes } => {
                        set_reg_const_usize!(Registers::R1, bytes);
                        move_into_reg_from_reg!(Registers::R2, Registers::STACK_TOP_POINTER);
                        add_byte!(ByteCodes::INTEGER_ADD);
                        move_into_reg_from_reg!(Registers::STACK_TOP_POINTER, Registers::R1);
                    },

                    IROperator::Nop => {
                        add_byte!(ByteCodes::NO_OPERATION);
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

    generate_text_section(function_graphs, &mut labels_to_resolve, &mut bytecode, &mut label_address_map);

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
