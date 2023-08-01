use rust_vm_lib::assembly::{AssemblyCode, ByteCode};
use rust_vm_lib::byte_code::{ByteCodes, is_jump_instruction};
use rust_vm_lib::vm::{Address, ADDRESS_SIZE};
use crate::data_types::DataType;
use crate::encoding::number_to_bytes;
use crate::error;
use crate::tokenizer::{tokenize_operands, is_name_character};
use crate::argmuments_table::{ARGUMENTS_TABLE, Args};
use rust_vm_lib::token::TokenValue;
use crate::token_to_byte_code::INSTRUCTION_CONVERSION_TABLE;
use std::collections::HashMap;


enum Section {
    Data,
    Text,
    None,
}


pub fn assemble(assembly: AssemblyCode, verbose: bool) -> ByteCode {

    let mut byte_code = ByteCode::new();

    // Stores the binary location of all the labels
    let mut label_map: HashMap<String, Address> = HashMap::new();
    // Stores the binary location of all the data variables
    let mut data_map: HashMap<String, Address> = HashMap::new();

    let mut current_section = Section::None;

    let mut program_start: Address = 0;

    let mut has_data_section = false;
    let mut has_text_section = false;

    let mut line_number: usize = 0;
    for line in assembly {
        line_number += 1;

        if verbose {
            println!("\nLine {}\t: {}", line_number, line);
        }
        let last_byte_code_length: Address = byte_code.len();

        // Remove redundant whitespaces
        let stripped_line = line.strip_prefix(' ').unwrap_or(&line);
        let stripped_line = stripped_line.strip_suffix(' ').unwrap_or(stripped_line);

        if stripped_line.is_empty() || stripped_line.starts_with('#') {
            // The line is either empty or a comment, skip it
            continue;
        }

        if stripped_line.starts_with('.') {
            // This line specifies a program section
            let section_name = stripped_line.strip_prefix('.').unwrap();
            let section_name = section_name.strip_suffix(':').unwrap_or_else(
                || error::invalid_section_declaration(section_name, line_number, &line, "Binary sections must end with a colon.")
            );

            match section_name {

                "data" => {
                    // Check for duplicate sections
                    if has_data_section {
                        error::invalid_section_declaration(section_name, line_number, &line, "A binary can only have one data section.")
                    }
                    current_section = Section::Data;
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
                    current_section = Section::Text;
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

            Section::Data => {

                // Parse the data declaration

                let (name, other) = stripped_line.split_once(' ').unwrap_or_else(
                    || error::invalid_data_declaration(line_number, &line, "Data declarations must have a name")
                );

                let (data_type, data) = other.split_once(' ').unwrap_or_else(
                    || error::invalid_data_declaration(line_number, &line, "Data declarations must have a type")
                );

                let data_type = DataType::from_name(data_type).unwrap_or_else(
                    || error::invalid_data_declaration(line_number, &line, format!("Unknown data type \"{}\"", data_type).as_str())
                );

                // Encode the string data into byte code
                let encoded_data: ByteCode = data_type.encode(data, line_number, &line);

                byte_code.extend(encoded_data);
                // Add the data name and its address in the binary to the data map
                data_map.insert(name.to_string(), last_byte_code_length);

            },

            Section::Text => {

                if stripped_line.starts_with('@') {
                    // The line is a label, add it to the label map
                    // Check if the label is a valid name
                    let label = stripped_line.strip_prefix('@').unwrap();
                    
                    for (char_index, c) in label.chars().enumerate() {
                        if !is_name_character(c) {
                            // Kind of a hacky way to get the index of the character in the line
                            let i = line.find("@").unwrap() + 1 + char_index;
                            error::invalid_character(c, line_number, i, &line, format!("Invalid label name \"{}\". Label names can only contain letters and underscores.", label).as_str());
                        }
                    }
                    
                    let label = stripped_line[1..].to_string();
                    label_map.insert(label, last_byte_code_length);
                    continue;
                }
        
                // List containing either a single operator or an operator and its arguments
                let raw_tokens = stripped_line.split_once(' ');
                
                if let Some(tokens) = raw_tokens {
                    // Operator has operands, tokenize the operands
                    let mut operands = tokenize_operands(tokens.1.to_string(), line_number, &line);
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
                    let operator = stripped_line;
        
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

            // Code cannot be put outside of a section
            Section::None => error::out_of_section(line_number, &line)

        }
        
        if verbose {
            println!("\t\t=> pos {}| {:?}", last_byte_code_length, &byte_code[last_byte_code_length..byte_code.len()]);
        }
        
    }

    // Append the exit instruction to the end of the binary
    byte_code.push(ByteCodes::EXIT as u8);

    // Append the address of the program start to the end of the binary
    // Assume that the byte encoding is always successful
    byte_code.extend(number_to_bytes(program_start as u64, ADDRESS_SIZE).unwrap());

    byte_code
}

