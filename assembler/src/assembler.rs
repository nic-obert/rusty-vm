use rust_vm_lib::assembly::{AssemblyCode, ByteCode};
use rust_vm_lib::byte_code::ByteCodes;
use rust_vm_lib::registers;
use rust_vm_lib::vm::Address;

use crate::data_types::DataType;
use crate::error;
use crate::tokenizer::{tokenize_operands, is_label_name};
use crate::argmuments_table::get_arguments_table;
use crate::token_to_byte_code::{get_token_converter, use_converter};
use crate::files;
use crate::argmuments_table;
use crate::configs;

use std::collections::HashMap;
use std::io;
use std::path::{Path, PathBuf};


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
    None,
}


/// Returns whether the name is a reserved name by the assembler
/// Reserved names are register names and instruction names
pub fn is_reserved_name(name: &str) -> bool {
    if let Some(_) = registers::get_register(name) {
        true
    } else if argmuments_table::get_arguments_table(name).is_some() {
        true
    } else {
        false
    }
    
}


/// Try to load an assembly unit
/// 
/// If successful, return the assembly code and the absolute path to the unit
fn load_asm_unit(unit_name: &str, current_unit_path: &Path) -> io::Result<(PathBuf, AssemblyCode)> {

    // TODO: search in standard assembly library first

    let unit_path = Path::new(unit_name);

    if unit_path.is_absolute() {
        // The unit path is absolute, try to load it directly
        return Ok((unit_path.to_path_buf(), files::load_assembly(unit_path)?));
    }

    // The unit path is relative

    // Try to load the unit from the standard library
    {
        let unit_path = configs::INCLUDE_LIB_PATH.join(unit_path);
        match files::load_assembly(&unit_path) {
            Ok(assembly) => return Ok((unit_path.canonicalize().unwrap(), assembly)),
            Err(_) => {}
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
                } else {
                    if c == '"' {
                        text_type = TextType::Asm;
                    } else if c == '\\' {
                        escape_char = true;
                    }
                }
            },

            TextType::Char {..} => {

                evaluated_line.push(c);

                if escape_char {
                    escape_char = false;
                } else {
                    if c == '\'' {
                        text_type = TextType::Asm;
                    } else if c == '\\' {
                        escape_char = true;
                    }
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


/// Assemble recursively an assembly unit and its dependencies
fn assemble_unit(assembly: AssemblyCode, verbose: bool, unit_path: &Path, byte_code: &mut ByteCode, export_label_declaration_map: &mut LabelMap, included_units: &mut Vec<String>, is_main_unit: bool) {

    // Check if the assembly unit has already been included
    let unit_path_string = unit_path.to_string_lossy().to_string();
    if included_units.contains(&unit_path_string) {
        // The assembly unit has already been included, skip it
        return;
    }
    included_units.push(unit_path_string);

    if verbose {
        if is_main_unit {
            println!("\nMain assembly unit: {} ({})\n", unit_path.file_name().unwrap().to_string_lossy(), unit_path.display());
        } else {
            println!("\nAssembly unit: {} ({})\n", unit_path.file_name().unwrap().to_string_lossy(), unit_path.display());
        }
    }
    
    // Stores the address in the bytecode of all the local labels
    let mut local_label_declaration_map = LabelMap::new();
    // Stores the references to labels and when they are referenced in the bytecode
    // Used later to substitute the labels with real addresses
    let mut label_reference_registry = LabelReferenceRegistry::new();

    let mut current_section = ProgramSection::None;

    let mut has_data_section = false;
    let mut has_text_section = false;
    let mut has_include_section = false;

    for (i, line) in assembly.iter().enumerate() {
        let line_number = i + 1;

        if verbose {
            println!("Line {: >4}, Pos: {: >5} | {}", line_number, byte_code.len(), line);
        }

        // Evaluate the compile-time special symbols
        let evaluated_line = evaluate_special_symbols(&line, byte_code.len(), line_number, unit_path);

        // Remove redundant whitespaces
        let trimmed_line = evaluated_line.trim();

        if trimmed_line.is_empty() || trimmed_line.starts_with('#') {
            // The line is either empty or a comment, skip it
            continue;
        }

        if let Some(mut section_name) = trimmed_line.strip_prefix('.') {
            // This line specifies a program section
            section_name = section_name.strip_suffix(':').unwrap_or_else(
                || error::invalid_section_declaration(unit_path, section_name, line_number, &line, "Assembly sections must end with a colon.")
            );

            match section_name {

                "include" => {
                    // Check for duplicate sections
                    if has_include_section {
                        error::invalid_section_declaration(unit_path, section_name, line_number, &line, "An assembly unit can only have one include section.")
                    }
                    current_section = ProgramSection::Include;
                    has_include_section = true;

                    continue;
                },

                "data" => {
                    // Check for duplicate sections
                    if has_data_section {
                        error::invalid_section_declaration(unit_path, section_name, line_number, &line, "An assembly unit can only have one data section.")
                    }
                    current_section = ProgramSection::Data;
                    has_data_section = true;

                    continue;
                },

                "text" => {
                    // Check for duplicate sections
                    if has_text_section {
                        error::invalid_section_declaration(unit_path, section_name, line_number, &line, "An assembly unit can only have one text section.")
                    }
                    current_section = ProgramSection::Text;
                    has_text_section = true;

                    continue;
                },

                _ => error::invalid_section_declaration(unit_path, section_name, line_number, &line, format!("Unknown assembly section name: \"{}\"", section_name).as_str())
            }
            
        }

        // Handle the assembly code depending on the current section
        match current_section {

            ProgramSection::Include => {

                let include_unit_raw = trimmed_line;

                let (include_path, include_asm) = match load_asm_unit(include_unit_raw, unit_path) {
                    Ok(x) => x,
                    Err(error) => error::include_error(unit_path, &error, include_unit_raw, line_number, &line)
                };

                // Assemble the included assembly unit
                assemble_unit(include_asm, verbose, &include_path, byte_code, &mut local_label_declaration_map, included_units, false);

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
                    || error::invalid_data_declaration(unit_path, line_number, &line, "Static data declarations must have a label")
                );

                // Check if the label is a reserved keyword
                if is_reserved_name(label) {
                    error::invalid_label_name(unit_path, label, line_number, &line, format!("\"{}\" is a reserved name.", label).as_str());
                }

                let (data_type_name, other) = other.split_once(char::is_whitespace).unwrap_or_else(
                    || error::invalid_data_declaration(unit_path, line_number, &line, "Static data declarations must have a type")
                );

                let data_type = DataType::from_name(data_type_name).unwrap_or_else(
                    || error::invalid_data_declaration(unit_path, line_number, &line, format!("Unknown data type \"{}\"", data_type_name).as_str())
                );

                // The data string is everything following the data type
                let data_string = other.trim();

                // Encode the string data into byte code
                let encoded_data: ByteCode = data_type.encode(data_string, line_number, &line, unit_path);

                byte_code.extend(encoded_data);

                // Add the data name and its address in the binary to the data map
                local_label_declaration_map.insert(label.to_string(), byte_code.len());

                if to_export {
                    export_label_declaration_map.insert(label.to_string(), byte_code.len());
                }

            },

            ProgramSection::Text => {

                // Check for label declarations first
                if let Some(label) = trimmed_line.strip_prefix('@') {
                    // The line is a label declaration, add it to the label declaration map

                    // Check if the label is to be exported (double consecutive @)
                    let (label, to_export): (&str, bool) = {
                        if let Some(label) = label.strip_prefix('@') {
                            (label.trim(), true)
                        } else {
                            (label.trim(), false)
                        }
                    };
                    
                    if !is_label_name(label) {
                        error::invalid_label_name(unit_path, label, line_number, &line, "Label names can only contain alphabetic characters and underscores.");
                    }

                    if is_main_unit && label == "start" {
                        // Automatically export the @start label if this is the main assembly unit
                        export_label_declaration_map.insert(label.to_string(), byte_code.len());
                    } else if to_export {
                        export_label_declaration_map.insert(label.to_string(), byte_code.len());
                    }
                    
                    local_label_declaration_map.insert(label.to_string(), byte_code.len());
                    continue;
                }
                
                // Split the operator from its arguments
                let (operator_name, raw_tokens): (&str, &str) = trimmed_line.split_once(char::is_whitespace).unwrap_or((
                    trimmed_line, ""
                ));

                let operands = tokenize_operands(raw_tokens, line_number, line, unit_path);
                
                let arg_table = get_arguments_table(operator_name).unwrap_or_else(
                    || error::invalid_instruction_name(unit_path, operator_name, line_number, line)
                );
            
                let (instruction_code, handled_size) = arg_table.get_instruction(operator_name, &operands, unit_path, line_number, line);
            
                // Convert the operands to byte code and append them to the byte code
                let converter = get_token_converter(instruction_code);
            
                // Add the instruction code to the byte code
                byte_code.push(instruction_code as u8);
            
                // Add the operands to the byte code
                let operand_bytes = use_converter(converter, operands, handled_size, &mut label_reference_registry, byte_code.len(), line_number, unit_path, line);
                byte_code.extend(operand_bytes);

            },

            // Code cannot be put outside of a program section
            ProgramSection::None => error::out_of_section(unit_path, line_number, &line)

        }
        
    }

    // After tokenization and conversion to bytecode, substitute the labels with their real address

    for reference in label_reference_registry {

        let real_address = local_label_declaration_map.get(&reference.name).unwrap_or_else(
            || error::undeclared_label(unit_path, &reference.name, reference.line_number, &assembly[reference.line_number - 1])
        );

        // Substitute the label with the real address (little endian)
        byte_code[reference.location..reference.location + 8].copy_from_slice(&real_address.to_le_bytes());

    }

    if verbose {
        if is_main_unit {
            println!("\nEnd of main assembly unit {} ({})\n", unit_path.file_name().unwrap().to_string_lossy(), unit_path.display());
        } else {
            println!("\nEnd of assembly unit {} ({})", unit_path.file_name().unwrap().to_string_lossy(), unit_path.display());
            println!("Exported labels: {:?}\n", export_label_declaration_map);
        }
    }

}


/// Assembles the assembly code into byte code
pub fn assemble(assembly: AssemblyCode, verbose: bool, unit_path: &Path) -> ByteCode {

    // Keep track of all the assembly units included to avoid duplicates
    let mut included_units: Vec<String> = Vec::new();

    let mut byte_code = ByteCode::new();

    let mut label_map = LabelMap::new();

    // Assemble recursively the main assembly unit and its dependencies
    assemble_unit(assembly, verbose, unit_path, &mut byte_code, &mut label_map, &mut included_units, true);

    // Append the exit instruction to the end of the binary
    byte_code.push(ByteCodes::EXIT as u8);

    // Append the address of the program start to the end of the binary

    let program_start = label_map.get("start").unwrap_or_else(
        || error::undeclared_label(unit_path, "start", 0, "The program must have a start label.")
    );

    byte_code.extend(program_start.to_le_bytes());

    byte_code
}

