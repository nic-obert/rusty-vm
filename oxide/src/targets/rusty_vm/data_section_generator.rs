use rusty_vm_lib::assembly::ByteCode;

use crate::symbol_table::SymbolTable;

use super::StaticAddressMap;


/// Generate static data section, equivalent to .data section in assembly
pub fn generate_static_data_section(symbol_table: &SymbolTable, static_address_map: &mut StaticAddressMap, bytecode: &mut ByteCode) {

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
