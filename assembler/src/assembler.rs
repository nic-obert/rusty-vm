use rusty_vm_lib::assembly::ByteCode;

use crate::generator::generate_bytecode;
use crate::lang::{AsmNode, ENTRY_SECTION_NAME};
use crate::{error, generator, parser, tokenizer};
use crate::module_manager::{AsmUnit, ModuleManager};
use crate::symbol_table::{ExportedSymbols, SymbolTable};

use std::fs;
use std::path::{Path, PathBuf};


/// Evaluates special compile time assembly symbols and returns the evaluated line
/// 
/// Substitutes $ symbols with the current binary address
/// 
/// Does not substitute $ symbols inside strings or character literals
/// 
/// Does not evaluate escape characters inside strings or character literals
// fn evaluate_special_symbols(line: &str, current_binary_address: Address, line_number: usize, unit_path: &Path, local_const_macros: &ConstMacroMap) -> String {

//     enum TextType {
//         Asm,
//         ConstMacro { starts_at: usize },
//         String { starts_at: (usize, usize) },
//         Char {starts_at: (usize, usize) },
//     }

//     let mut evaluated_line = String::with_capacity(line.len());

//     let mut text_type = TextType::Asm;
//     let mut escape_char = false;

//     for (char_index, c) in line.chars().enumerate() {

//         match text_type {

//             TextType::Asm => {
//                 // Has to be handled later
//             },

//             TextType::String {..} => {

//                 evaluated_line.push(c);

//                 if escape_char {
//                     escape_char = false;
//                 } else if c == '"' {
//                     text_type = TextType::Asm;
//                 } else if c == '\\' {
//                     escape_char = true;
//                 }

//                 continue;
//             },

//             TextType::Char {..} => {

//                 evaluated_line.push(c);

//                 if escape_char {
//                     escape_char = false;
//                 } else if c == '\'' {
//                     text_type = TextType::Asm;
//                 } else if c == '\\' {
//                     escape_char = true;
//                 }
                
//                 continue;
//             },

//             TextType::ConstMacro { starts_at } => {

//                 if is_identifier_char(c, starts_at + 1 == char_index) {
//                     continue;
//                 }

//                 // The const macro name is finished
//                 let const_macro_name = &line[starts_at + 1..char_index];

//                 if let Some(const_macro) = local_const_macros.get(const_macro_name) {
//                     // See if the macro can be replaced now
//                     evaluated_line.push_str(const_macro.replace.as_str());
//                 } else {
//                     // Maybe the macro will be replaced later
//                     evaluated_line.push_str(format!("={}", const_macro_name).as_str());
//                 }

//                 text_type = TextType::Asm;
//             },

//         }

//         // TextType::Asm

//         match c {
//             '$' => {
//                 evaluated_line.push_str(format!("{}", current_binary_address).as_str());
//             },
//             '"' => {
//                 evaluated_line.push('"');
//                 text_type = TextType::String { starts_at: (line_number, char_index) };
//             },
//             '\'' => {
//                 evaluated_line.push('\'');
//                 text_type = TextType::Char { starts_at: (line_number, char_index) };
//             },
//             '#' => {
//                 // Skip comments
//                 break;
//             }
//             '=' => {
//                 text_type = TextType::ConstMacro { starts_at: char_index };
//             },
//             '&' => {
//                 let symbol = get_unique_symbol_name();
//                 evaluated_line.push_str(symbol.as_str());
//             }
//             _ => {
//                 evaluated_line.push(c);
//             }
//         }
//     }

//     // Check for unclosed delimited literals
//     match text_type {
//         TextType::Asm => {
//             // If the text type is Asm, then there were no unclosed strings or character literals
//         },
//         TextType::Char { starts_at } => {
//             error::unclosed_char_literal(unit_path, starts_at.0, starts_at.1, line);
//         },
//         TextType::String { starts_at } => {
//             error::unclosed_string_literal(unit_path, starts_at.0, starts_at.1, line);
//         }
//         TextType::ConstMacro { starts_at } => {

//             // The const macro name is finished
//             let const_macro_name = &line[starts_at + 1..];

//             if let Some(const_macro) = local_const_macros.get(const_macro_name) {
//                 // See if the macro can be replaced now
//                 evaluated_line.push_str(const_macro.replace.as_str());
//             } else {
//                 // Maybe the macro will be replaced later
//                 evaluated_line.push_str(format!("={}", const_macro_name).as_str());
//             }
//         }
//     }

//     evaluated_line
// }


/// Assemble recursively an assembly unit and its dependencies
// fn assemble_unit(asm_unit: AssemblyUnit, verbose: bool, program_info: &mut ProgramInfo) {

    // Check if the assembly unit has already been included
    // if let Some((exported_labels, exported_macros, exported_const_macros)) = program_info.included_units.get(asm_unit.path) {
    //     if verbose {
    //         println!("Unit already included: {}", asm_unit.path.display());
    //         println!("Exported labels: {:?}\n", exported_labels.keys());
    //         println!("Exported macros: {:?}\n", exported_macros.keys());
    //         println!("Exported const macros: {:?}\n", exported_const_macros.keys());
    //     }

    //     program_info.included_units.insert(asm_unit.path.to_path_buf(), (exported_labels.clone(), exported_macros.clone(), exported_const_macros.clone()));
    //     return;
    // }
    // // Insert a temporary entry in the included units map to avoid infinite recursive unit inclusion
    // program_info.included_units.insert(asm_unit.path.to_path_buf(), (LabelMap::new(), MacroMap::new(), ConstMacroMap::new()));


    // if verbose {
    //     if asm_unit.is_main_unit {
    //         println!("\nMain assembly unit: {} ({})\n", asm_unit.path.file_name().unwrap().to_string_lossy(), asm_unit.path.display());
    //     } else {
    //         println!("\nAssembly unit: {} ({})\n", asm_unit.path.file_name().unwrap().to_string_lossy(), asm_unit.path.display());
    //     }
    // }

    // let mut label_info = LabelInfo::new();

    // let mut macro_info = MacroInfo::new();

    // let mut section_info = SectionInfo::new();

    // for (i, line) in asm_unit.assembly.iter().enumerate() {
    //     let line_number = i + 1;

    //     if verbose {
    //         println!("Line {: >4}, Pos: {: >5} | {}", line_number, program_info.byte_code.len(), line);
    //     }

    //     // Evaluate the compile-time special symbols
    //     // let evaluated_line = evaluate_special_symbols(line, program_info.byte_code.len(), line_number, asm_unit.path, &macro_info.local_const_macros);

    //     // Remove redundant whitespaces
    //     // let trimmed_line = evaluated_line.trim();

    //     if trimmed_line.is_empty() {
    //         // The line is either empty, skip it
    //         continue;
    //     }

    //     let last_byte_code_address: Address = program_info.byte_code.len();

    //     // parse_line(trimmed_line, &mut macro_info, &asm_unit, &evaluated_line, line_number, program_info, &mut label_info, &mut section_info, verbose);
        
    //     if verbose && last_byte_code_address != program_info.byte_code.len() {
    //         println!(" => {:?}", &program_info.byte_code[last_byte_code_address..]);
    //     }
        
    // }

    // if matches!(section_info.current_section, ProgramSection::MacroDefinition) {
    //     let def = macro_info.current_macro.as_ref().unwrap_or_else(
    //         || panic!("Internal assembler error: current macro is None inside macro definition. This is a bug.")
    //     );
    //     error::unclosed_macro_definition(asm_unit.path, &def.name, def.line_number, asm_unit.assembly[def.line_number - 1].as_str());
    // }

    // // After tokenization and conversion to bytecode, substitute the labels with their real address

    // for reference in label_info.label_references {

    //     let label = label_info.local_labels.get(&reference.name).unwrap_or_else(
    //         || error::undeclared_label(asm_unit.path, &reference.name, &label_info.local_labels, reference.line_number, &asm_unit.assembly[reference.line_number - 1])
    //     );

    //     // Substitute the label with the real address (little endian)
    //     program_info.byte_code[reference.location..reference.location + ADDRESS_SIZE].copy_from_slice(&label.address.to_le_bytes());

    // }

    // if verbose {
    //     if asm_unit.is_main_unit {
    //         println!("\nEnd of main assembly unit {} ({})\n", asm_unit.path.file_name().unwrap().to_string_lossy(), asm_unit.path.display());
    //     } else {
    //         println!("\nEnd of assembly unit {} ({})\n", asm_unit.path.file_name().unwrap().to_string_lossy(), asm_unit.path.display());
    //         println!("Exported labels: {:?}\n", label_info.export_labels.keys());
    //         println!("Exported macros: {:?}\n", macro_info.export_macros.keys());
    //         println!("Exported const macros: {:?}\n", macro_info.export_const_macros.keys());
    //     }
    // }

    // // Add the assembly unit to the included units
    // program_info.included_units.insert(asm_unit.path.to_path_buf(), (label_info.export_labels, macro_info.export_macros, macro_info.export_const_macros));

// }


// #[allow(clippy::too_many_arguments)]
// fn parse_line(trimmed_line: &str, macro_info: &mut MacroInfo, asm_unit: &AssemblyUnit, line: &str, line_number: usize, program_info: &mut ProgramInfo, label_info: &mut LabelInfo, section_info: &mut SectionInfo, verbose: bool) {

//     if let Some(mut section_name) = trimmed_line.strip_prefix('.') {
//         // This line specifies a program section
//         section_name = section_name.strip_suffix(':').unwrap_or_else(
//             || error::invalid_section_declaration(asm_unit.path, section_name, line_number, line, "Assembly sections must end with a colon.")
//         );

//         match section_name {

//             "bss" => {
//                 // Check for duplicate sections
//                 if section_info.has_bss_section {
//                     error::invalid_section_declaration(asm_unit.path, section_name, line_number, line, "An assembly unit can only have one bss section.")
//                 }
//                 section_info.current_section = ProgramSection::Bss;
//                 section_info.has_bss_section = true;
//             }

//             "include" => {
//                 // Check for duplicate sections
//                 if section_info.has_include_section {
//                     error::invalid_section_declaration(asm_unit.path, section_name, line_number, line, "An assembly unit can only have one include section.")
//                 }
//                 section_info.current_section = ProgramSection::Include;
//                 section_info.has_include_section = true;
//             },

//             "data" => {
//                 // Check for duplicate sections
//                 if section_info.has_data_section {
//                     error::invalid_section_declaration(asm_unit.path, section_name, line_number, line, "An assembly unit can only have one data section.")
//                 }
//                 section_info.current_section = ProgramSection::Data;
//                 section_info.has_data_section = true;
//             },

//             "text" => {
//                 // Check for duplicate sections
//                 if section_info.has_text_section {
//                     error::invalid_section_declaration(asm_unit.path, section_name, line_number, line, "An assembly unit can only have one text section.")
//                 }
//                 section_info.current_section = ProgramSection::Text;
//                 section_info.has_text_section = true;
//             },

//             _ => error::invalid_section_declaration(asm_unit.path, section_name, line_number, line, format!("Unknown assembly section name: \"{}\"", section_name).as_str())
//         }

//         // There's nothing after the section declaration, so return
//         return;
//     }

//     // Handle the assembly code depending on the current section
//     match section_info.current_section {

//         ProgramSection::MacroDefinition => {
            
//             let macro_definition = macro_info.current_macro.as_mut().unwrap_or_else(
//                 || panic!("Internal assembler error: current macro is None inside macro definition. This is a bug.")
//             );
            
//             // Check for macro definition end
//             if let Some(mline) = trimmed_line.strip_prefix("%endmacro") {

//                 if !mline.is_empty() {
//                     error::invalid_macro_declaration(asm_unit.path, macro_definition.name.as_str(), line_number, line, "Macro definition end `%endmacro` must be on its own line");
//                 }

//                 if let Some(def) = macro_info.local_macros.insert(macro_definition.name.clone(), macro_definition.clone()) {
//                     error::macro_redeclaration(asm_unit.path, macro_definition.name.as_str(), def.line_number, &def.unit_path, line_number, line);
//                 }

//                 if macro_definition.to_export {
//                     macro_info.export_macros.insert(macro_definition.name.clone(), macro_info.current_macro.take().unwrap());
//                 }

//                 // Macros can only be defined inside text sections, so the section is assumed to be text
//                 section_info.current_section = ProgramSection::Text;

//             } else {
//                 // The line is part of the macro body
//                 macro_definition.body.push(trimmed_line.to_string());
//             }

//         },

//         ProgramSection::Include => {

//             // Check if the include is to re-export (prefix with @@)
//             let (include_unit_raw, to_export) = {
//                 if let Some(include_unit_raw) = trimmed_line.strip_prefix("@@") {
//                     (include_unit_raw.trim(), true)
//                 } else {
//                     (trimmed_line, false)
//                 }
//             };

//             let (include_path, include_asm) = match load_asm_unit(include_unit_raw, asm_unit.path) {
//                 Ok(x) => x,
//                 Err(error) => error::include_error(asm_unit.path, &error, include_unit_raw, line_number, line)
//             };

//             let new_asm_unit = AssemblyUnit::new(&include_path, include_asm, false);

//             // Assemble the included assembly unit
//             assemble_unit(new_asm_unit, verbose, program_info);
                
//             let (exported_labels, exported_macros, exported_const_macros): &(LabelMap, MacroMap, ConstMacroMap) = program_info.included_units.get(&include_path).unwrap_or_else(
//                 || panic!("Internal assembler error: included unit not found in included units map. This is a bug.")
//             );

//             if to_export {
//                 label_info.export_labels.extend(exported_labels.clone());
//                 macro_info.export_macros.extend(exported_macros.clone());
//                 macro_info.export_const_macros.extend(exported_const_macros.clone());
//             }

//             label_info.local_labels.extend(exported_labels.clone());
//             macro_info.local_macros.extend(exported_macros.clone());
//             macro_info.local_const_macros.extend(exported_const_macros.clone());

//         },

//         ProgramSection::Data => {

//             // Parse the static data declaration

//             // Check if the data label has to be exported (double consecutive @)
//             let (to_export, trimmed_line) = {
//                 if let Some(trimmed_line) = trimmed_line.strip_prefix("@@") {
//                     // Trim the line again to remove eventual extra spaces
//                     (true, trimmed_line.trim_start())
//                 } else {
//                     (false, trimmed_line)
//                 }
//             };

//             // Extract the label name
//             let (label, other) = trimmed_line.split_once(char::is_whitespace).unwrap_or_else(
//                 || error::invalid_data_declaration(asm_unit.path, line_number, line, "Static data declarations must have a label")
//             );

//             // Check if the label is a reserved keyword
//             if is_reserved_name(label) {
//                 error::invalid_label_name(asm_unit.path, label, line_number, line, format!("\"{}\" is a reserved name.", label).as_str());
//             }

//             if other.trim_start().is_empty() {
//                 error::invalid_data_declaration(asm_unit.path, line_number, line, "Static data declarations must have a type");
//             }

//             let (data_type_name, other) = other.split_once(char::is_whitespace).unwrap_or((other, ""));

//             let data_type = DataType::from_name(data_type_name).unwrap_or_else(
//                 || error::invalid_data_declaration(asm_unit.path, line_number, line, format!("Unknown data type \"{}\"", data_type_name).as_str())
//             );

//             // The data string is everything following the data type
//             let data_string = other.trim_start();

//             // Encode the string data into byte code
//             let encoded_data: ByteCode = data_type.encode(data_string, line_number, line, asm_unit.path);

//             let label_declaration = LabelDeclaration::new(program_info.byte_code.len(), asm_unit.path.to_path_buf());

//             // Add the data name and its address in the binary to the data map
//             label_info.local_labels.insert(label.to_string(), label_declaration.clone());

//             if to_export {
//                 label_info.export_labels.insert(label.to_string(), label_declaration);
//             }

//             // Add the data to the byte code
//             program_info.byte_code.extend(encoded_data);

//         },

//         ProgramSection::Bss => {

//             // Check if the bss label has to be exported (double consecutive @)
//             let (to_export, trimmed_line) = {
//                 if let Some(trimmed_line) = trimmed_line.strip_prefix("@@") {
//                     // Trim the line again to remove eventual extra spaces
//                     (true, trimmed_line.trim_start())
//                 } else {
//                     (false, trimmed_line)
//                 }
//             };

//             // Extract the label name
//             let (label, other) = trimmed_line.split_once(char::is_whitespace).unwrap_or_else(
//                 || error::invalid_bss_declaration(asm_unit.path, line_number, line, "Static BSS declarations must have a label")
//             );

//             // Check if the label is a reserved keyword
//             if is_reserved_name(label) {
//                 error::invalid_label_name(asm_unit.path, label, line_number, line, format!("\"{}\" is a reserved name.", label).as_str());
//             }

//             if other.trim_start().is_empty() {
//                 error::invalid_bss_declaration(asm_unit.path, line_number, line, "Static BSS declarations must have a type");
//             }

//             let (data_type_name, other) = other.split_once(char::is_whitespace).unwrap_or((other, ""));

//             let data_type = DataType::from_name(data_type_name).unwrap_or_else(
//                 || error::invalid_bss_declaration(asm_unit.path, line_number, line, format!("Unknown data type \"{}\"", data_type_name).as_str())
//             );

//             let data_size = data_type.size().unwrap_or_else(
//                 || error::invalid_bss_declaration(asm_unit.path, line_number, line, format!("Static size for {} is unknown", data_type_name).as_str())
//             );

//             if !other.trim_start().is_empty() {
//                 error::invalid_bss_declaration(asm_unit.path, line_number, line, "Static BSS declarations cannot have anything after the data type");
//             }

//             let label_declaration = LabelDeclaration::new(program_info.byte_code.len(), asm_unit.path.to_path_buf());

//             // Add the data name and its address in the binary to the data map
//             label_info.local_labels.insert(label.to_string(), label_declaration.clone());

//             if to_export {
//                 label_info.export_labels.insert(label.to_string(), label_declaration);
//             }

//             // Add a data placeholder to the byte code
//             // Doesn't take advantage of the .bss section optimization, though
//             program_info.byte_code.extend(vec![0; data_size]);

//         },

//         ProgramSection::Text => {

//             // Check for label declarations
//             if let Some(label) = trimmed_line.strip_prefix('@') {

//                 // Check if the label is to be exported (double consecutive @)
//                 let (label, to_export): (&str, bool) = {
//                     if let Some(label) = label.strip_prefix('@') {
//                         (label.trim(), true)
//                     } else {
//                         (label.trim(), false)
//                     }
//                 };
                
//                 if !is_identifier_name(label) {
//                     error::invalid_label_name(asm_unit.path, label, line_number, line, "Label names can only contain alphabetic characters, numbers (except for the first character), and underscores.");
//                 }

//                 if is_reserved_name(label) {
//                     error::invalid_label_name(asm_unit.path, label, line_number, line, format!("\"{}\" is a reserved name.", label).as_str());
//                 }

//                 let label_declaration = LabelDeclaration::new(program_info.byte_code.len(), asm_unit.path.to_path_buf());

//                 if asm_unit.is_main_unit && label == "start" || to_export {
//                     label_info.export_labels.insert(label.to_string(), label_declaration.clone());
//                 }

//                 if label_info.local_labels.insert(label.to_string(), label_declaration).is_some() {
//                     error::label_redeclaration(asm_unit.path, label, line_number, line);
//                 }

//                 return;
//             }


//             // Check for macro declaration
//             if let Some(macro_declaration) = trimmed_line.strip_prefix('%') {

//                 // Check if the macro is to be exported (double consecutive %)
//                 let (macro_declaration, to_export): (&str, bool) = {
//                     if let Some(macro_declaration) = macro_declaration.strip_prefix('%') {
//                         (macro_declaration.trim_start(), true)
//                     } else {
//                         (macro_declaration.trim_start(), false)
//                     }
//                 };

//                 // Check if this is a const macro declaration
//                 let (macro_declaration, is_const_macro): (&str, bool) = {
//                     if let Some(macro_declaration) = macro_declaration.strip_prefix('-') {
//                         (macro_declaration.trim_start(), true)
//                     } else {
//                         (macro_declaration.trim_start(), false)
//                     }
//                 };

//                 let (macro_name, post) = macro_declaration.split_once(
//                     |c: char| c.is_whitespace() || c == ':'
//                 ).unwrap_or_else(
//                     || error::invalid_macro_declaration(asm_unit.path, "", line_number, line, "Macro declarations must have a name")
//                 );

//                 if !is_identifier_name(macro_name) {
//                     error::invalid_macro_declaration(asm_unit.path, macro_name, line_number, line, "Macro names can only contain alphabetic characters, numbers (except for the first character), and underscores.");
//                 }

//                 if is_reserved_name(macro_name) {
//                     error::invalid_macro_declaration(asm_unit.path, macro_name, line_number, line, format!("\"{}\" is a reserved name.", macro_name).as_str());
//                 }

//                 // Constant macros don't accept arguments
//                 if is_const_macro {

//                     let replace = post.trim_start();

//                     if replace.is_empty() {
//                         error::invalid_macro_declaration(asm_unit.path, macro_name, line_number, line, "Constant macros must have a replacement text value");
//                     }

//                     let macro_declaration = ConstMacroDefinition::new(
//                         macro_name.to_string(), 
//                         replace.to_string(), 
//                         asm_unit.path.to_path_buf(), 
//                         line_number
//                     );
                    
//                     // Const macros can be redeclared
//                     macro_info.local_const_macros.insert(macro_name.to_string(), macro_declaration.clone());

//                     if to_export {
//                         macro_info.export_const_macros.insert(macro_name.to_string(), macro_declaration);
//                     }

//                     return;
//                 }

//                 let post = post.trim();
//                 let mut macro_args = Vec::new();
//                 if let Some(post) = post.strip_prefix(':') {
//                     // The macro declaration is finished

//                     if !post.is_empty() {
//                         error::invalid_macro_declaration(asm_unit.path, macro_name, line_number, line, "Cannot have anything after the colon in a macro declaration");
//                     }

//                 } else {
//                     // The macro declaration has arguments, parse them

//                     macro_args.extend(
//                         post.split(|c: char| c.is_whitespace() || c == ':')
//                         .map(|s| s.trim())
//                         .filter(|s| !s.is_empty())
//                         .map(|arg_name| {

//                             if !is_identifier_name(arg_name) {
//                                 error::invalid_macro_declaration(asm_unit.path, macro_name, line_number, line, format!("Macro argument names can only contain alphabetic characters, numbers (except for the first character), and underscores.\nInvalid argument name: \"{}\"", arg_name).as_str());
//                             }

//                             if is_reserved_name(arg_name) {
//                                 error::invalid_macro_declaration(asm_unit.path, macro_name, line_number, line, format!("\"{}\" is a reserved name.", arg_name).as_str());
//                             }

//                             arg_name.to_string()
//                         })
//                     );                     

//                 }

//                 macro_info.current_macro = Some(MacroDefinition::new(
//                     macro_name.to_string(),
//                     macro_args,
//                     AssemblyCode::new(),
//                     asm_unit.path.to_path_buf(),
//                     line_number,
//                     to_export
//                 ));

//                 section_info.current_section = ProgramSection::MacroDefinition;

//                 return;
//             }


//             // Check for macro call (starts with !)
//             if let Some(macro_call) = trimmed_line.strip_prefix('!').map(|s| s.trim()) {

//                 let (macro_name, post) = macro_call.split_once(char::is_whitespace).unwrap_or((macro_call, ""));

//                 if macro_name.is_empty() {
//                     error::invalid_macro_call(asm_unit.path, line_number, line, "Macro calls must have a name");
//                 }

//                 // Get the macro definition
//                 let def = macro_info.local_macros.get(macro_name).unwrap_or_else(
//                     || error::undeclared_macro(asm_unit.path, macro_name, &macro_info.local_macros, line_number, line)
//                 );

//                 // Get the arguments of the macro call
//                 let macro_args: Vec<&str> = MacroArgIterator::new(post).collect();
                
//                 if macro_args.len() != def.args.len() {
//                     error::invalid_macro_call(asm_unit.path, line_number, line, format!("Macro \"{}\" ({}, {}) expects {} arguments, but {} were given.", macro_name, def.unit_path.display(), def.line_number, def.args.len(), macro_args.len()).as_str());
//                 }

//                 // Replace the macro argument placeholders in the macro body with the macro call arguments
//                 let mut mlines: Vec<String> = def.body.clone();
//                 for mline in mlines.iter_mut() {
                
//                     // Not very efficient, but works
//                     for (i, arg) in def.args.iter().enumerate() {
//                         *mline = mline.replace(format!("{{{}}}", arg).as_str(), macro_args[i]);
//                     }

//                 }

//                 // Parse the macro body
//                 for mline in mlines {

//                     if verbose {
//                         println!("Macro {: >3}, Pos: {: >5} | {}", line_number, program_info.byte_code.len(), mline);
//                     }

//                     // Evaluate the compile-time special symbols that were not evaluated before
//                     let mline = evaluate_special_symbols(mline.as_str(), program_info.byte_code.len(), line_number, asm_unit.path, &macro_info.local_const_macros);

//                     parse_line(&mline, macro_info, asm_unit, line, line_number, program_info, label_info, section_info, verbose);

//                 }

//                 return;
//             }


//             // The line doesn't contain label declarations, macros, or special operators, so it's an instruction
//             assemble_instruction(asm_unit, trimmed_line, line, line_number, &mut program_info.byte_code, &mut label_info.label_references);

//         },

//         // Code cannot be put outside of a program section
//         ProgramSection::None => error::out_of_section(asm_unit.path, line_number, line)

//     }

// }


// #[inline(always)]
// fn handle_pseudo_instruction(pi: PseudoInstruction, asm_unit: &AssemblyUnit, args: &str, line: &str, line_number: usize, byte_code: &mut ByteCode) {

//     match pi {

//         PseudoInstruction::DefineData => {

//             let (data_type_name, other) = args.split_once(char::is_whitespace).unwrap_or_else(
//                 || error::invalid_data_declaration(asm_unit.path, line_number, line, "Static data declarations must have a type")
//             );

//             let data_type = DataType::from_name(data_type_name).unwrap_or_else(
//                 || error::invalid_data_declaration(asm_unit.path, line_number, line, format!("Unknown data type \"{}\"", data_type_name).as_str())
//             );

//             // The data string is everything following the data type
//             let data_string = other.trim();

//             // Encode the string data into byte code
//             let encoded_data: ByteCode = data_type.encode(data_string, line_number, line, asm_unit.path);
            
//             // Add the data to the byte code
//             byte_code.extend(encoded_data);
//         },

//     }

// }


/// Assemble an instruction into byte code
// fn assemble_instruction(asm_unit: &AssemblyUnit, trimmed_line: &str, line: &str, line_number: usize, byte_code: &mut ByteCode, label_reference_registry: &mut LabelReferenceRegistry) {

//     // Split the operator from its arguments
//     let (operator_name, raw_tokens): (&str, &str) = trimmed_line.split_once(char::is_whitespace).unwrap_or((
//         trimmed_line, ""
//     ));


//     // Check for pseudo instructions
//     if let Some(pi) = get_pseudo_instruction(operator_name) {
//         handle_pseudo_instruction(pi, asm_unit, raw_tokens.trim_start(), line, line_number, byte_code);
//         return;
//     }

//     // The instruction is a regular instruction

//     // let operands = tokenize_operands(raw_tokens, line_number, line, asm_unit.path);
    
//     // let arg_table = get_arguments_table(operator_name).unwrap_or_else(
//     //     || error::invalid_instruction_name(asm_unit.path, operator_name, line_number, line)
//     // );

//     // let operation = arg_table.get_operation(operator_name, &operands, asm_unit.path, line_number, line);

//     // // Add the instruction code to the byte code
//     // byte_code.push(operation.instruction as u8);

//     // // Add the operands to the byte code
//     // let operand_bytes = generate_operand_bytecode(operation.instruction, operands, operation.handled_size, label_reference_registry, byte_code.len(), line_number, asm_unit.path, line);
    
//     // if operand_bytes.len() != operation.total_arg_size as usize {
//     //     panic!("The generated operand byte code size {} for instruction \"{}\" does not match the expected size {}. This is a bug.", operand_bytes.len(), operation.instruction, operation.total_arg_size);
//     // }

//     // byte_code.extend(operand_bytes);

// }


/// Assembles the assembly code into byte code
// pub fn assemble(assembly: AssemblyCode, verbose: bool, unit_path: &Path, just_check: bool) -> ByteCode {

//     let mut program_info = ProgramInfo::new();

//     let asm_unit = AssemblyUnit::new(unit_path, assembly, true);

//     // Assemble recursively the main assembly unit and its dependencies
//     assemble_unit(asm_unit, verbose, &mut program_info);

//     if just_check {
//         println!("✅ No errors found.");
//         std::process::exit(0);
//     }

//     // Append the exit instruction to the end of the binary
//     program_info.byte_code.push(ByteCodes::EXIT as u8);

//     // Append the address of the program start to the end of the binary

//     let (label_map, _macro_map, _const_macro_map) = program_info.included_units.get(unit_path).unwrap_or_else(
//         || panic!("Internal assembler error: main assembly unit not found in included units map. This is a bug.")
//     );

//     let program_start = label_map.get("start").unwrap_or_else(
//         || error::undeclared_label(unit_path, "start", label_map, 0, "The program must have a start label.")
//     );

//     program_info.byte_code.extend(program_start.address.to_le_bytes());

//     if verbose {
//         println!("Byte code size is {} bytes", program_info.byte_code.len());
//         println!("Start address is {}", program_start.address);
//     }

//     program_info.byte_code
// }


// struct MacroArgIterator<'a> {

//     args: &'a str,

// }


// impl MacroArgIterator<'_> {

//     fn new(args: &str) -> MacroArgIterator<'_> {
//         MacroArgIterator {
//             args,
//         }
//     }

// }


// impl<'a> Iterator for MacroArgIterator<'a> {

//     type Item = &'a str;

//     fn next(&mut self) -> Option<Self::Item> {
        
//         if self.args.is_empty() {
//             return None;
//         }

//         enum TextType {
//             String,
//             Char,
//             Array,
//             None,
//         }
        
//         let mut text_type = TextType::None;
//         let mut string_escape = false;
//         let mut array_depth: usize = 0;

//         for (index, c) in self.args.chars().enumerate() {

//             match text_type {

//                 TextType::Array => match c {

//                     '[' => array_depth += 1,

//                     ']' => {
//                         array_depth -= 1;
//                         if array_depth == 0 {
//                             text_type = TextType::None;
//                         }
//                     },

//                     _ => {}

//                 },

//                 TextType::None => match c {

//                     '[' => {
//                         text_type = TextType::Array;
//                         array_depth = 1;
//                     },

//                     ' ' | '\t' 
//                     => {
//                         let arg = self.args[..index].trim();
//                         if arg.is_empty() {
//                             return None;
//                         }

//                         self.args = &self.args[index + 1..];

//                         return Some(arg);
//                     },
                    
//                     '\'' => {
//                         text_type = TextType::Char;
//                     },

//                     '"' => {
//                         text_type = TextType::String;
//                     },

//                     _ => {}
                    
//                 },

//                 TextType::String => {
                        
//                     if string_escape {
//                         string_escape = false;

//                     } else if c == '"' {
//                         text_type = TextType::None;

//                     } else if c == '\\' {
//                         string_escape = true;
//                     }

//                 },

//                 TextType::Char => {

//                     if string_escape {
//                         string_escape = false;

//                     } else if c == '\'' {
//                         text_type = TextType::None;

//                     } else if c == '\\' {
//                         string_escape = true;
//                     }

//                 },
//             }
//         }

//         let arg = self.args.trim();
//         if arg.is_empty() {
//             return None;
//         }

//         self.args = "";

//         Some(arg)
//     }

// }


/// Load the assembly if it isn't already loaded
pub fn load_unit_asm<'a>(caller_directory: Option<&Path>, unit_path: &'a Path, symbol_table: &SymbolTable<'a>, module_manager: &'a ModuleManager<'a>, bytecode: &mut ByteCode) -> Option<Box<[AsmNode<'a>]>> {

    // Shadow the previous `unit_path` to avoid confusion with the variables
    let unit_path = module_manager.resolve_include_path(caller_directory, unit_path)
        .unwrap_or_else(|err| 
            error::io_error(err, format!("Failed to canonicalize path \"{}\"", unit_path.display()).as_str()));

    if module_manager.is_loaded(&unit_path) {
        // The module was already imported, there's no need to re-import it
        return None;
    }

    let raw_source = fs::read_to_string(unit_path)
        .unwrap_or_else(|err| error::io_error(err, format!("Could not load file \"{}\"", unit_path.display()).as_str()));

    let asm_unit = module_manager.add_unit(unit_path, AsmUnit::new(raw_source));

    let token_lines = tokenizer::tokenize(&asm_unit.lines, unit_path);

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


pub fn assemble_included_unit<'a>(caller_directory: &Path, unit_path: &'a Path, module_manager: &'a ModuleManager<'a>, bytecode: &mut ByteCode) -> ExportedSymbols<'a> {

    let symbol_table = SymbolTable::new();

    let asm = if let Some(asm) = load_unit_asm(Some(caller_directory), unit_path, &symbol_table, &module_manager, bytecode) {
        asm
    } else {
        return Default::default();
    };

    generator::generate_bytecode(asm, &symbol_table, module_manager, bytecode);

    symbol_table.export_symbols()
}


/// Recursively assemble the given ASM unit and all the included units
pub fn assemble_all(caller_directory: &Path, unit_path: &Path, include_paths: Vec<PathBuf>) -> ByteCode {
    
    let module_manager = ModuleManager::new(include_paths);

    let symbol_table = SymbolTable::new();
    
    let mut bytecode = ByteCode::new();

    let asm = load_unit_asm(Some(caller_directory), unit_path, &symbol_table, &module_manager, &mut bytecode)
        .expect("Main ASM unit should not be already loaded");

    generate_bytecode(asm, &symbol_table, &module_manager, &mut bytecode);

    if let Some(program_start) = symbol_table.get_resolved_label(ENTRY_SECTION_NAME) {
        bytecode.extend(program_start.to_le_bytes());
    } else {
        error::missing_entry_point(unit_path);
    }

    bytecode
}

