use std::collections::HashMap;
use std::mem;

use rusty_vm_lib::assembly::ByteCode;
use rusty_vm_lib::byte_code::ByteCodes;
use rusty_vm_lib::vm::Address;
use rusty_vm_lib::registers::{Registers, GENERAL_PURPOSE_REGISTER_COUNT};

use crate::irc::{LabelID, IROperator};
use crate::symbol_table::{StaticID, SymbolTable};
use crate::flow_analyzer::FunctionGraph;


type LabelAddressMap = HashMap<LabelID, Address>;
type StaticAddressMap = HashMap<StaticID, Address>;


// struct GeneralPurposeRegisterSet {
//     registers: Vec<bool>,
// }

// impl GeneralPurposeRegisterSet {

//     pub fn new() -> Self {
//         Self {
//             registers: vec![false; GENERAL_PURPOSE_REGISTER_COUNT]
//         }
//     }


//     pub fn set(&mut self, reg: Registers) {
//         self.registers[reg as usize] = true;
//     }


//     pub fn is_set(&mut self, reg: Registers) -> bool {
//         self.registers[reg as usize]
//     }


//     pub fn clear(&mut self, reg: Registers) {
//         self.registers[reg as usize] = false;
//     }

// }


pub fn generate_bytecode(symbol_table: &SymbolTable, function_graphs: Vec<FunctionGraph>) -> ByteCode {
    /*
        Generate a static section for static data
        Generate a text section for the code
        Substitute labels with actual addresses
    */

    let mut label_address_map = LabelAddressMap::new();
    let mut static_address_map = StaticAddressMap::new();
    // let mut reg_set = GeneralPurposeRegisterSet::new();

    let mut bytecode = ByteCode::new();

    // Generate static data section (equivalent to .data section in assembly)

    for (static_id, static_value) in symbol_table.get_statics() {

        let static_size = static_value.data_type.static_size().unwrap_or_else(
            |()| panic!("Could not determine static size of {:?}. This is a bug.", static_value.data_type)
        );

        let byte_repr = static_value.value.as_bytes();

        assert_eq!(static_size, byte_repr.len());

        static_address_map.insert(static_id, bytecode.len());

        bytecode.extend(byte_repr);
    }

    // Generate the code section (equivalent to .text section in assembly)
    // And also populate the label-address map

    // List of labels that will need to be filled in later, when all label addresses are known.
    let mut labels_to_resolve: Vec<Address> = Vec::new();
    
    for function_graph in function_graphs {

        for block in function_graph {

            for ir_node in block.borrow().code.iter() {

                macro_rules! pushbc {
                    ($instruction:path) => {
                        bytecode.push($instruction as u8);
                    }
                }

                macro_rules! set_reg_const {
                    ($reg:path, $val:expr) => {
                        pushbc!(ByteCodes::MOVE_INTO_REG_FROM_CONST);
                        bytecode.extend(mem::size_of::<usize>().to_le_bytes());
                        pushbc!($reg);
                        bytecode.extend(($val).to_le_bytes());
                    }
                }

                macro_rules! move_into_reg_from_reg {
                    ($reg1:path, $reg2:path) => {
                        pushbc!(ByteCodes::MOVE_INTO_REG_FROM_REG);
                        pushbc!($reg1);
                        pushbc!($reg2);
                    }
                }

                macro_rules! placeholder_label {
                    ($label:ident) => {
                        labels_to_resolve.push(bytecode.len());
                        bytecode.extend(($label.0.0).to_le_bytes());
                    }
                }

                match &ir_node.op {

                    IROperator::Add { target, left, right } => {
                        // TODO: we need to keep a record of which tns map to which memory address in the stack.                
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
                        pushbc!(ByteCodes::JUMP);
                        placeholder_label!(target);
                    },

                    IROperator::JumpIf { condition, target } => todo!(),
                    IROperator::JumpIfNot { condition, target } => todo!(),

                    IROperator::Label { label } => {
                        label_address_map.insert(label.0, bytecode.len());
                    },

                    IROperator::Call { return_target, return_label, callable, args } => todo!(),
                    IROperator::Return => todo!(),

                    IROperator::PushScope { bytes } => {
                        set_reg_const!(Registers::R1, bytes);
                        move_into_reg_from_reg!(Registers::R2, Registers::STACK_BASE_POINTER);
                        // The stack grows downwards
                        pushbc!(ByteCodes::INTEGER_SUB);
                        move_into_reg_from_reg!(Registers::STACK_BASE_POINTER, Registers::R1);
                    },
                    IROperator::PopScope { bytes } => {
                        set_reg_const!(Registers::R1, bytes);
                        move_into_reg_from_reg!(Registers::R2, Registers::STACK_BASE_POINTER);
                        pushbc!(ByteCodes::INTEGER_ADD);
                        move_into_reg_from_reg!(Registers::STACK_BASE_POINTER, Registers::R1);
                    },

                    IROperator::Nop => {
                        pushbc!(ByteCodes::NO_OPERATION);
                    },
                }
            }
        }
    }

    // Substitute labels with actual addresses
    for label_location in labels_to_resolve {
        let label_id = LabelID(usize::from_le_bytes(
            bytecode[label_location..label_location + mem::size_of::<LabelID>()].try_into().unwrap()
        ));
        let address = label_address_map.get(&label_id).unwrap();
        bytecode[label_location..label_location + mem::size_of::<LabelID>()].copy_from_slice(&address.to_le_bytes());
    }

    // Specify the entry point of the program (main function or __init__ function)

    bytecode
}

