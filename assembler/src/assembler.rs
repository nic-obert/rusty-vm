use rust_vm_lib::byte_code::{ByteCodes, is_jump_instruction};
use crate::tokenizer::tokenize_operands;
use crate::argmuments_table::{ARGUMENTS_TABLE, Args};
use rust_vm_lib::token::TokenValue;
use crate::token_to_byte_code::INSTRUCTION_CONVERSION_TABLE;
use std::collections::HashMap;


pub type AssemblyCode = Vec<String>;
pub type ByteCode = Vec<u8>;


pub fn assemble(assembly: AssemblyCode) -> ByteCode {

    let mut byte_code = ByteCode::new();

    let mut label_map: HashMap<String, usize> = HashMap::new();

    let mut line_number: usize = 0;
    for line in assembly {
        line_number += 1;

        // Remove redundant whitespaces
        let stripped_line = line.strip_prefix(' ').unwrap().strip_suffix(' ').unwrap();
        if stripped_line.is_empty() || stripped_line.starts_with(';') {
            // The line is either empty or a comment, skip it
            continue;
        }

        // List containing either a single operator or an operator and its arguments
        let raw_tokens = stripped_line.split_once(' ');
        
        if let Some(tokens) = raw_tokens {
            // Operator has operands, tokenize the operands
            let mut operands = tokenize_operands(tokens.1.to_string());
            let operator = tokens.0;

            let possible_instructions = ARGUMENTS_TABLE.get(operator).unwrap_or_else(
                || panic!("Unknown instruction \"{}\" at line {} \"{}\"", operator, line_number, line)
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
                        panic!("Invalid number of arguments for instruction \"{}\" at line {} \"{}\", expected 1", operator, line_number, line);
                    }

                    if let Some(possible) = argument.get(operands[0].value.to_ordinal() as usize) {
                        if let Some(instruction) = possible {
                            instruction_code = instruction.0;
                            handled_size = instruction.1;
                        } else {
                            panic!("Invalid argument for instruction \"{}\" at line {} \"{}\"", operator, line_number, line);
                        }

                    } else {
                        // The operand type is not valid for the operation
                        panic!("Unknown operand \"{}\" for instruction \"{}\" in line {} \"{}\"", operands[0], operator, line_number, line);
                    }

                }

                Args::Two(argument) => {
                    // The operator has two arguments
                    // Check if the operand number is valid
                    if operands.len() != 2 {
                        panic!("Invalid number of arguments for instruction \"{}\" at line {} \"{}\", expected 2", operator, line_number, line);
                    }

                    if let Some(possible) = argument.get(operands[0].value.to_ordinal() as usize) {
                        if let Some(possible) = possible {

                            if let Some(possible) = possible.get(operands[1].value.to_ordinal() as usize) {
                                
                                if let Some(instruction) = possible {
                                    instruction_code = instruction.0;
                                    handled_size = instruction.1;
                                } else {
                                    panic!("Invalid argument for instruction \"{}\" at line {} \"{}\"", operator, line_number, line);
                                }

                            } else {
                                // The operand type is not valid for the operation
                                panic!("Unknown operand \"{}\" for instruction \"{}\" in line {} \"{}\"", operands[0], operator, line_number, line);
                            }

                        } else {
                            panic!("Invalid argument for instruction \"{}\" at line {} \"{}\"", operator, line_number, line);
                        }

                    } else {
                        // The operand type is not valid for the operation
                        panic!("Unknown operand \"{}\" for instruction \"{}\" in line {} \"{}\"", operands[0], operator, line_number, line);
                    }
                }

                _ => {
                    // In this branch, the operator has arguments, so Args::Zero is not a valid case
                    panic!("Invalid number of arguments for instruction \"{}\" at line {} \"{}\", expected 0", operator, line_number, line);
                }
            }

            // If the operator is a label, store its byte code location
            if matches!(instruction_code, ByteCodes::LABEL) {
                // Remove because the operand won't be used anymore and the loop will continue (or panic)
                if let TokenValue::Label(label) = operands.remove(0).value {
                    label_map.insert(label, byte_code.len());
                    // Labels are not part of the byte code, so skip the rest of the loop
                    continue;
                } else {
                    panic!("Invalid label at line {} \"{}\"", line_number, line);
                }
            }

            // Substitute the label with the byte code location for jump instructions
            if is_jump_instruction(instruction_code) {
                if let TokenValue::Label(label) = &operands[0].value {
                    operands[0].value = TokenValue::AddressLiteral(*label_map.get(label).unwrap_or_else(
                        || panic!("Unknown label \"{}\" at line {} \"{}\"", label, line_number, line)
                    ));
                } else {
                    panic!("Invalid label at line {} \"{}\"", line_number, line);
                }
            }

            // Convert the operands to byte code and append them to the byte code
            let converter = INSTRUCTION_CONVERSION_TABLE.get(instruction_code as usize).unwrap_or_else(
                || panic!("Unknown instruction \"{}\" at line {} \"{}\"", operator, line_number, line)
            );

            // Add the instruction code to the byte code
            byte_code.push(instruction_code as u8);
            // Add the operands to the byte code
            if let Some(operand_bytes) = converter(&operands, handled_size) {
                byte_code.extend(operand_bytes);
            } else {
                panic!("Operands for instruction \"{}\" at line {} \"{}\" are invalid", operator, line_number, line);
            }

        } else {
            // Operator has no operands
            let operator = stripped_line;

            let possible_instructions = ARGUMENTS_TABLE.get(operator).unwrap_or_else(
                || panic!("Unknown instruction \"{}\" at line {} \"{}\"", operator, line_number, line)
            );

            // In this branch possible_instructions is just a Tuple of ByteCodes and a u8
            if let Args::Zero(operation) = possible_instructions {
                // Push the operator to the byte_code with no arguments
                byte_code.push(operation.0 as u8);
            }
            
        }
        
    }

    byte_code
}

