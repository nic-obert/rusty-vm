use rust_vm_lib::assembly::{AssemblyCode, ByteCode};
use rust_vm_lib::byte_code::ByteCodes;
use rust_vm_lib::vm::{Address, ADDRESS_SIZE};

use crate::data_types::DataType;
use crate::error;
use crate::tokenizer::{tokenize_operands, is_identifier_name, is_reserved_name};
use crate::argmuments_table::get_arguments_table;
use crate::token_to_byte_code::{get_token_converter, use_converter};
use crate::files;
use crate::configs;

use std::collections::HashMap;
use std::io;
use std::path::{Path, PathBuf};


/// Maps a macro name to its definition
type MacroMap = HashMap<String, MacroDefinition>;


#[derive(Clone, Debug)]
struct MacroDefinition {

    pub name: String,
    pub args: Vec<String>,
    pub body: AssemblyCode,
    pub unit_path: PathBuf,
    pub line_number: usize,
    pub to_export: bool,

}


impl MacroDefinition {

    pub fn new(name: String, args: Vec<String>, body: AssemblyCode, unit_path: PathBuf, line_number: usize, to_export: bool) -> MacroDefinition {
        MacroDefinition {
            name,
            args,
            body,
            unit_path,
            line_number,
            to_export,
        }
    }

}


/// Maps a label name to its address in the binary
pub type LabelMap = HashMap<String, Address>;

pub struct LabelReference {

    pub name: String,
    pub location: Address,
    pub line_number: usize,

}


impl LabelReference {

    pub fn new(name: String, location: Address, line_number: usize) -> LabelReference {
        LabelReference {
            name,
            location,
            line_number,
        }
    }

}


/// Records the references to labels in the assembly code
pub type LabelReferenceRegistry = Vec<LabelReference>;


pub trait AddLabelReference {

    fn add_reference(&mut self, name: String, location: Address, line_number: usize);

}


impl AddLabelReference for LabelReferenceRegistry {

    fn add_reference(&mut self, name: String, location: Address, line_number: usize) {
        self.push(LabelReference::new(name, location, line_number));
    }

}


/// Represents a section in the assembly code
enum ProgramSection {
    Data,
    Text,
    Include,

    /// Not really a program section, but works to tell the assembler that it's parsing a macro.
    MacroDefinition,

    None,
}


/// Try to load an assembly unit
/// 
/// If successful, return the assembly code and the absolute path to the unit
fn load_asm_unit(unit_name: &str, current_unit_path: &Path) -> io::Result<(PathBuf, AssemblyCode)> {

    let unit_path = Path::new(unit_name);

    if unit_path.is_absolute() {
        // The unit path is absolute, try to load it directly
        return Ok((unit_path.to_path_buf(), files::load_assembly(unit_path)?));
    }

    // The unit path is relative

    // Try to load the unit from the standard library
    {
        let unit_path = configs::INCLUDE_LIB_PATH.join(unit_path);
        if let Ok(assembly) = files::load_assembly(&unit_path) {
            return Ok((unit_path.canonicalize().unwrap(), assembly))
        }
    }

    // Finally, try to load the unit from the current assembly unit directory

    let parent_dir = match current_unit_path.parent() {
        Some(parent_dir) => parent_dir,
        None => {
            return Err(
                io::Error::new(
                    io::ErrorKind::Other,
                    format!("Failed to get parent directory of \"{}\"", current_unit_path.display())
                )
            );
        }
    };  

    // Get the absolute path of the include unit
    let unit_path = parent_dir.join(unit_name).canonicalize()?;

    let assembly = files::load_assembly(&unit_path)?;

    Ok((unit_path, assembly))
}


/// Evaluates special compile time assembly symbols and returns the evaluated line
/// Substitutes $ symbols with the current binary address
/// Does not substitute $ symbols inside strings or character literals
/// Does not evaluate escape characters inside strings or character literals
fn evaluate_special_symbols(line: &str, current_binary_address: Address, line_number: usize, unit_path: &Path) -> String {

    enum TextType {
        Asm,
        String { starts_at: (usize, usize) },
        Char {starts_at: (usize, usize) },
    }

    let mut evaluated_line = String::with_capacity(line.len());

    let mut text_type = TextType::Asm;
    let mut escape_char = false;

    for (char_index, c) in line.chars().enumerate() {

        match text_type {

            TextType::Asm => {
                
                match c {
                    '$' => {
                        evaluated_line.push_str(format!("{}", current_binary_address).as_str());
                    },
                    '"' => {
                        evaluated_line.push('"');
                        text_type = TextType::String { starts_at: (line_number, char_index) };
                    },
                    '\'' => {
                        evaluated_line.push('\'');
                        text_type = TextType::Char { starts_at: (line_number, char_index) };
                    },
                    '#' => {
                        // Skip comments
                        break;
                    }
                    _ => {
                        evaluated_line.push(c);
                    }
                }

            },

            TextType::String {..} => {

                evaluated_line.push(c);

                if escape_char {
                    escape_char = false;
                } else if c == '"' {
                    text_type = TextType::Asm;
                } else if c == '\\' {
                    escape_char = true;
                }
            },

            TextType::Char {..} => {

                evaluated_line.push(c);

                if escape_char {
                    escape_char = false;
                } else if c == '\'' {
                    text_type = TextType::Asm;
                } else if c == '\\' {
                    escape_char = true;
                }

            }

        }
    }

    // Check for unclosed delimited literals
    match text_type {
        TextType::Asm => {
            // If the text type is Asm, then there were no unclosed strings or character literals
        },
        TextType::Char { starts_at } => {
            error::unclosed_char_literal(unit_path, starts_at.0, starts_at.1, line);
        },
        TextType::String { starts_at } => {
            error::unclosed_string_literal(unit_path, starts_at.0, starts_at.1, line);
        }
    }

    evaluated_line
}


type ASMUnitMap = HashMap<PathBuf, (LabelMap, MacroMap)>;


struct AssemblyUnit<'a> {

    pub path: &'a Path,
    pub assembly: AssemblyCode,
    pub is_main_unit: bool,

}


impl AssemblyUnit<'_> {

    pub fn new(path: &Path, assembly: AssemblyCode, is_main_unit: bool) -> AssemblyUnit {
        AssemblyUnit {
            path,
            assembly,
            is_main_unit,
        }
    }

}


/// Assemble recursively an assembly unit and its dependencies
fn assemble_unit(asm_unit: AssemblyUnit, verbose: bool, byte_code: &mut ByteCode, external_export_label_declaration_map: &mut LabelMap, external_export_macro_definition_map: &mut MacroMap, included_units: &mut ASMUnitMap) {

    // Check if the assembly unit has already been included
    if let Some((exported_labels, exported_macros)) = included_units.get(asm_unit.path) {
        if verbose {
            println!("Unit already included: {}", asm_unit.path.display());
            println!("Exported labels: {:?}\n", exported_labels);
            println!("Exported macros: {:?}\n", exported_macros);
        }

        // The assembly unit has already been included, do not include it again
        // Export the labels of the already included assembly unit and return
        external_export_label_declaration_map.extend(exported_labels.clone());

        return;
    }
    // Insert a temporary entry in the included units map to avoid infinite recursive unit inclusion
    included_units.insert(asm_unit.path.to_path_buf(), (LabelMap::new(), MacroMap::new()));


    if verbose {
        if asm_unit.is_main_unit {
            println!("\nMain assembly unit: {} ({})\n", asm_unit.path.file_name().unwrap().to_string_lossy(), asm_unit.path.display());
        } else {
            println!("\nAssembly unit: {} ({})\n", asm_unit.path.file_name().unwrap().to_string_lossy(), asm_unit.path.display());
        }
    }

    // It's necessary to create a local export label map to avoid accessing externally declared labels
    let mut export_label_declaration_map = LabelMap::new();

    // It's necessary to create a local export macro map to avoid accessing externally declared macros
    let mut export_macro_declaration_map = MacroMap::new();
    
    // Stores the address in the bytecode of all the local labels
    let mut local_label_declaration_map = LabelMap::new();
    // Stores the references to labels and when they are referenced in the bytecode
    // Used later to substitute the labels with real addresses
    let mut label_reference_registry = LabelReferenceRegistry::new();

    let mut local_macro_definition_map = MacroMap::new();

    let mut current_macro: Option<MacroDefinition> = None;

    let mut current_section = ProgramSection::None;

    let mut has_data_section = false;
    let mut has_text_section = false;
    let mut has_include_section = false;

    for (i, line) in asm_unit.assembly.iter().enumerate() {
        let line_number = i + 1;

        if verbose {
            println!("Line {: >4}, Pos: {: >5} | {}", line_number, byte_code.len(), line);
        }

        // Evaluate the compile-time special symbols
        let evaluated_line = evaluate_special_symbols(line, byte_code.len(), line_number, asm_unit.path);

        // Remove redundant whitespaces
        let trimmed_line = evaluated_line.trim();

        if trimmed_line.is_empty() || trimmed_line.starts_with('#') {
            // The line is either empty or a comment, skip it
            continue;
        }

        let last_byte_code_address: Address = byte_code.len();

        if let Some(mut section_name) = trimmed_line.strip_prefix('.') {
            // This line specifies a program section
            section_name = section_name.strip_suffix(':').unwrap_or_else(
                || error::invalid_section_declaration(asm_unit.path, section_name, line_number, line, "Assembly sections must end with a colon.")
            );

            match section_name {

                "include" => {
                    // Check for duplicate sections
                    if has_include_section {
                        error::invalid_section_declaration(asm_unit.path, section_name, line_number, line, "An assembly unit can only have one include section.")
                    }
                    current_section = ProgramSection::Include;
                    has_include_section = true;

                    continue;
                },

                "data" => {
                    // Check for duplicate sections
                    if has_data_section {
                        error::invalid_section_declaration(asm_unit.path, section_name, line_number, line, "An assembly unit can only have one data section.")
                    }
                    current_section = ProgramSection::Data;
                    has_data_section = true;

                    continue;
                },

                "text" => {
                    // Check for duplicate sections
                    if has_text_section {
                        error::invalid_section_declaration(asm_unit.path, section_name, line_number, line, "An assembly unit can only have one text section.")
                    }
                    current_section = ProgramSection::Text;
                    has_text_section = true;

                    continue;
                },

                _ => error::invalid_section_declaration(asm_unit.path, section_name, line_number, line, format!("Unknown assembly section name: \"{}\"", section_name).as_str())
            }
            
        }

        // Handle the assembly code depending on the current section
        match current_section {

            ProgramSection::MacroDefinition => {
                
                let macro_definition = current_macro.as_mut().unwrap_or_else(
                    || panic!("Internal assembler error: current macro is None inside macro definition. This is a bug.")
                );

                if trimmed_line == format!("%{}", macro_definition.name) {
                    // The macro definition is finished

                    if macro_definition.to_export {
                        export_macro_declaration_map.insert(macro_definition.name.clone(), macro_definition.clone());
                    }

                    local_macro_definition_map.insert(macro_definition.name.clone(), current_macro.take().unwrap());

                    // Macros can only be defined inside text sections, so the section is assumed to be text
                    current_section = ProgramSection::Text;

                    continue;
                }

                macro_definition.body.push(line.trim().to_string());

            },

            ProgramSection::Include => {

                // Check if the include is to re-export (prefix with @@)
                let (include_unit_raw, to_export) = {
                    if let Some(include_unit_raw) = trimmed_line.strip_prefix("@@") {
                        (include_unit_raw.trim(), true)
                    } else {
                        (trimmed_line, false)
                    }
                };

                let (include_path, include_asm) = match load_asm_unit(include_unit_raw, asm_unit.path) {
                    Ok(x) => x,
                    Err(error) => error::include_error(asm_unit.path, &error, include_unit_raw, line_number, line)
                };

                let new_asm_unit = AssemblyUnit::new(&include_path, include_asm, false);

                // Assemble the included assembly unit
                if to_export {
                    let mut labels = LabelMap::new();
                    let mut macros = MacroMap::new();

                    assemble_unit(new_asm_unit, verbose, byte_code, &mut labels, &mut macros, included_units);
                    
                    export_label_declaration_map.extend(labels.clone());
                    local_label_declaration_map.extend(labels);

                    export_macro_declaration_map.extend(macros.clone());
                    local_macro_definition_map.extend(macros);

                } else {
                    assemble_unit(new_asm_unit, verbose, byte_code, &mut local_label_declaration_map, &mut local_macro_definition_map, included_units);
                }

            },

            ProgramSection::Data => {

                // Parse the static data declaration

                // Check if the data label has to be exported (double consecutive @)
                let (to_export, trimmed_line) = {
                    if let Some(trimmed_line) = trimmed_line.strip_prefix("@@") {
                        // Trim the line again to remove eventual extra spaces
                        (true, trimmed_line.trim())
                    } else {
                        (false, trimmed_line)
                    }
                };

                // Extract the label name
                let (label, other) = trimmed_line.split_once(char::is_whitespace).unwrap_or_else(
                    || error::invalid_data_declaration(asm_unit.path, line_number, line, "Static data declarations must have a label")
                );

                // Check if the label is a reserved keyword
                if is_reserved_name(label) {
                    error::invalid_label_name(asm_unit.path, label, line_number, line, format!("\"{}\" is a reserved name.", label).as_str());
                }

                let (data_type_name, other) = other.split_once(char::is_whitespace).unwrap_or_else(
                    || error::invalid_data_declaration(asm_unit.path, line_number, line, "Static data declarations must have a type")
                );

                let data_type = DataType::from_name(data_type_name).unwrap_or_else(
                    || error::invalid_data_declaration(asm_unit.path, line_number, line, format!("Unknown data type \"{}\"", data_type_name).as_str())
                );

                // The data string is everything following the data type
                let data_string = other.trim();

                // Encode the string data into byte code
                let encoded_data: ByteCode = data_type.encode(data_string, line_number, line, asm_unit.path);

                // Add the data name and its address in the binary to the data map
                local_label_declaration_map.insert(label.to_string(), byte_code.len());

                if to_export {
                    export_label_declaration_map.insert(label.to_string(), byte_code.len());
                }

                // Add the data to the byte code
                byte_code.extend(encoded_data);

            },

            ProgramSection::Text => {

                // Check for label declarations
                if let Some(label) = trimmed_line.strip_prefix('@') {

                    // Check if the label is to be exported (double consecutive @)
                    let (label, to_export): (&str, bool) = {
                        if let Some(label) = label.strip_prefix('@') {
                            (label.trim(), true)
                        } else {
                            (label.trim(), false)
                        }
                    };
                    
                    if !is_identifier_name(label) {
                        error::invalid_label_name(asm_unit.path, label, line_number, line, "Label names can only contain alphabetic characters, numbers (except for the first character), and underscores.");
                    }

                    if is_reserved_name(label) {
                        error::invalid_label_name(asm_unit.path, label, line_number, line, format!("\"{}\" is a reserved name.", label).as_str());
                    }

                    if asm_unit.is_main_unit && label == "start" || to_export {
                        export_label_declaration_map.insert(label.to_string(), byte_code.len());
                    }
                    
                    local_label_declaration_map.insert(label.to_string(), byte_code.len());

                    continue;
                }


                // Check for macro declaration
                if let Some(macro_declaration) = trimmed_line.strip_prefix('%') {

                    // Check if the macro is to be exported (double consecutive %)
                    let (macro_declaration, to_export): (&str, bool) = {
                        if let Some(macro_declaration) = macro_declaration.strip_prefix('%') {
                            (macro_declaration.trim(), true)
                        } else {
                            (macro_declaration.trim(), false)
                        }
                    };

                    let (macro_name, post) = macro_declaration.split_once(
                        |c: char| c.is_whitespace() || c == ':'
                    ).unwrap_or_else(
                        || error::invalid_macro_declaration(asm_unit.path, line_number, line, "Macro declarations must have a name")
                    );

                    if !is_identifier_name(macro_name) {
                        error::invalid_macro_declaration(asm_unit.path, line_number, line, "Macro names can only contain alphabetic characters, numbers (except for the first character), and underscores.");
                    }

                    if is_reserved_name(macro_name) {
                        error::invalid_macro_declaration(asm_unit.path, line_number, line, format!("\"{}\" is a reserved name.", macro_name).as_str());
                    }

                    let post = post.trim();
                    let mut macro_args = Vec::new();
                    if let Some(post) = post.strip_prefix(':') {
                        // The macro declaration is finished

                        if !post.is_empty() {
                            error::invalid_macro_declaration(asm_unit.path, line_number, line, "Cannot have anything after the colon in a macro declaration");
                        }

                    } else {
                        // The macro declaration has arguments, parse them

                        macro_args.extend(
                            post.split(|c: char| c.is_whitespace() || c == ':')
                            .map(|s| s.trim())
                            .filter(|s| !s.is_empty())
                            .map(|arg_name| {

                                if !is_identifier_name(arg_name) {
                                    error::invalid_macro_declaration(asm_unit.path, line_number, line, "Macro argument names can only contain alphabetic characters, numbers (except for the first character), and underscores.");
                                }
    
                                if is_reserved_name(arg_name) {
                                    error::invalid_macro_declaration(asm_unit.path, line_number, line, format!("\"{}\" is a reserved name.", arg_name).as_str());
                                }
    
                                arg_name.to_string()
                            })
                        );                     

                    }

                    current_macro = Some(MacroDefinition::new(
                        macro_name.to_string(),
                        macro_args,
                        AssemblyCode::new(),
                        asm_unit.path.to_path_buf(),
                        line_number,
                        to_export
                    ));

                    current_section = ProgramSection::MacroDefinition;

                    continue;
                }


                // Check for macro call (starts with !)
                if let Some(macro_call) = trimmed_line.strip_prefix('!').map(|s| s.trim()) {

                    let (macro_name, post) = macro_call.split_once(
                        |c: char| c.is_whitespace() || c == ':'
                    ).unwrap_or_else(
                        || error::invalid_macro_call(asm_unit.path, line_number, line, "Macro calls must have a name")
                    );

                    // Get the macro definition
                    let def = local_macro_definition_map.get(macro_name).unwrap_or_else(
                        || error::undeclared_macro(asm_unit.path, macro_name, line_number, line)
                    );

                    // Get the arguments of the macro call
                    let macro_args: Vec<&str> = post.split(char::is_whitespace)
                        .map(|s| s.trim())
                        .filter(|s| !s.is_empty())
                        .collect();             
                    
                    if macro_args.len() != def.args.len() {
                        error::invalid_macro_call(asm_unit.path, line_number, line, format!("Macro \"{}\" ({}, {}) expects {} arguments, but {} were given.", macro_name, def.unit_path.display(), def.line_number, def.args.len(), macro_args.len()).as_str());
                    }
 
                    // Substitute the provided arguments with the placeholders in the macro body and assemble the line
                    // Not very efficient, but works
                    for mline in def.body.iter() {

                        let mut mline = mline.clone();

                        for (i, arg) in def.args.iter().enumerate() {
                            mline = mline.replace(format!("{{{}}}", arg).as_str(), macro_args[i]);
                        }

                        assemble_instruction(&asm_unit, &mline, line, line_number, byte_code, &mut label_reference_registry);
                        
                    }

                    continue;
                }

                // The line doesn't contain label declarations, macros, or special stuff: it is a regular instruction
                assemble_instruction(&asm_unit, trimmed_line, line, line_number, byte_code, &mut label_reference_registry);

            },

            // Code cannot be put outside of a program section
            ProgramSection::None => error::out_of_section(asm_unit.path, line_number, line)

        }

        if verbose {
            println!(" => {:?}", &byte_code[last_byte_code_address..]);
        }
        
    }

    // After tokenization and conversion to bytecode, substitute the labels with their real address

    for reference in label_reference_registry {

        let real_address = local_label_declaration_map.get(&reference.name).unwrap_or_else(
            || error::undeclared_label(asm_unit.path, &reference.name, reference.line_number, &asm_unit.assembly[reference.line_number - 1])
        );

        // Substitute the label with the real address (little endian)
        byte_code[reference.location..reference.location + ADDRESS_SIZE].copy_from_slice(&real_address.to_le_bytes());

    }

    if verbose {
        if asm_unit.is_main_unit {
            println!("\nEnd of main assembly unit {} ({})\n", asm_unit.path.file_name().unwrap().to_string_lossy(), asm_unit.path.display());
        } else {
            println!("\nEnd of assembly unit {} ({})", asm_unit.path.file_name().unwrap().to_string_lossy(), asm_unit.path.display());
            println!("Exported labels: {:?}\n", export_label_declaration_map);
            println!("Exported macros: {:?}\n", export_macro_declaration_map);
        }
    }

    // Export the labels of the assembly unit
    external_export_label_declaration_map.extend(export_label_declaration_map.clone());
    external_export_macro_definition_map.extend(export_macro_declaration_map.clone());

    // Add the assembly unit to the included units
    included_units.insert(asm_unit.path.to_path_buf(), (export_label_declaration_map, export_macro_declaration_map));

}


fn assemble_instruction(asm_unit: &AssemblyUnit, trimmed_line: &str, line: &str, line_number: usize, byte_code: &mut ByteCode, label_reference_registry: &mut LabelReferenceRegistry) {

    // Split the operator from its arguments
    let (operator_name, raw_tokens): (&str, &str) = trimmed_line.split_once(char::is_whitespace).unwrap_or((
        trimmed_line, ""
    ));

    let operands = tokenize_operands(raw_tokens, line_number, line, asm_unit.path);
    
    let arg_table = get_arguments_table(operator_name).unwrap_or_else(
        || error::invalid_instruction_name(asm_unit.path, operator_name, line_number, line)
    );

    let operation = arg_table.get_operation(operator_name, &operands, asm_unit.path, line_number, line);

    // Convert the operands to byte code and append them to the byte code
    let converter = get_token_converter(operation.instruction);

    // Add the instruction code to the byte code
    byte_code.push(operation.instruction as u8);

    // Add the operands to the byte code
    let operand_bytes = use_converter(converter, operands, operation.handled_size, label_reference_registry, byte_code.len(), line_number, asm_unit.path, line);
    
    if operand_bytes.len() != operation.total_arg_size as usize {
        panic!("The generated operand byte code size {} for instruction \"{}\" does not match the expected size {}. This is a bug.", operand_bytes.len(), operation.instruction, operation.total_arg_size);
    }

    byte_code.extend(operand_bytes);

}


/// Assembles the assembly code into byte code
pub fn assemble(assembly: AssemblyCode, verbose: bool, unit_path: &Path) -> ByteCode {

    // Keep track of all the assembly units included to avoid duplicates
    let mut included_units = ASMUnitMap::new();

    let mut byte_code = ByteCode::new();

    let mut label_map = LabelMap::new();
    let mut macro_map = MacroMap::new();

    let asm_unit = AssemblyUnit::new(unit_path, assembly, true);

    // Assemble recursively the main assembly unit and its dependencies
    assemble_unit(asm_unit, verbose, &mut byte_code, &mut label_map, &mut macro_map, &mut included_units);

    // Append the exit instruction to the end of the binary
    byte_code.push(ByteCodes::EXIT as u8);

    // Append the address of the program start to the end of the binary

    let program_start = label_map.get("start").unwrap_or_else(
        || error::undeclared_label(unit_path, "start", 0, "The program must have a start label.")
    );

    byte_code.extend(program_start.to_le_bytes());

    if verbose {
        println!("Byte code size is {} bytes", byte_code.len());
        println!("Start address is {}", program_start);
    }

    byte_code
}

