use rust_vm_lib::assembly::{AssemblyCode, ByteCode};
use rust_vm_lib::byte_code::{ByteCodes, is_jump_instruction};
use rust_vm_lib::registers;
use rust_vm_lib::vm::{Address, ADDRESS_SIZE};
use rust_vm_lib::token::TokenValue;

use crate::data_types::DataType;
use crate::encoding::number_to_bytes;
use crate::error;
use crate::tokenizer::{tokenize_operands, is_label_name};
use crate::argmuments_table::{get_arguments_table, ArgTable};
use crate::token_to_byte_code::INSTRUCTION_CONVERSION_TABLE;
use crate::files;
use crate::argmuments_table;
use crate::configs;

use std::collections::HashMap;
use std::io;
use std::path::{Path, PathBuf};


pub type LabelMap = HashMap<String, Address>;


/// Represents a section in the assembly code
enum ProgramSection {
    Data,
    Text,
    Include,
    None,
}


/// Returns whether the name is a reserved name by the assembler
/// Reserved names are register names and instruction names
fn is_reserved_name(name: &str) -> bool {
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

    let parent_dir = current_unit_path.parent().unwrap_or_else(
        || panic!("Failed to get parent directory of \"{}\"", current_unit_path.display())
    );  

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
fn assemble_unit(assembly: AssemblyCode, verbose: bool, unit_path: &Path, byte_code: &mut ByteCode, export_label_map: &mut LabelMap, included_units: &mut Vec<String>, is_main_unit: bool) {

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
    
    // Stores the binary location of all the local labels
    let mut local_label_map = LabelMap::new();

    let mut current_section = ProgramSection::None;

    let mut has_data_section = false;
    let mut has_text_section = false;
    let mut has_include_section = false;

    let mut line_number: usize = 0;
    for line in assembly {
        line_number += 1;

        if verbose {
            println!("Line {}\t| {}", line_number, line);
        }
        let last_byte_code_length: Address = byte_code.len();

        // Evaluate the compile-time special symbols
        let line = evaluate_special_symbols(&line, last_byte_code_length, line_number, unit_path);

        // Remove redundant whitespaces
        let trimmed_line = line.trim();

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

                    if verbose {
                        println!(".include:\n");
                    }

                    continue;
                },

                "data" => {
                    // Check for duplicate sections
                    if has_data_section {
                        error::invalid_section_declaration(unit_path, section_name, line_number, &line, "An assembly unit can only have one data section.")
                    }
                    current_section = ProgramSection::Data;
                    has_data_section = true;
                    
                    if verbose {
                        println!(".data:\n");
                    }

                    continue;
                },

                "text" => {
                    // Check for duplicate sections
                    if has_text_section {
                        error::invalid_section_declaration(unit_path, section_name, line_number, &line, "An assembly unit can only have one text section.")
                    }
                    current_section = ProgramSection::Text;
                    has_text_section = true;

                    if verbose {
                        println!(".text:\n");
                    }

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
                assemble_unit(include_asm, verbose, &include_path, byte_code, &mut local_label_map, included_units, false);

            },

            ProgramSection::Data => {

                // Parse the static data declaration

                // Check if the data label has to be exported (double consecutive @)
                // Also, return an iterator over the statement
                let (mut statement_iter,  to_export) = {
                    if let Some(trimmed_line) = trimmed_line.strip_prefix("@@") {
                        (trimmed_line.split_whitespace(), true)
                    } else {
                        (trimmed_line.split_whitespace(), false)
                    }
                };

                let label = statement_iter.next().unwrap_or_else(
                    || error::invalid_data_declaration(unit_path, line_number, &line, "Static data declarations must have a label")
                );

                // Check if the label is reserved
                if is_reserved_name(label) {
                    error::invalid_label_name(unit_path, label, line_number, &line, format!("\"{}\" is a reserved name.", label).as_str());
                }

                let data_type_name = statement_iter.next().unwrap_or_else(
                    || error::invalid_data_declaration(unit_path, line_number, &line, "Static data declarations must have a type")
                );

                let data_type = DataType::from_name(data_type_name).unwrap_or_else(
                    || error::invalid_data_declaration(unit_path, line_number, &line, format!("Unknown data type \"{}\"", data_type_name).as_str())
                );

                // The data string is everything following the data type (kind of a shitty hacky way to do this)
                // TODO: improve this mess
                let data_string = trimmed_line.split_once(data_type_name).unwrap().1.trim();

                // Encode the string data into byte code
                let encoded_data: ByteCode = data_type.encode(data_string, line_number, &line, unit_path);

                byte_code.extend(encoded_data);

                // Add the data name and its address in the binary to the data map
                local_label_map.insert(label.to_string(), last_byte_code_length);

                if to_export {
                    export_label_map.insert(label.to_string(), last_byte_code_length);
                }

            },

            ProgramSection::Text => {

                if let Some(label) = trimmed_line.strip_prefix('@') {
                    // The line is a label, add it to the label map

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
                        export_label_map.insert(label.to_string(), last_byte_code_length);
                    } else if to_export {
                        export_label_map.insert(label.to_string(), last_byte_code_length);
                    }
                    
                    local_label_map.insert(label.to_string(), last_byte_code_length);
                    continue;
                }
        
                // List containing either a single operator or an operator and its arguments
                // TODO: handle non-strctly spaces in instructions
                let raw_tokens = trimmed_line.split_once(' ');
                
                if let Some(tokens) = raw_tokens {
                    // Operator has operands, tokenize the operands
                    let mut operands = tokenize_operands(tokens.1.to_string(), line_number, &line, &local_label_map, unit_path);
                    let operator = tokens.0;
        
                    let arg_table = get_arguments_table(operator).unwrap_or_else(
                        || error::invalid_instruction_name(unit_path, operator, line_number, &line)
                    );
        
                    let instruction_code: ByteCodes;
                    let handled_size: u8;
        
                    // Filter out all the possible byte code instructions associated with the operator
                    // This match statement is a mess, but I didn't want to rewrite the ARGUMENTS_TABLE
                    match arg_table {
                        ArgTable::One(argument) => {
                            // The operator has one argument
                            // Check if the operand number is valid
                            if operands.len() != 1 {
                                error::invalid_arg_number(unit_path, operands.len(), 1, line_number, &line, operator);
                            }
        
                            if let Some(possible) = argument.get(operands[0].value.to_ordinal() as usize) {
                                if let Some(instruction) = possible {
                                    instruction_code = instruction.0;
                                    handled_size = instruction.1;
                                } else {
                                    error::invalid_token_argument(unit_path, operator, &operands[0], line_number, &line);
                                }
        
                            } else {
                                // The operand type is not valid for the operation
                                error::invalid_token_argument(unit_path, operator, &operands[0], line_number, &line);
                            }
        
                        }
        
                        ArgTable::Two(argument) => {
                            // The operator has two arguments
                            // Check if the operand number is valid
                            if operands.len() != 2 {
                                error::invalid_arg_number(unit_path, operands.len(), 2, line_number, &line, operator);
                            }
        
                            if let Some(possible) = argument.get(operands[0].value.to_ordinal() as usize) {
                                if let Some(possible) = possible {
        
                                    if let Some(possible) = possible.get(operands[1].value.to_ordinal() as usize) {
                                        
                                        if let Some(instruction) = possible {
                                            instruction_code = instruction.0;
                                            handled_size = instruction.1;
                                        } else {
                                            error::invalid_token_argument(unit_path, operator, &operands[1], line_number, &line);
                                        }
        
                                    } else {
                                        // The operand type is not valid for the operation
                                        error::invalid_token_argument(unit_path, operator, &operands[1], line_number, &line);
                                    }
        
                                } else {
                                    error::invalid_token_argument(unit_path, operator, &operands[0], line_number, &line);
                                }
        
                            } else {
                                // The operand type is not valid for the operation
                                error::invalid_token_argument(unit_path, operator, &operands[0], line_number, &line);
                            }
                        }
        
                        _ => {
                            // The operator has no arguments, but some were given
                            error::invalid_arg_number(unit_path, operands.len(), 0, line_number, &line, operator);
                        }
                    }
        
                    // Substitute the label with the byte code location for jump instructions
                    if is_jump_instruction(instruction_code) {
                        if let TokenValue::Name(label) = &operands[0].value {
                            operands[0].value = TokenValue::AddressLiteral(*local_label_map.get(label).unwrap_or_else(
                                || error::undeclared_label(unit_path, label, line_number, &line)
                            ));
                        } else {
                            error::invalid_token(unit_path, &operands[0], line_number, &line, "Jump instructions can only take valid label names as arguments.");
                        }
                    }
        
                    // Convert the operands to byte code and append them to the byte code
                    let converter = INSTRUCTION_CONVERSION_TABLE.get(instruction_code as usize).unwrap_or_else(
                        || panic!("Unknown instruction \"{}\" at line {} \"{}\". This is a bug.", operator, line_number, line)
                    );
        
                    // Add the instruction code to the byte code
                    byte_code.push(instruction_code as u8);
        
                    // Add the operands to the byte code
                    match converter(&operands, handled_size) {

                        Ok(converted) => {
                            if let Some(operand_bytes) = converted {
                                byte_code.extend(operand_bytes);
                            } else {
                                // The instruction should have operands, but they could not be converted to bytecode
                                panic!("Operands for instruction \"{}\" at line {} \"{}\" could not be converted to bytecode. This is a bug.", operator, line_number, line);
                            }
                        },

                        Err(message) => error::invalid_instruction_arguments(unit_path, operator, line_number, &line, &message)
                    }
        
                } else {
                    // Operator has no operands
                    let operator = trimmed_line;
        
                    let arg_table = get_arguments_table(operator).unwrap_or_else(
                        || error::invalid_instruction_name(unit_path, operator, line_number, &line)
                    );
        
                    // In this branch possible_instructions is just a Tuple of ByteCodes and a u8
                    if let ArgTable::Zero(operation) = arg_table {
                        // Push the operator to the byte_code with no arguments
                        byte_code.push(operation.0 as u8);
                    } else {
                        // The operator requires arguments, but none were given
                        // TODO: make this modular
                        error::invalid_arg_number(unit_path, 0, arg_table.required_args(), line_number, &line, operator);
                    }
                    
                }
        
            },

            // Code cannot be put outside of a program section
            ProgramSection::None => error::out_of_section(unit_path, line_number, &line)

        }
        
        if verbose {
            println!("\t\t=> pos {}: {:?}", last_byte_code_length, &byte_code[last_byte_code_length..byte_code.len()]);
        }
        
    }

    if verbose {
        if is_main_unit {
            println!("\nEnd of main assembly unit {} ({})\n", unit_path.file_name().unwrap().to_string_lossy(), unit_path.display());
        } else {
            println!("\nEnd of assembly unit {} ({})", unit_path.file_name().unwrap().to_string_lossy(), unit_path.display());
            println!("Exported labels: {:?}\n", export_label_map);
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

    // Assume that the byte encoding is always successful
    byte_code.extend(
        number_to_bytes(*program_start as u64, ADDRESS_SIZE).unwrap()
    );

    byte_code
}

