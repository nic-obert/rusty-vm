use std::collections::HashMap;

use rusty_vm_lib::assembly::ByteCode;
use rusty_vm_lib::vm::Address;

use crate::irc::{LabelID, IROperator};
use crate::symbol_table::{StaticID, SymbolTable};
use crate::flow_analyzer::FunctionGraph;


type LabelAddressMap = HashMap<LabelID, Address>;
type StaticAddressMap = HashMap<StaticID, Address>;


pub fn generate_bytecode(symbol_table: &SymbolTable, function_graphs: Vec<FunctionGraph>) -> ByteCode {
    /*
        Generate a static section for static data
        Generate a text section for the code
        Substitute labels with actual addresses
    */

    let mut label_address_map = LabelAddressMap::new();
    let mut static_address_map = StaticAddressMap::new();

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

    let mut labels_to_resolve: Vec<Address> = Vec::new();
    
    for function_graph in function_graphs {

        for block in function_graph {

            for ir_node in block.borrow().code.iter() {

                match &ir_node.op {
                    IROperator::Add { target, left, right } => todo!(),
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
                    IROperator::Jump { target } => todo!(),
                    IROperator::JumpIf { condition, target } => todo!(),
                    IROperator::JumpIfNot { condition, target } => todo!(),
                    IROperator::Label { label } => todo!(),
                    IROperator::Call { return_target, return_label, callable, args } => todo!(),
                    IROperator::Return => todo!(),
                    IROperator::PushScope { bytes } => todo!(),
                    IROperator::PopScope { bytes } => todo!(),
                    IROperator::Nop => todo!(),
                }
            }
        }
    }

    // Substitute labels with actual addresses

    // Specify the entry point of the program

    bytecode
}

