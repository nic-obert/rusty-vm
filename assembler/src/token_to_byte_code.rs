#![allow(clippy::too_many_arguments)]
#![allow(clippy::no_effect)]

use std::mem;
use std::path::Path;

use rusty_vm_lib::assembly::ByteCode;
use rusty_vm_lib::registers::REGISTER_ID_SIZE;
use rusty_vm_lib::token::{Token, TokenValue};
use rusty_vm_lib::byte_code::ByteCodes;
use rusty_vm_lib::vm::{ADDRESS_SIZE, Address};

use crate::assembler::{LabelReferenceRegistry, AddLabelReference};
use crate::error;


/// Extract the value of a specific token variant. Treat other variants as unreachable.
macro_rules! extract {
    ($token:expr, $variant:ident) => {
        if let TokenValue::$variant(value) = $token.value {
            value
        } else {
            unreachable!()
        }
    };
}


/// Returns the number of bytes needed to represent the number.
/// 
/// This function assumes little endian.
fn number_size(number: i64) -> usize {
    match number.cmp(&0) {
        std::cmp::Ordering::Equal => 1,
        std::cmp::Ordering::Less => 8,
        std::cmp::Ordering::Greater => {
            number.to_le_bytes().iter().rev().skip_while(|&&b| b == 0).count()
        }
    }
}


/// Try to fit the given number into the given number of bytes.
fn fit_into_bytes(number: i64, size: u8) -> Option<ByteCode> {
    if number_size(number) <= size as usize {
        Some(number.to_le_bytes()[..size as usize].to_vec())
    } else {
        None
    }
}


/// A placeholder for the real address of a label.
const LABEL_PLACEHOLDER: [u8; ADDRESS_SIZE] = (0 as Address).to_le_bytes();


pub fn generate_operand_bytecode(instruction: ByteCodes, mut operands: Vec<Token>, handled_size: u8, label_registry: &mut LabelReferenceRegistry, last_byte_code: Address, line_number: usize, unit_path: &Path, line: &str) -> ByteCode {
    match instruction {

        ByteCodes::INTEGER_ADD |
        ByteCodes::INTEGER_SUB |
        ByteCodes::INTEGER_MUL |
        ByteCodes::INTEGER_DIV |
        ByteCodes::INTEGER_MOD |
        ByteCodes::FLOAT_ADD |
        ByteCodes::FLOAT_SUB |
        ByteCodes::FLOAT_MUL |
        ByteCodes::FLOAT_DIV |
        ByteCodes::FLOAT_MOD |
        ByteCodes::AND |
        ByteCodes::OR |
        ByteCodes::XOR |
        ByteCodes::NOT |
        ByteCodes::SHIFT_LEFT |
        ByteCodes::SHIFT_RIGHT |
        ByteCodes::RETURN |
        ByteCodes::EXIT
         => {
            ByteCode::new()
        },

        ByteCodes::INC_REG => {
            extract!(operands[0], Register).to_bytes().to_vec()
        },

        ByteCodes::INC_ADDR_IN_REG => {
            vec![
                handled_size,
                extract!(operands[0], AddressInRegister) as u8
            ]
        },

        ByteCodes::INC_ADDR_LITERAL => {
            let cap = 1 + ADDRESS_SIZE;
            let mut bytes = ByteCode::with_capacity(cap);
            bytes.push(handled_size);
        
            match &mut operands[0].value {
                TokenValue::AddressLiteral { value, .. } => {
                    bytes.extend(value.to_le_bytes());
                },
                TokenValue::AddressAtLabel(label) => {
                    label_registry.add_reference(mem::take(label), last_byte_code + bytes.len(), line_number);
                    bytes.extend(LABEL_PLACEHOLDER);
                },
                _ => unreachable!()
            }
        
            bytes
        },

        ByteCodes::DEC_REG => {
            extract!(operands[0], Register).to_bytes().to_vec()
        },

        ByteCodes::DEC_ADDR_IN_REG => {
            vec![
                handled_size,
                extract!(operands[0], AddressInRegister) as u8
            ]
        },

        ByteCodes::DEC_ADDR_LITERAL => {
            let cap = 1 + ADDRESS_SIZE;
            let mut bytes = ByteCode::with_capacity(cap);
            bytes.push(handled_size);
        
            match &mut operands[0].value {
                TokenValue::AddressLiteral { value, .. } => {
                    bytes.extend(value.to_le_bytes());
                },
                TokenValue::AddressAtLabel(label) => {
                    label_registry.add_reference(mem::take(label), last_byte_code + bytes.len(), line_number);
                    bytes.extend(LABEL_PLACEHOLDER);
                },
                _ => unreachable!()
            }
        
            bytes
        },

        ByteCodes::NO_OPERATION => todo!(),

        ByteCodes::MOVE_INTO_REG_FROM_REG => {
            vec![
                extract!(operands[0], Register) as u8,
                extract!(operands[1], Register) as u8
            ]
        },

        ByteCodes::MOVE_INTO_REG_FROM_ADDR_IN_REG => {
            vec![
                handled_size,
                extract!(operands[0], Register) as u8,
                extract!(operands[1], AddressInRegister) as u8
            ]
        },

        ByteCodes::MOVE_INTO_REG_FROM_CONST => {
            let mut bytes = ByteCode::with_capacity(1 + REGISTER_ID_SIZE + handled_size as usize);
            bytes.push(handled_size);
        
            let dest_reg = extract!(operands[0], Register) as u8;
            bytes.push(dest_reg);
        
            match &mut operands[1].value {
                TokenValue::Number { value, .. }=> {
                    let repr = fit_into_bytes(*value, handled_size).unwrap_or_else(
                        || error::number_out_of_range::<u64>(unit_path, value.to_string().as_str(), 10, handled_size, line_number, line)
                    );
                    bytes.extend(repr);
                },
                TokenValue::Label(label) => {
                    label_registry.add_reference(mem::take(label), last_byte_code + bytes.len(), line_number);
                    bytes.extend(LABEL_PLACEHOLDER);
                },
                _ => unreachable!()
            }
        
            bytes
        },

        ByteCodes::MOVE_INTO_REG_FROM_ADDR_LITERAL => {
            let mut bytes = ByteCode::with_capacity(1 + REGISTER_ID_SIZE + ADDRESS_SIZE);
            bytes.push(handled_size);
        
            let dest_reg = extract!(operands[0], Register) as u8;
            bytes.push(dest_reg);
        
            match &mut operands[1].value {
                TokenValue::AddressLiteral { value, .. } => bytes.extend(value.to_le_bytes()),
                TokenValue::AddressAtLabel(label) => {
                    label_registry.add_reference(mem::take(label), last_byte_code + bytes.len(), line_number);
                    bytes.extend(LABEL_PLACEHOLDER);
                },
                _ => unreachable!()
            }
            
            bytes
        },

        ByteCodes::MOVE_INTO_ADDR_IN_REG_FROM_REG => {
            vec![
                handled_size,
                extract!(operands[0], AddressInRegister) as u8,
                extract!(operands[1], Register) as u8
            ]
        },

        ByteCodes::MOVE_INTO_ADDR_IN_REG_FROM_ADDR_IN_REG => {
            vec![
                handled_size,
                extract!(operands[0], AddressInRegister) as u8,
                extract!(operands[1], AddressInRegister) as u8
            ]
        },

        ByteCodes::MOVE_INTO_ADDR_IN_REG_FROM_CONST => {
            let mut bytes = ByteCode::with_capacity(1 + REGISTER_ID_SIZE + handled_size as usize);
            bytes.push(handled_size);
        
            let dest_reg = extract!(operands[0], AddressInRegister) as u8;
            bytes.push(dest_reg);
        
            match &mut operands[1].value {
                TokenValue::Number { value, .. }=> {
                    let repr = fit_into_bytes(*value, handled_size).unwrap_or_else(
                        || error::number_out_of_range::<u64>(unit_path, value.to_string().as_str(), 10, handled_size, line_number, line)
                    );
                    bytes.extend(repr);
                },
                TokenValue::Label(label) => {
                    label_registry.add_reference(mem::take(label), last_byte_code + bytes.len(), line_number);
                    bytes.extend(LABEL_PLACEHOLDER);
                },
                _ => unreachable!()
            }
        
            bytes
        },

        ByteCodes::MOVE_INTO_ADDR_IN_REG_FROM_ADDR_LITERAL => {
            let mut bytes = ByteCode::with_capacity(1 + REGISTER_ID_SIZE + ADDRESS_SIZE);
            bytes.push(handled_size);
        
            let dest_reg = extract!(operands[0], AddressInRegister) as u8;
            bytes.push(dest_reg);
        
            match &mut operands[1].value {
                TokenValue::AddressLiteral { value, .. } => bytes.extend(value.to_le_bytes()),
                TokenValue::AddressAtLabel(label) => {
                    label_registry.add_reference(mem::take(label), last_byte_code + bytes.len(), line_number);
                    bytes.extend(LABEL_PLACEHOLDER);
                },
                _ => unreachable!()
            }
        
            bytes
        },

        ByteCodes::MOVE_INTO_ADDR_LITERAL_FROM_REG => {
            let mut bytes = ByteCode::with_capacity(1 + ADDRESS_SIZE + REGISTER_ID_SIZE);
            bytes.push(handled_size);
        
            match &mut operands[0].value {
                TokenValue::AddressLiteral { value, .. } => bytes.extend(value.to_le_bytes()),
                TokenValue::AddressAtLabel(label) => {
                    label_registry.add_reference(mem::take(label), last_byte_code + bytes.len(), line_number);
                    bytes.extend(LABEL_PLACEHOLDER);
                },
                _ => unreachable!()
            }
        
            let src_reg = extract!(operands[1], Register) as u8;
            bytes.push(src_reg);
        
            bytes
        },

        ByteCodes::MOVE_INTO_ADDR_LITERAL_FROM_ADDR_IN_REG => {
            let mut bytes = ByteCode::with_capacity(1 + ADDRESS_SIZE + REGISTER_ID_SIZE);
            bytes.push(handled_size);
        
            match &mut operands[0].value {
                TokenValue::AddressLiteral { value, .. } => bytes.extend(value.to_le_bytes()),
                TokenValue::AddressAtLabel(label) => {
                    label_registry.add_reference(mem::take(label), last_byte_code + bytes.len(), line_number);
                    bytes.extend(LABEL_PLACEHOLDER);
                },
                _ => unreachable!()
            }
        
            let src_reg = extract!(operands[1], AddressInRegister) as u8;
            bytes.push(src_reg);
        
            bytes
        },

        ByteCodes::MOVE_INTO_ADDR_LITERAL_FROM_CONST => {
            let mut bytes = ByteCode::with_capacity(1 + ADDRESS_SIZE + handled_size as usize);
            bytes.push(handled_size);
        
            match &mut operands[0].value {
                TokenValue::AddressLiteral { value, .. } => bytes.extend(value.to_le_bytes()),
                TokenValue::AddressAtLabel(label) => {
                    label_registry.add_reference(mem::take(label), last_byte_code + bytes.len(), line_number);
                    bytes.extend(LABEL_PLACEHOLDER);
                },
                _ => unreachable!()
            }
        
            match &mut operands[1].value {
                TokenValue::Number { value, .. }=> {
                    let repr = fit_into_bytes(*value, handled_size).unwrap_or_else(
                        || error::number_out_of_range::<u64>(unit_path, value.to_string().as_str(), 10, handled_size, line_number, line)
                    );
                    bytes.extend(repr);
                },
                TokenValue::Label(label) => {
                    label_registry.add_reference(mem::take(label), last_byte_code + bytes.len(), line_number);
                    bytes.extend(LABEL_PLACEHOLDER);
                },
                _ => unreachable!()
            }
        
            bytes
        },

        ByteCodes::MOVE_INTO_ADDR_LITERAL_FROM_ADDR_LITERAL => {
            let mut bytes = ByteCode::with_capacity(1 + ADDRESS_SIZE + ADDRESS_SIZE);
            bytes.push(handled_size);
        
            match &mut operands[0].value {
                TokenValue::AddressLiteral { value, .. } => bytes.extend(value.to_le_bytes()),
                TokenValue::AddressAtLabel(label) => {
                    label_registry.add_reference(mem::take(label), last_byte_code + bytes.len(), line_number);
                    bytes.extend(LABEL_PLACEHOLDER);
                },
                _ => unreachable!()
            }
        
            match &mut operands[1].value {
                TokenValue::AddressLiteral { value, .. } => bytes.extend(value.to_le_bytes()),
                TokenValue::AddressAtLabel(label) => {
                    label_registry.add_reference(mem::take(label), last_byte_code + bytes.len(), line_number);
                    bytes.extend(LABEL_PLACEHOLDER);
                },
                _ => unreachable!()
            }
        
            bytes
        },

        ByteCodes::PUSH_FROM_REG => {
            extract!(operands[0], Register).to_bytes().to_vec()
        },
        
        ByteCodes::PUSH_FROM_ADDR_IN_REG => {
            vec![
                handled_size,
                extract!(operands[0], AddressInRegister) as u8
            ]
        },
        
        ByteCodes::PUSH_FROM_CONST => {
            let mut bytes = ByteCode::with_capacity(1 + handled_size as usize);
            bytes.push(handled_size);
        
            match &mut operands[0].value {
                TokenValue::Number { value, .. }=> {
                    let repr = fit_into_bytes(*value, handled_size).unwrap_or_else(
                        || error::number_out_of_range::<u64>(unit_path, value.to_string().as_str(), 10, handled_size, line_number, line)
                    );
                    bytes.extend(repr);
                },
                TokenValue::Label(label) => {
                    label_registry.add_reference(mem::take(label), last_byte_code + bytes.len(), line_number);
                    bytes.extend(LABEL_PLACEHOLDER);
                },
                _ => unreachable!()
            }
        
            bytes
        },
        
        ByteCodes::PUSH_FROM_ADDR_LITERAL => {
            let mut bytes = ByteCode::with_capacity(1 + ADDRESS_SIZE);
            bytes.push(handled_size);
        
            match &mut operands[0].value {
                TokenValue::AddressLiteral { value, .. } => bytes.extend(value.to_le_bytes()),
                TokenValue::AddressAtLabel(label) => {
                    label_registry.add_reference(mem::take(label), last_byte_code + bytes.len(), line_number);
                    bytes.extend(LABEL_PLACEHOLDER);
                },
                _ => unreachable!()
            }
        
            bytes
        },
        
        ByteCodes::PUSH_STACK_POINTER_REG => {
            vec![
                extract!(operands[0], Register) as u8
            ]
        },
        
        ByteCodes::PUSH_STACK_POINTER_ADDR_IN_REG => {
            vec![
                handled_size,
                extract!(operands[0], AddressInRegister) as u8
            ]
        },
        
        ByteCodes::PUSH_STACK_POINTER_CONST => {
            let mut bytes = ByteCode::with_capacity(1 + handled_size as usize);
            bytes.push(handled_size);
        
            match &mut operands[0].value {
                TokenValue::Number { value, .. }=> {
                    let repr = fit_into_bytes(*value, handled_size).unwrap_or_else(
                        || error::number_out_of_range::<u64>(unit_path, value.to_string().as_str(), 10,  handled_size, line_number, line)
                    );
                    bytes.extend(repr);
                },
                TokenValue::Label(label) => {
                    label_registry.add_reference(mem::take(label), last_byte_code + bytes.len(), line_number);
                    bytes.extend(LABEL_PLACEHOLDER);
                },
                _ => unreachable!()
            }
        
            bytes
        },
        
        ByteCodes::PUSH_STACK_POINTER_ADDR_LITERAL => {
            let mut bytes = ByteCode::with_capacity(1 + ADDRESS_SIZE);
            bytes.push(handled_size);
        
            match &mut operands[0].value {
                TokenValue::AddressLiteral { value, .. } => bytes.extend(value.to_le_bytes()),
                TokenValue::AddressAtLabel(label) => {
                    label_registry.add_reference(mem::take(label), last_byte_code + 1, line_number);
                    bytes.extend(LABEL_PLACEHOLDER);
                },
                _ => unreachable!()
            }
        
            bytes
        },
        
        ByteCodes::POP_INTO_REG => {
            vec![
                handled_size,
                extract!(operands[0], Register) as u8
            ]
        },
        
        ByteCodes::POP_INTO_ADDR_IN_REG => {
            vec![
                handled_size,
                extract!(operands[0], AddressInRegister) as u8
            ]
        },
        
        ByteCodes::POP_INTO_ADDR_LITERAL => {
            let mut bytes = ByteCode::with_capacity(1 + ADDRESS_SIZE);
            bytes.push(handled_size);
        
            match &mut operands[0].value {
                TokenValue::AddressLiteral { value, .. } => bytes.extend(value.to_le_bytes()),
                TokenValue::AddressAtLabel(label) => {
                    label_registry.add_reference(mem::take(label), last_byte_code + bytes.len(), line_number);
                    bytes.extend(LABEL_PLACEHOLDER);
                }
                _ => unreachable!()
            }
        
            bytes
        },
        
        ByteCodes::POP_STACK_POINTER_REG => {
            vec![
                extract!(operands[0], Register) as u8
            ]
        },
        
        ByteCodes::POP_STACK_POINTER_ADDR_IN_REG => {
            vec![
                handled_size,
                extract!(operands[0], AddressInRegister) as u8
            ]
        },
        
        ByteCodes::POP_STACK_POINTER_CONST => {
            let mut bytes = ByteCode::with_capacity(1 + handled_size as usize);
            bytes.push(handled_size);
        
            match &mut operands[0].value {
                TokenValue::Number { value, .. }=> {
                    let repr = fit_into_bytes(*value, handled_size).unwrap_or_else(
                        || error::number_out_of_range::<u64>(unit_path, value.to_string().as_str(), 10,  handled_size, line_number, line)
                    );
                    bytes.extend(repr);
                },
                TokenValue::Label(label) => {
                    label_registry.add_reference(mem::take(label), last_byte_code + bytes.len(), line_number);
                    bytes.extend(LABEL_PLACEHOLDER);
                },
                _ => unreachable!()
            }
        
            bytes
        },
        
        ByteCodes::POP_STACK_POINTER_ADDR_LITERAL => {
            let mut bytes = ByteCode::with_capacity(1 + ADDRESS_SIZE);
            bytes.push(handled_size);
        
            match &mut operands[0].value {
                TokenValue::AddressLiteral { value, .. } => bytes.extend(value.to_le_bytes()),
                TokenValue::AddressAtLabel(label) => {
                    label_registry.add_reference(mem::take(label), last_byte_code + 1, line_number);
                    bytes.extend(LABEL_PLACEHOLDER);
                },
                _ => unreachable!()
            }
        
            bytes
        },
        
        ByteCodes::LABEL => {
            unreachable!()
        },
        
        ByteCodes::JUMP |
        ByteCodes::JUMP_NOT_ZERO |
        ByteCodes::JUMP_ZERO |
        ByteCodes::JUMP_GREATER |
        ByteCodes::JUMP_LESS |
        ByteCodes::JUMP_GREATER_OR_EQUAL |
        ByteCodes::JUMP_LESS_OR_EQUAL |
        ByteCodes::JUMP_CARRY |
        ByteCodes::JUMP_NOT_CARRY |
        ByteCodes::JUMP_OVERFLOW |
        ByteCodes::JUMP_NOT_OVERFLOW |
        ByteCodes::JUMP_SIGN |
        ByteCodes::JUMP_NOT_SIGN |
        ByteCodes::CALL
         => {
            match &mut operands[0].value {
                TokenValue::Label(label) => {
                    label_registry.add_reference(mem::take(label), last_byte_code, line_number);
                    LABEL_PLACEHOLDER.to_vec()
                },
                _ => unreachable!()
            }
        },
        
        
        ByteCodes::COMPARE_REG_REG => {
            vec![
                extract!(operands[0], Register) as u8,
                extract!(operands[1], Register) as u8
            ]
        },
        
        ByteCodes::COMPARE_REG_ADDR_IN_REG => {
            vec![
                handled_size,
                extract!(operands[0], Register) as u8,
                extract!(operands[1], AddressInRegister) as u8
            ]
        },
        
        ByteCodes::COMPARE_REG_CONST => {
            let mut bytes = ByteCode::with_capacity(1 + REGISTER_ID_SIZE + handled_size as usize);
            bytes.push(handled_size);
        
            let left_reg = extract!(operands[0], Register) as u8;
            bytes.push(left_reg);
        
            match &mut operands[1].value {
                TokenValue::Number { value, .. }=> {
                    let repr = fit_into_bytes(*value, handled_size).unwrap_or_else(
                        || error::number_out_of_range::<u64>(unit_path, value.to_string().as_str(), 10,  handled_size, line_number, line)
                    );
                    bytes.extend(repr);
                },
                TokenValue::Label(label) => {
                    label_registry.add_reference(mem::take(label), last_byte_code + bytes.len(), line_number);
                    bytes.extend(LABEL_PLACEHOLDER);
                },
                _ => unreachable!()
            }
        
            bytes
        },
        
        ByteCodes::COMPARE_REG_ADDR_LITERAL => {
            let mut bytes = ByteCode::with_capacity(1 + REGISTER_ID_SIZE + ADDRESS_SIZE);
            bytes.push(handled_size);
        
            let left_reg = extract!(operands[0], Register) as u8;
            bytes.push(left_reg);
        
            match &mut operands[1].value {
                TokenValue::AddressLiteral { value, .. } => bytes.extend(value.to_le_bytes()),
                TokenValue::AddressAtLabel(label) => {
                    label_registry.add_reference(mem::take(label), last_byte_code + bytes.len(), line_number);
                    bytes.extend(LABEL_PLACEHOLDER);
                },
                _ => unreachable!()
            }
        
            bytes
        },
        
        ByteCodes::COMPARE_ADDR_IN_REG_REG => {
            vec![
                handled_size,
                extract!(operands[0], AddressInRegister) as u8,
                extract!(operands[1], Register) as u8
            ]
        },
        
        ByteCodes::COMPARE_ADDR_IN_REG_ADDR_IN_REG => {
            vec![
                handled_size,
                extract!(operands[0], AddressInRegister) as u8,
                extract!(operands[1], AddressInRegister) as u8
            ]
        },
        
        ByteCodes::COMPARE_ADDR_IN_REG_CONST => {
            let mut bytes = ByteCode::with_capacity(1 + REGISTER_ID_SIZE + handled_size as usize);
            bytes.push(handled_size);
        
            let left_reg = extract!(operands[0], AddressInRegister) as u8;
            bytes.push(left_reg);
        
            match &mut operands[1].value {
                TokenValue::Number { value, .. }=> {
                    let repr = fit_into_bytes(*value, handled_size).unwrap_or_else(
                        || error::number_out_of_range::<u64>(unit_path, value.to_string().as_str(), 10,  handled_size, line_number, line)
                    );
                    bytes.extend(repr);
                },
                TokenValue::Label(label) => {
                    label_registry.add_reference(mem::take(label), last_byte_code + bytes.len(), line_number);
                    bytes.extend(LABEL_PLACEHOLDER);
                },
                _ => unreachable!()
            }
        
            bytes
        },
        
        ByteCodes::COMPARE_ADDR_IN_REG_ADDR_LITERAL => {
            let mut bytes = ByteCode::with_capacity(1 + REGISTER_ID_SIZE + ADDRESS_SIZE);
            bytes.push(handled_size);
        
            let left_reg = extract!(operands[0], AddressInRegister) as u8;
            bytes.push(left_reg);
        
            match &mut operands[1].value {
                TokenValue::AddressLiteral { value, .. } => bytes.extend(value.to_le_bytes()),
                TokenValue::AddressAtLabel(label) => {
                    label_registry.add_reference(mem::take(label), last_byte_code + bytes.len(), line_number);
                    bytes.extend(LABEL_PLACEHOLDER);
                },
                _ => unreachable!()
            }
        
            bytes
        },
        
        ByteCodes::COMPARE_CONST_REG => {
            let mut bytes = ByteCode::with_capacity(1 + handled_size as usize + REGISTER_ID_SIZE);
            bytes.push(handled_size);
        
            match &mut operands[0].value {
                TokenValue::Number { value, .. }=> {
                    let repr = fit_into_bytes(*value, handled_size).unwrap_or_else(
                        || error::number_out_of_range::<u64>(unit_path, value.to_string().as_str(), 10, handled_size, line_number, line)
                    );
                    bytes.extend(repr);
                },
                TokenValue::Label(label) => {
                    label_registry.add_reference(mem::take(label), last_byte_code + bytes.len(), line_number);
                    bytes.extend(LABEL_PLACEHOLDER)
                },
                _ => unreachable!()
            }
        
            let right_reg = extract!(operands[1], Register) as u8;
            bytes.push(right_reg);
        
            bytes
        },
        
        ByteCodes::COMPARE_CONST_ADDR_IN_REG => {
            let mut bytes = ByteCode::with_capacity(1 + handled_size as usize + REGISTER_ID_SIZE);
            bytes.push(handled_size);
        
            match &mut operands[0].value {
                TokenValue::Number { value, .. }=> {
                    let repr = fit_into_bytes(*value, handled_size).unwrap_or_else(
                        || error::number_out_of_range::<u64>(unit_path, value.to_string().as_str(), 10,  handled_size, line_number, line)
                    );
                    bytes.extend(repr);
                },
                TokenValue::Label(label) => {
                    label_registry.add_reference(mem::take(label), last_byte_code + bytes.len(), line_number);
                    bytes.extend(LABEL_PLACEHOLDER)
                },
                _ => unreachable!()
            }
        
            let right_reg = extract!(operands[1], AddressInRegister) as u8;
            bytes.push(right_reg);
        
            bytes
        },
        
        ByteCodes::COMPARE_CONST_CONST => {
            let mut bytes = ByteCode::with_capacity(1 + handled_size as usize + handled_size as usize);
            bytes.push(handled_size);
        
            match &mut operands[0].value {
                TokenValue::Number { value, .. }=> {
                    let repr = fit_into_bytes(*value, handled_size).unwrap_or_else(
                        || error::number_out_of_range::<u64>(unit_path, value.to_string().as_str(), 10,  handled_size, line_number, line)
                    );
                    bytes.extend(repr);
                },
                TokenValue::Label(label) => {
                    label_registry.add_reference(mem::take(label), last_byte_code + bytes.len(), line_number);
                    bytes.extend(LABEL_PLACEHOLDER)
                },
                _ => unreachable!()
            }
        
            match &mut operands[1].value {
                TokenValue::Number { value, .. }=> {
                    let repr = fit_into_bytes(*value, handled_size).unwrap_or_else(
                        || error::number_out_of_range::<u64>(unit_path, value.to_string().as_str(), 10,  handled_size, line_number, line)
                    );
                    bytes.extend(repr);
                },
                TokenValue::Label(label) => {
                    label_registry.add_reference(mem::take(label), last_byte_code + bytes.len(), line_number);
                    bytes.extend(LABEL_PLACEHOLDER)
                },
                _ => unreachable!()
            }
        
            bytes
        },
        
        ByteCodes::COMPARE_CONST_ADDR_LITERAL => {
            let mut bytes = ByteCode::with_capacity(1 + handled_size as usize + ADDRESS_SIZE);
            bytes.push(handled_size);
        
            match &mut operands[0].value {
                TokenValue::Number { value, .. }=> {
                    let repr = fit_into_bytes(*value, handled_size).unwrap_or_else(
                        || error::number_out_of_range::<u64>(unit_path, value.to_string().as_str(), 10,  handled_size, line_number, line)
                    );
                    bytes.extend(repr);
                },
                TokenValue::Label(label) => {
                    label_registry.add_reference(mem::take(label), last_byte_code + bytes.len(), line_number);
                    bytes.extend(LABEL_PLACEHOLDER)
                },
                _ => unreachable!()
            }
        
            match &mut operands[1].value {
                TokenValue::AddressLiteral { value, .. } => bytes.extend(value.to_le_bytes()),
                TokenValue::AddressAtLabel(label) => {
                    label_registry.add_reference(mem::take(label), last_byte_code + bytes.len(), line_number);
                    bytes.extend(LABEL_PLACEHOLDER)
                },
                _ => unreachable!()
            }
        
            bytes
        },
        
        ByteCodes::COMPARE_ADDR_LITERAL_REG => {
            let mut bytes = ByteCode::with_capacity(1 + ADDRESS_SIZE + REGISTER_ID_SIZE);
            bytes.push(handled_size);
        
            match &mut operands[0].value {
                TokenValue::AddressLiteral { value, .. } => bytes.extend(value.to_le_bytes()),
                TokenValue::AddressAtLabel(label) => {
                    label_registry.add_reference(mem::take(label), last_byte_code + bytes.len(), line_number);
                    bytes.extend(LABEL_PLACEHOLDER)
                },
                _ => unreachable!()
            }
        
            let right_reg = extract!(operands[1], Register) as u8;
            bytes.push(right_reg);
        
            bytes
        },
        
        ByteCodes::COMPARE_ADDR_LITERAL_ADDR_IN_REG => {
            let mut bytes = ByteCode::with_capacity(1 + ADDRESS_SIZE + REGISTER_ID_SIZE);
            bytes.push(handled_size);
        
            match &mut operands[0].value {
                TokenValue::AddressLiteral { value, .. } => bytes.extend(value.to_le_bytes()),
                TokenValue::AddressAtLabel(label) => {
                    label_registry.add_reference(mem::take(label), last_byte_code + bytes.len(), line_number);
                    bytes.extend(LABEL_PLACEHOLDER)
                },
                _ => unreachable!()
            }
        
            let right_reg = extract!(operands[1], AddressInRegister) as u8;
            bytes.push(right_reg);
        
            bytes
        },
        
        ByteCodes::COMPARE_ADDR_LITERAL_CONST => {
            let mut bytes = ByteCode::with_capacity(1 + ADDRESS_SIZE + handled_size as usize);
            bytes.push(handled_size);
        
            match &mut operands[0].value {
                TokenValue::AddressLiteral { value, .. } => bytes.extend(value.to_le_bytes()),
                TokenValue::AddressAtLabel(label) => {
                    label_registry.add_reference(mem::take(label), last_byte_code + bytes.len(), line_number);
                    bytes.extend(LABEL_PLACEHOLDER)
                },
                _ => unreachable!()
            }
        
            match &mut operands[1].value {
                TokenValue::Number { value, .. } => {
                    let repr = fit_into_bytes(*value, handled_size).unwrap_or_else(
                        || error::number_out_of_range::<u64>(unit_path, value.to_string().as_str(), 10, handled_size, line_number, line)
                    );
                    bytes.extend(repr);
                },
                TokenValue::Label(label) => {
                    label_registry.add_reference(mem::take(label), last_byte_code + bytes.len(), line_number);
                    bytes.extend(LABEL_PLACEHOLDER)
                },
                _ => unreachable!()
            }
        
            bytes
        },
        
        ByteCodes::COMPARE_ADDR_LITERAL_ADDR_LITERAL => {
            let mut bytes = ByteCode::with_capacity(1 + ADDRESS_SIZE + ADDRESS_SIZE);
            bytes.push(handled_size);
        
            match &mut operands[0].value {
                TokenValue::AddressLiteral { value, .. } => bytes.extend(value.to_le_bytes()),
                TokenValue::AddressAtLabel(label) => {
                    label_registry.add_reference(mem::take(label), last_byte_code + bytes.len(), line_number);
                    bytes.extend(LABEL_PLACEHOLDER)
                },
                _ => unreachable!()
            }
        
            match &mut operands[1].value {
                TokenValue::AddressLiteral { value, .. } => bytes.extend(value.to_le_bytes()),
                TokenValue::AddressAtLabel(label) => {
                    label_registry.add_reference(mem::take(label), last_byte_code + bytes.len(), line_number);
                    bytes.extend(LABEL_PLACEHOLDER)
                },
                _ => unreachable!()
            }
        
            bytes
        },
        
        
        ByteCodes::INTERRUPT_REG => {
            vec![
                extract!(operands[0], Register) as u8
            ]
        },
        
        ByteCodes::INTERRUPT_ADDR_IN_REG => {
            vec![
                extract!(operands[0], AddressInRegister) as u8
            ]
        },
        
        ByteCodes::INTERRUPT_CONST => {
            let mut bytes = ByteCode::with_capacity(1);

            match &mut operands[0].value {
                TokenValue::Number { value, .. } => {
                    let repr = fit_into_bytes(*value, 1).unwrap_or_else(
                        || error::number_out_of_range::<u64>(unit_path, value.to_string().as_str(), 10, 1, line_number, line)
                    );
                    bytes.extend(repr);
                },
                _ => unreachable!()
            }
        
            bytes
        },
        
        ByteCodes::INTERRUPT_ADDR_LITERAL => {
            match &mut operands[0].value {
                TokenValue::AddressLiteral { value, .. } => value.to_le_bytes().to_vec(),
                TokenValue::AddressAtLabel(label) => {
                    label_registry.add_reference(mem::take(label), last_byte_code, line_number);
                    LABEL_PLACEHOLDER.to_vec()
                },
                _ => unreachable!()
            }
        },
        
    }
}

