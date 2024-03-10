use std::collections::HashMap;

use rusty_vm_lib::assembly::ByteCode;
use rusty_vm_lib::vm::Address;

use crate::irc::LabelID;
use crate::symbol_table::SymbolTable;
use crate::flow_analyzer::FunctionGraph;


type LabelAddressMap = HashMap<LabelID, Address>;


pub fn generate_bytecode(symbol_table: &SymbolTable, function_graphs: Vec<FunctionGraph>) -> ByteCode {
    /*
        Generate a static section for static data
        Generate a text section for the code

    */

    todo!()
}

