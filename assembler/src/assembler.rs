use rusty_vm_lib::assembly::ByteCode;
use rusty_vm_lib::byte_code::ByteCodes;

use crate::generator::generate_bytecode;
use crate::lang::{AsmNode, ENTRY_SECTION_NAME};
use crate::{error, generator, parser, tokenizer};
use crate::module_manager::{AsmUnit, ModuleManager, UnitPath};
use crate::symbol_table::{ExportedSymbols, SymbolTable};

use std::fs;
use std::path::{Path, PathBuf};


/// Load the assembly if it isn't already loaded
pub fn load_unit_asm<'a>(unit_path: UnitPath<'a>, symbol_table: &SymbolTable<'a>, module_manager: &'a ModuleManager<'a>, bytecode: &mut ByteCode) -> Option<Box<[AsmNode<'a>]>> {

    if module_manager.is_loaded(unit_path) {
        // The module was already imported, there's no need to re-import it
        return None;
    }

    let raw_source = fs::read_to_string(unit_path.as_path())
        .unwrap_or_else(|err| error::io_error(err, format!("Could not load file \"{}\"", unit_path).as_str()));

    let asm_unit = module_manager.add_unit(unit_path, AsmUnit::new(raw_source));

    let token_lines = tokenizer::tokenize(asm_unit.lines(), unit_path);

    // println!("\nTokens:\n");
    // for line in &token_lines {
    //     for token in line.iter() {
    //         println!("{}", token);
    //     }
    // }

    let asm = parser::parse(token_lines, symbol_table, module_manager, bytecode);

    // println!("\n\nNodes:\n");
    // for node in &asm {
    //     println!("{:?}", node);
    // }

    Some(asm)
}


pub fn assemble_included_unit<'a>(unit_path: UnitPath<'a>, module_manager: &'a ModuleManager<'a>, bytecode: &mut ByteCode) -> &'a ExportedSymbols<'a> {

    let symbol_table = SymbolTable::new();

    let asm = if let Some(asm) = load_unit_asm(unit_path, &symbol_table, &module_manager, bytecode) {
        asm
    } else {
        return module_manager.get_unit_exports(unit_path);
    };

    generator::generate_bytecode(asm, &symbol_table, module_manager, bytecode);

    let exports = symbol_table.export_symbols();
    module_manager.set_unit_exports(unit_path, exports)
}


/// Recursively assemble the given ASM unit and all the included units
pub fn assemble_all(caller_directory: &Path, unit_path: &Path, include_paths: Vec<PathBuf>) -> ByteCode {
    
    let module_manager = ModuleManager::new(include_paths);
    
    // Shadow the previous `unit_path` to avoid confusion with the variables
    let unit_path = module_manager.resolve_include_path(caller_directory, unit_path)
        .unwrap_or_else(|err| 
            error::io_error(err, format!("Failed to resolve path \"{}\"", unit_path.display()).as_str()
        )
    );

    let symbol_table = SymbolTable::new();
    
    let mut bytecode = ByteCode::new();

    let asm = load_unit_asm(unit_path, &symbol_table, &module_manager, &mut bytecode)
        .expect("Main ASM unit should not be already loaded");

    generate_bytecode(asm, &symbol_table, &module_manager, &mut bytecode);

    // Place a failsafe exit instruction at the end of the program in case the programmer forgot to exit.
    // Without an exit instruction, the VM would keep on executing from memory past the code section.
    bytecode.push(ByteCodes::EXIT as u8);

    if let Some(program_start) = symbol_table.get_resolved_label(ENTRY_SECTION_NAME) {
        bytecode.extend(program_start.to_le_bytes());
    } else {
        error::missing_entry_point(unit_path);
    }

    bytecode
}

