use std::collections::HashMap;
use std::mem;

use num_traits::ToBytes;

use rusty_vm_lib::assembly::ByteCode;
use rusty_vm_lib::vm::Address;

use crate::irc::{Label, LabelID, IRIDGenerator};
use crate::symbol_table::{StaticID, SymbolTable};
use crate::flow_analyzer::FunctionGraph;

use super::data_section_generator::generate_static_data_section;
use super::text_section_generator::generate_text_section;


pub(super) type LabelAddressMap = HashMap<LabelID, Address>;
pub(super) type StaticAddressMap = HashMap<StaticID, Address>;


pub(super) struct LabelGenerator {
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
