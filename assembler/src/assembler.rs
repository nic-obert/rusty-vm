use rust_vm_lib::assembly::{AssemblyCode, ByteCode};
use rust_vm_lib::byte_code::{ByteCodes, is_jump_instruction};
use rust_vm_lib::vm::{Address, ADDRESS_SIZE};
use rust_vm_lib::token::TokenValue;

use crate::data_types::DataType;
use crate::encoding::number_to_bytes;
use crate::error::{self, invalid_label_name};
use crate::tokenizer::{tokenize_operands, is_label_name};
use crate::argmuments_table::{ARGUMENTS_TABLE, Args};
use crate::token_to_byte_code::INSTRUCTION_CONVERSION_TABLE;

use std::collections::HashMap;


pub type LabelMap = HashMap<String, Address>;


/// Represents a section in the assembly code
enum ProgramSection {
    Data,
    Text,
    None,
}


/// Evaluates special compile time assembly symbols and returns the evaluated line
/// Substitutes $ symbols with the current binary address
/// Does not substitute $ symbols inside strings or character literals
/// Does not evaluate escape characters inside strings or character literals
fn evaluate_special_symbols(line: &str, current_binary_address: Address, line_number: usize) -> String {

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
            error::unclosed_char_literal(starts_at.0, starts_at.1, line);
        },
        TextType::String { starts_at } => {
            error::unclosed_string_literal(starts_at.0, starts_at.1, line);
        }
    }

    evaluated_line
}


/// Assembles the assembly code into byte code
pub fn assemble(assembly: AssemblyCode, verbose: bool) -> ByteCode {

    let mut byte_code = ByteCode::new();

    // Stores the binary location of all the labels
    let mut label_map = LabelMap::new();

    let mut current_section = ProgramSection::None;

    let mut program_start: Address = 0;

    let mut has_data_section = false;
    let mut has_text_section = false;

    let mut line_number: usize = 0;
    for line in assembly {
        line_number += 1;

        if verbose {
            println!("Line {}\t| {}", line_number, line);
        }
        let last_byte_code_length: Address = byte_code.len();

        // Evaluate the compile-time special symbols
        let line = evaluate_special_symbols(&line, last_byte_code_length, line_number);

        // Remove redundant whitespaces
        let trimmed_line = line.trim();

        if trimmed_line.is_empty() || trimmed_line.starts_with('#') {
            // The line is either empty or a comment, skip it
            continue;
        }

        if trimmed_line.starts_with('.') {
            // This line specifies a program section
            let section_name = trimmed_line.strip_prefix('.').unwrap();
            let section_name = section_name.strip_suffix(':').unwrap_or_else(
                || error::invalid_section_declaration(section_name, line_number, &line, "Binary sections must end with a colon.")
            );

            match section_name {

                "data" => {
                    // Check for duplicate sections
                    if has_data_section {
                        error::invalid_section_declaration(section_name, line_number, &line, "A binary can only have one data section.")
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
                        error::invalid_section_declaration(section_name, line_number, &line, "A binary can only have one text section.")
                    }
                    current_section = ProgramSection::Text;
                    program_start = last_byte_code_length;
                    has_text_section = true;

                    if verbose {
                        println!(".text:\n");
                    }

                    continue;
                },

                _ => error::invalid_section_declaration(section_name, line_number, &line, format!("Unknown section name: \"{}\"", section_name).as_str())
            }
            
        }

        // Handle the assembly code depending on the current section
        match current_section {

            ProgramSection::Data => {

                // Parse the static data declaration

                let mut statement_iter = trimmed_line.split_whitespace();

                let label = statement_iter.next().unwrap_or_else(
                    || error::invalid_data_declaration(line_number, &line, "Static data declarations must have a label")
                );

                let data_type_name = statement_iter.next().unwrap_or_else(
                    || error::invalid_data_declaration(line_number, &line, "Static data declarations must have a type")
                );

                let data_type = DataType::from_name(data_type_name).unwrap_or_else(
                    || error::invalid_data_declaration(line_number, &line, format!("Unknown data type \"{}\"", data_type_name).as_str())
                );

                let data_string = statement_iter.next().unwrap_or_else(
                    || error::invalid_data_declaration(line_number, &line, "Static data declarations must have a value")
                );

                if statement_iter.next().is_some() {
                    error::invalid_data_declaration(line_number, &line, "Static data declarations can only have a label, a type and a value");
                }
                
                // Encode the string data into byte code
                let encoded_data: ByteCode = data_type.encode(data_string, line_number, &line);

                byte_code.extend(encoded_data);
                // Add the data name and its address in the binary to the data map
                label_map.insert(label.to_string(), last_byte_code_length);

            },

            ProgramSection::Text => {

                if trimmed_line.starts_with('@') {
                    // The line is a label, add it to the label map

                    let label = trimmed_line.strip_prefix('@').unwrap();
                    
                    if !is_label_name(label) {
                        invalid_label_name(label, line_number, &line);
                    }
                    
                    label_map.insert(label.to_string(), last_byte_code_length);
                    continue;
                }
        
                // List containing either a single operator or an operator and its arguments
                // TODO: handle non-strctly spaces
                let raw_tokens = trimmed_line.split_once(' ');
                
                if let Some(tokens) = raw_tokens {
                    // Operator has operands, tokenize the operands
                    let mut operands = tokenize_operands(tokens.1.to_string(), line_number, &line, &label_map);
                    let operator = tokens.0;
        
                    let possible_instructions = ARGUMENTS_TABLE.get(operator).unwrap_or_else(
                        || error::invalid_instruction_name(operator, line_number, &line)
                    );
        
                    let instruction_code: ByteCodes;
                    let handled_size: u8;
        
                    // Filter out all the possible byte code instructions associated with the operator
                    // This match statement is a mess, but I didn't want to rewrite the ARGUMENTS_TABLE
                    match possible_instructions {
                        Args::One(argument) => {
                            // The operator has one argument
                            // Check if the operand number is valid
                            if operands.len() != 1 {
                                error::invalid_arg_number(operands.len(), 1, line_number, &line, operator);
                            }
        
                            if let Some(possible) = argument.get(operands[0].value.to_ordinal() as usize) {
                                if let Some(instruction) = possible {
                                    instruction_code = instruction.0;
                                    handled_size = instruction.1;
                                } else {
                                    error::invalid_token_argument(operator, &operands[0], line_number, &line);
                                }
        
                            } else {
                                // The operand type is not valid for the operation
                                error::invalid_token_argument(operator, &operands[0], line_number, &line);
                            }
        
                        }
        
                        Args::Two(argument) => {
                            // The operator has two arguments
                            // Check if the operand number is valid
                            if operands.len() != 2 {
                                error::invalid_arg_number(operands.len(), 2, line_number, &line, operator);
                            }
        
                            if let Some(possible) = argument.get(operands[0].value.to_ordinal() as usize) {
                                if let Some(possible) = possible {
        
                                    if let Some(possible) = possible.get(operands[1].value.to_ordinal() as usize) {
                                        
                                        if let Some(instruction) = possible {
                                            instruction_code = instruction.0;
                                            handled_size = instruction.1;
                                        } else {
                                            error::invalid_token_argument(operator, &operands[1], line_number, &line);
                                        }
        
                                    } else {
                                        // The operand type is not valid for the operation
                                        error::invalid_token_argument(operator, &operands[1], line_number, &line);
                                    }
        
                                } else {
                                    error::invalid_token_argument(operator, &operands[0], line_number, &line);
                                }
        
                            } else {
                                // The operand type is not valid for the operation
                                error::invalid_token_argument(operator, &operands[0], line_number, &line);
                            }
                        }
        
                        _ => {
                            // The operator has no arguments, but some were given
                            error::invalid_arg_number(operands.len(), 0, line_number, &line, operator);
                        }
                    }
        
                    // Substitute the label with the byte code location for jump instructions
                    if is_jump_instruction(instruction_code) {
                        if let TokenValue::Name(label) = &operands[0].value {
                            operands[0].value = TokenValue::AddressLiteral(*label_map.get(label).unwrap_or_else(
                                || error::undeclared_label(label, line_number, &line)
                            ));
                        } else {
                            error::invalid_token(&operands[0], line_number, &line, "Jump instructions can only take valid label names as arguments.");
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
                        Err(message) => error::invalid_instruction_arguments(operator, line_number, &line, &message)
                    }
        
                } else {
                    // Operator has no operands
                    let operator = trimmed_line;
        
                    let possible_instructions = ARGUMENTS_TABLE.get(operator).unwrap_or_else(
                        || error::invalid_instruction_name(operator, line_number, &line)
                    );
        
                    // In this branch possible_instructions is just a Tuple of ByteCodes and a u8
                    if let Args::Zero(operation) = possible_instructions {
                        // Push the operator to the byte_code with no arguments
                        byte_code.push(operation.0 as u8);
                    }
                    
                }
        
            },

            // Code cannot be put outside of a program section
            ProgramSection::None => error::out_of_section(line_number, &line)

        }
        
        if verbose {
            println!("\t\t=> pos {}: {:?}", last_byte_code_length, &byte_code[last_byte_code_length..byte_code.len()]);
        }
        
    }

    // Append the exit instruction to the end of the binary
    byte_code.push(ByteCodes::EXIT as u8);

    // Append the address of the program start to the end of the binary
    // Assume that the byte encoding is always successful
    byte_code.extend(number_to_bytes(program_start as u64, ADDRESS_SIZE).unwrap());

    byte_code
}

