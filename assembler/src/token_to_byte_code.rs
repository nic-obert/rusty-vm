use std::mem;
use std::path::Path;

use rust_vm_lib::assembly::ByteCode;
use rust_vm_lib::registers::REGISTER_ID_SIZE;
use rust_vm_lib::token::{Token, TokenValue};
use rust_vm_lib::byte_code::{ByteCodes, BYTE_CODE_COUNT};
use rust_vm_lib::vm::{ADDRESS_SIZE, Address};

use crate::assembler::{LabelReferenceRegistry, AddLabelReference};
use crate::error;


/// Statically checks if the argument is a valid symbol.
/// 
/// Symbols may be deleted in the code. This macro assures that no function is invalidated because of that.
macro_rules! assert_exists {
    ($x:expr) => {
        #[allow(path_statements)]
        const _: () = { $x ; () };
    };
}


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
    if number == 0 {
        return 1;
    }
    number.to_le_bytes().iter().rev().skip_while(|&&b| b == 0).count()
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


/// The type of a function that converts a list of tokens into byte code.
pub type TokenConverter = fn(Vec<Token>, u8, &mut LabelReferenceRegistry, Address, usize, &Path, &str) -> ByteCode;


/// Return the token converter for the given byte code.
#[inline(always)]
pub fn get_token_converter(byte_code: ByteCodes) -> TokenConverter {
    INSTRUCTION_CONVERSION_TABLE[byte_code as usize]
}


/// Use the token converter to convert the given tokens into byte code.
/// 
/// This function exists to export named parameters instead of a closure to the public interface.
#[inline(always)]
pub fn use_converter(converter: TokenConverter, operands: Vec<Token>, handled_size: u8, label_registry: &mut LabelReferenceRegistry, last_byte_code: Address, line_number: usize, unit_path: &Path, line: &str) -> ByteCode {
    converter(operands, handled_size, label_registry, last_byte_code, line_number, unit_path, line)
}


// Converter functions


fn convert_add (_operands: Vec<Token>, _handled_size: u8, _label_registry: &mut LabelReferenceRegistry, _last_byte_code: Address, _line_number: usize, _unit_path: &Path, _line: &str) -> ByteCode {
    assert_exists!(ByteCodes::ADD);

    Vec::new()
}


fn convert_sub (_operands: Vec<Token>, _handled_size: u8, _label_registry: &mut LabelReferenceRegistry, _last_byte_code: Address, _line_number: usize, _unit_path: &Path, _line: &str) -> ByteCode {
    assert_exists!(ByteCodes::SUB);

    Vec::new()
}


fn convert_mul (_operands: Vec<Token>, _handled_size: u8, _label_registry: &mut LabelReferenceRegistry, _last_byte_code: Address, _line_number: usize, _unit_path: &Path, _line: &str) -> ByteCode {
    assert_exists!(ByteCodes::MUL);

    Vec::new()
}


fn convert_div (_operands: Vec<Token>, _handled_size: u8, _label_registry: &mut LabelReferenceRegistry, _last_byte_code: Address, _line_number: usize, _unit_path: &Path, _line: &str) -> ByteCode {
    assert_exists!(ByteCodes::DIV);
    
    Vec::new()
}


fn convert_mod (_operands: Vec<Token>, _handled_size: u8, _label_registry: &mut LabelReferenceRegistry, _last_byte_code: Address, _line_number: usize, _unit_path: &Path, _line: &str) -> ByteCode {
    assert_exists!(ByteCodes::MOD);
        
    Vec::new()
}


fn convert_inc_reg(operands: Vec<Token>, _handled_size: u8, _label_registry: &mut LabelReferenceRegistry, _last_byte_code: Address, _line_number: usize, _unit_path: &Path, _line: &str) -> ByteCode {
    assert_exists!(ByteCodes::INC_REG);
    
    extract!(operands[0], Register).to_bytes().to_vec()
}


fn convert_inc_addr_in_reg(operands: Vec<Token>, handled_size: u8, _label_registry: &mut LabelReferenceRegistry, _last_byte_code: Address, _line_number: usize, _unit_path: &Path, _line: &str) -> ByteCode {
    assert_exists!(ByteCodes::INC_ADDR_IN_REG);

    vec![
        handled_size,
        extract!(operands[0], AddressInRegister) as u8
    ]
}


fn convert_inc_addr_literal(mut operands: Vec<Token>, handled_size: u8, label_registry: &mut LabelReferenceRegistry, last_byte_code: Address, line_number: usize, _unit_path: &Path, _line: &str) -> ByteCode {
    assert_exists!(ByteCodes::INC_ADDR_LITERAL);

    let mut bytes = Vec::with_capacity(1 + ADDRESS_SIZE);
    bytes.push(handled_size);

    match &mut operands[0].value {
        TokenValue::AddressLiteral(address) => {
            bytes.extend(address.to_le_bytes());
        },
        TokenValue::AddressAtLabel(label) => {
            label_registry.add_reference(mem::take(label), last_byte_code + bytes.len(), line_number);
            bytes.extend(LABEL_PLACEHOLDER);
        },
        _ => unreachable!()
    }

    bytes
}


fn convert_dec_reg(operands: Vec<Token>, _handled_size: u8, _label_registry: &mut LabelReferenceRegistry, _last_byte_code: Address, _line_number: usize, _unit_path: &Path, _line: &str) -> ByteCode {
    assert_exists!(ByteCodes::DEC_REG);
    
    extract!(operands[0], Register).to_bytes().to_vec()
}


fn convert_dec_addr_in_reg(operands: Vec<Token>, handled_size: u8, _label_registry: &mut LabelReferenceRegistry, _last_byte_code: Address, _line_number: usize, _unit_path: &Path, _line: &str) -> ByteCode {
    assert_exists!(ByteCodes::DEC_ADDR_IN_REG);
    
    vec![
        handled_size,
        extract!(operands[0], AddressInRegister) as u8
    ]
}


fn convert_dec_addr_literal(mut operands: Vec<Token>, handled_size: u8, label_registry: &mut LabelReferenceRegistry, last_byte_code: Address, line_number: usize, _unit_path: &Path, _line: &str) -> ByteCode {
    assert_exists!(ByteCodes::DEC_ADDR_LITERAL);

    let mut bytes = Vec::with_capacity(1 + ADDRESS_SIZE);
    bytes.push(handled_size);

    match &mut operands[0].value {
        TokenValue::AddressLiteral(address) => {
            bytes.extend(address.to_le_bytes());
        },
        TokenValue::AddressAtLabel(label) => {
            label_registry.add_reference(mem::take(label), last_byte_code + bytes.len(), line_number);
            bytes.extend(LABEL_PLACEHOLDER);
        },
        _ => unreachable!()
    }

    bytes
}


fn convert_no_operation (_operands: Vec<Token>, _handled_size: u8, _label_registry: &mut LabelReferenceRegistry, _last_byte_code: Address, _line_number: usize, _unit_path: &Path, _line: &str) -> ByteCode {
    assert_exists!(ByteCodes::NO_OPERATION);
    
    Vec::new()
}


fn convert_move_into_reg_from_reg(operands: Vec<Token>, _handled_size: u8, _label_registry: &mut LabelReferenceRegistry, _last_byte_code: Address, _line_number: usize, _unit_path: &Path, _line: &str) -> ByteCode {
    assert_exists!(ByteCodes::MOVE_INTO_REG_FROM_REG);
    
    vec![
        extract!(operands[0], Register) as u8,
        extract!(operands[1], Register) as u8
    ]
}


fn convert_move_into_reg_from_addr_in_reg(operands: Vec<Token>, handled_size: u8, _label_registry: &mut LabelReferenceRegistry, _last_byte_code: Address, _line_number: usize, _unit_path: &Path, _line: &str) -> ByteCode {
    assert_exists!(ByteCodes::MOVE_INTO_REG_FROM_ADDR_IN_REG);
    
    vec![
        handled_size,
        extract!(operands[0], Register) as u8,
        extract!(operands[1], AddressInRegister) as u8
    ]
}


fn convert_move_into_reg_from_const(mut operands: Vec<Token>, handled_size: u8, label_registry: &mut LabelReferenceRegistry, last_byte_code: Address, line_number: usize, unit_path: &Path, line: &str) -> ByteCode {
    assert_exists!(ByteCodes::MOVE_INTO_REG_FROM_CONST);

    let mut bytes = Vec::with_capacity(1 + REGISTER_ID_SIZE + handled_size as usize);
    bytes.push(handled_size);

    let dest_reg = extract!(operands[0], Register) as u8;
    bytes.push(dest_reg as u8);

    match &mut operands[1].value {
        TokenValue::Number(value) => {
            let repr = fit_into_bytes(*value, handled_size).unwrap_or_else(
                || error::number_out_of_range(unit_path, *value, handled_size, line_number, line)
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
}


fn convert_move_into_reg_from_addr_literal(mut operands: Vec<Token>, handled_size: u8, label_registry: &mut LabelReferenceRegistry, last_byte_code: Address, line_number: usize, _unit_path: &Path, _line: &str) -> ByteCode {
    assert_exists!(ByteCodes::MOVE_INTO_REG_FROM_ADDR_LITERAL);

    let mut bytes = Vec::with_capacity(1 + REGISTER_ID_SIZE + ADDRESS_SIZE);
    bytes.push(handled_size);

    let dest_reg = extract!(operands[0], Register) as u8;
    bytes.push(dest_reg as u8);

    match &mut operands[1].value {
        TokenValue::AddressLiteral(address) => bytes.extend(address.to_le_bytes()),
        TokenValue::AddressAtLabel(label) => {
            label_registry.add_reference(mem::take(label), last_byte_code + bytes.len(), line_number);
            bytes.extend(LABEL_PLACEHOLDER);
        },
        _ => unreachable!()
    }
    
    bytes
}


fn convert_move_into_addr_in_reg_from_reg(operands: Vec<Token>, handled_size: u8, _label_registry: &mut LabelReferenceRegistry, _last_byte_code: Address, _line_number: usize, _unit_path: &Path, _line: &str) -> ByteCode {
    assert_exists!(ByteCodes::MOVE_INTO_ADDR_IN_REG_FROM_REG);
    
    vec![
        handled_size,
        extract!(operands[0], AddressInRegister) as u8,
        extract!(operands[1], Register) as u8
    ]
}


fn convert_move_into_addr_in_reg_from_addr_in_reg(operands: Vec<Token>, handled_size: u8, _label_registry: &mut LabelReferenceRegistry, _last_byte_code: Address, _line_number: usize, _unit_path: &Path, _line: &str) -> ByteCode {
    assert_exists!(ByteCodes::MOVE_INTO_ADDR_IN_REG_FROM_ADDR_IN_REG);
    
    vec![
        handled_size,
        extract!(operands[0], AddressInRegister) as u8,
        extract!(operands[1], AddressInRegister) as u8
    ]
}


fn convert_move_into_addr_in_reg_from_const(mut operands: Vec<Token>, handled_size: u8, label_registry: &mut LabelReferenceRegistry, last_byte_code: Address, line_number: usize, unit_path: &Path, line: &str) -> ByteCode {
    assert_exists!(ByteCodes::MOVE_INTO_ADDR_IN_REG_FROM_CONST);

    let mut bytes = Vec::with_capacity(1 + REGISTER_ID_SIZE + handled_size as usize);
    bytes.push(handled_size as u8);

    let dest_reg = extract!(operands[0], AddressInRegister) as u8;
    bytes.push(dest_reg as u8);

    match &mut operands[1].value {
        TokenValue::Number(value) => {
            let repr = fit_into_bytes(*value, handled_size).unwrap_or_else(
                || error::number_out_of_range(unit_path, *value, handled_size, line_number, line)
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
}


fn convert_move_into_addr_in_reg_from_addr_literal(mut operands: Vec<Token>, handled_size: u8, label_registry: &mut LabelReferenceRegistry, last_byte_code: Address, line_number: usize, _unit_path: &Path, _line: &str) -> ByteCode {
    assert_exists!(ByteCodes::MOVE_INTO_ADDR_IN_REG_FROM_ADDR_LITERAL);

    let mut bytes = Vec::with_capacity(1 + REGISTER_ID_SIZE + ADDRESS_SIZE);
    bytes.push(handled_size);

    let dest_reg = extract!(operands[0], AddressInRegister) as u8;
    bytes.push(dest_reg as u8);

    match &mut operands[1].value {
        TokenValue::AddressLiteral(address) => bytes.extend(address.to_le_bytes()),
        TokenValue::AddressAtLabel(label) => {
            label_registry.add_reference(mem::take(label), last_byte_code + bytes.len(), line_number);
            bytes.extend(LABEL_PLACEHOLDER);
        },
        _ => unreachable!()
    }

    bytes
}


fn convert_move_into_addr_literal_from_reg(mut operands: Vec<Token>, handled_size: u8, label_registry: &mut LabelReferenceRegistry, last_byte_code: Address, line_number: usize, _unit_path: &Path, _line: &str) -> ByteCode {
    assert_exists!(ByteCodes::MOVE_INTO_ADDR_LITERAL_FROM_REG);

    let mut bytes = Vec::with_capacity(1 + ADDRESS_SIZE + REGISTER_ID_SIZE);
    bytes.push(handled_size);

    match &mut operands[0].value {
        TokenValue::AddressLiteral(address) => bytes.extend(address.to_le_bytes()),
        TokenValue::AddressAtLabel(label) => {
            label_registry.add_reference(mem::take(label), last_byte_code + bytes.len(), line_number);
            bytes.extend(LABEL_PLACEHOLDER);
        },
        _ => unreachable!()
    }

    let src_reg = extract!(operands[1], Register) as u8;
    bytes.push(src_reg);

    bytes
}


fn convert_move_into_addr_literal_from_addr_in_reg(mut operands: Vec<Token>, handled_size: u8, label_registry: &mut LabelReferenceRegistry, last_byte_code: Address, line_number: usize, _unit_path: &Path, _line: &str) -> ByteCode {
    assert_exists!(ByteCodes::MOVE_INTO_ADDR_LITERAL_FROM_ADDR_IN_REG);

    let mut bytes = Vec::with_capacity(1 + ADDRESS_SIZE + REGISTER_ID_SIZE);
    bytes.push(handled_size);

    match &mut operands[0].value {
        TokenValue::AddressLiteral(address) => bytes.extend(address.to_le_bytes()),
        TokenValue::AddressAtLabel(label) => {
            label_registry.add_reference(mem::take(label), last_byte_code + bytes.len(), line_number);
            bytes.extend(LABEL_PLACEHOLDER);
        },
        _ => unreachable!()
    }

    let src_reg = extract!(operands[1], AddressInRegister) as u8;
    bytes.push(src_reg);

    bytes
}


fn convert_move_into_addr_literal_from_const(mut operands: Vec<Token>, handled_size: u8, label_registry: &mut LabelReferenceRegistry, last_byte_code: Address, line_number: usize, unit_path: &Path, line: &str) -> ByteCode {
    assert_exists!(ByteCodes::MOVE_INTO_ADDR_LITERAL_FROM_CONST);

    let mut bytes = Vec::with_capacity(1 + ADDRESS_SIZE + handled_size as usize);
    bytes.push(handled_size);

    match &mut operands[0].value {
        TokenValue::AddressLiteral(address) => bytes.extend(address.to_le_bytes()),
        TokenValue::AddressAtLabel(label) => {
            label_registry.add_reference(mem::take(label), last_byte_code + bytes.len(), line_number);
            bytes.extend(LABEL_PLACEHOLDER);
        },
        _ => unreachable!()
    }

    match &mut operands[1].value {
        TokenValue::Number(value) => {
            let repr = fit_into_bytes(*value, handled_size).unwrap_or_else(
                || error::number_out_of_range(unit_path, *value, handled_size, line_number, line)
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
}


fn convert_move_into_addr_literal_from_addr_literal(mut operands: Vec<Token>, handled_size: u8, label_registry: &mut LabelReferenceRegistry, last_byte_code: Address, line_number: usize, _unit_path: &Path, _line: &str) -> ByteCode {
    assert_exists!(ByteCodes::MOVE_INTO_ADDR_LITERAL_FROM_ADDR_LITERAL);

    let mut bytes = Vec::with_capacity(1 + ADDRESS_SIZE + ADDRESS_SIZE);
    bytes.push(handled_size);

    match &mut operands[0].value {
        TokenValue::AddressLiteral(address) => bytes.extend(address.to_le_bytes()),
        TokenValue::AddressAtLabel(label) => {
            label_registry.add_reference(mem::take(label), last_byte_code + bytes.len(), line_number);
            bytes.extend(LABEL_PLACEHOLDER);
        },
        _ => unreachable!()
    }

    match &mut operands[1].value {
        TokenValue::AddressLiteral(address) => bytes.extend(address.to_le_bytes()),
        TokenValue::AddressAtLabel(label) => {
            label_registry.add_reference(mem::take(label), last_byte_code + bytes.len(), line_number);
            bytes.extend(LABEL_PLACEHOLDER);
        },
        _ => unreachable!()
    }

    bytes
}


fn convert_push_from_reg(operands: Vec<Token>, _handled_size: u8, _label_registry: &mut LabelReferenceRegistry, _last_byte_code: Address, _line_number: usize, _unit_path: &Path, _line: &str) -> ByteCode {
    assert_exists!(ByteCodes::PUSH_FROM_REG);
    
    extract!(operands[0], Register).to_bytes().to_vec()
}


fn convert_push_from_addr_in_reg(operands: Vec<Token>, handled_size: u8, _label_registry: &mut LabelReferenceRegistry, _last_byte_code: Address, _line_number: usize, _unit_path: &Path, _line: &str) -> ByteCode {
    assert_exists!(ByteCodes::PUSH_FROM_ADDR_IN_REG);
    
    vec![
        handled_size,
        extract!(operands[0], AddressInRegister) as u8
    ]
}


fn convert_push_from_const(mut operands: Vec<Token>, handled_size: u8, label_registry: &mut LabelReferenceRegistry, last_byte_code: Address, line_number: usize, unit_path: &Path, line: &str) -> ByteCode {
    assert_exists!(ByteCodes::PUSH_FROM_CONST);

    let mut bytes = Vec::with_capacity(1 + handled_size as usize);
    bytes.push(handled_size);

    match &mut operands[0].value {
        TokenValue::Number(value) => {
            let repr = fit_into_bytes(*value, handled_size).unwrap_or_else(
                || error::number_out_of_range(unit_path, *value, handled_size, line_number, line)
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
}


fn convert_push_from_addr_literal(mut operands: Vec<Token>, handled_size: u8, label_registry: &mut LabelReferenceRegistry, last_byte_code: Address, line_number: usize, _unit_path: &Path, _line: &str) -> ByteCode {
    assert_exists!(ByteCodes::PUSH_FROM_ADDR_LITERAL);

    let mut bytes = Vec::with_capacity(1 + ADDRESS_SIZE);
    bytes.push(handled_size);

    match &mut operands[0].value {
        TokenValue::AddressLiteral(address) => bytes.extend(address.to_le_bytes()),
        TokenValue::AddressAtLabel(label) => {
            label_registry.add_reference(mem::take(label), last_byte_code + bytes.len(), line_number);
            bytes.extend(LABEL_PLACEHOLDER);
        },
        _ => unreachable!()
    }

    bytes
}


fn convert_pop_into_reg(operands: Vec<Token>, _handled_size: u8, _label_registry: &mut LabelReferenceRegistry, _last_byte_code: Address, _line_number: usize, _unit_path: &Path, _line: &str) -> ByteCode {
    assert_exists!(ByteCodes::POP_INTO_REG);

    extract!(operands[0], Register).to_bytes().to_vec()
}


fn convert_pop_into_addr_in_reg(operands: Vec<Token>, handled_size: u8, _label_registry: &mut LabelReferenceRegistry, _last_byte_code: Address, _line_number: usize, _unit_path: &Path, _line: &str) -> ByteCode {
    vec![
        handled_size,
        extract!(operands[0], AddressInRegister) as u8
    ]
}


fn convert_pop_into_addr_literal(mut operands: Vec<Token>, handled_size: u8, label_registry: &mut LabelReferenceRegistry, last_byte_code: Address, line_number: usize, _unit_path: &Path, _line: &str) -> ByteCode {
    assert_exists!(ByteCodes::POP_INTO_ADDR_LITERAL);    
    
    let mut bytes = Vec::with_capacity(1 + ADDRESS_SIZE);
    bytes.push(handled_size);

    match &mut operands[0].value {
        TokenValue::AddressLiteral(address) => bytes.extend(address.to_le_bytes()),
        TokenValue::AddressAtLabel(label) => {
            label_registry.add_reference(mem::take(label), last_byte_code + bytes.len(), line_number);
            bytes.extend(LABEL_PLACEHOLDER);
        }
        _ => unreachable!()
    }

    bytes
}


/// This is just a placeholder function to keep the indixes valid
fn convert_label (_operands: Vec<Token>, _handled_size: u8, _label_registry: &mut LabelReferenceRegistry, _last_byte_code: Address, _line_number: usize, _unit_path: &Path, _line: &str) -> ByteCode {
    unreachable!()
}


fn convert_jump_to_reg(operands: Vec<Token>, _handled_size: u8, _label_registry: &mut LabelReferenceRegistry, _last_byte_code: Address, _line_number: usize, _unit_path: &Path, _line: &str) -> ByteCode {
    assert_exists!(ByteCodes::JUMP_TO_REG);

    extract!(operands[0], Register).to_bytes().to_vec()
}


fn convert_jump_to_addr_in_reg(operands: Vec<Token>, _handled_size: u8, _label_registry: &mut LabelReferenceRegistry, _last_byte_code: Address, _line_number: usize, _unit_path: &Path, _line: &str) -> ByteCode {
    assert_exists!(ByteCodes::JUMP_TO_ADDR_IN_REG);
    
    extract!(operands[0], AddressInRegister).to_bytes().to_vec()
}


fn convert_jump_to_const(mut operands: Vec<Token>, _handled_size: u8, label_registry: &mut LabelReferenceRegistry, last_byte_code: Address, line_number: usize, unit_path: &Path, line: &str) -> ByteCode {
    assert_exists!(ByteCodes::JUMP_TO_CONST);
    
    match &mut operands[0].value {
        TokenValue::Number(value) => {
            fit_into_bytes(*value, ADDRESS_SIZE as u8).unwrap_or_else(
                || error::number_out_of_range(unit_path, *value, ADDRESS_SIZE as u8, line_number, line)
            )
        },
        TokenValue::Label(label) => {
            label_registry.add_reference(mem::take(label), last_byte_code, line_number);
            LABEL_PLACEHOLDER.to_vec()
        },
        _ => unreachable!()
    }
}


fn convert_jump_to_addr_literal(mut operands: Vec<Token>, _handled_size: u8, label_registry: &mut LabelReferenceRegistry, last_byte_code: Address, line_number: usize, _unit_path: &Path, _line: &str) -> ByteCode {
    assert_exists!(ByteCodes::JUMP_TO_ADDR_LITERAL);
    
    match &mut operands[0].value {
        TokenValue::AddressLiteral(address) => address.to_le_bytes().to_vec(),
        TokenValue::AddressAtLabel(label) => {
            label_registry.add_reference(mem::take(label), last_byte_code, line_number);
            LABEL_PLACEHOLDER.to_vec()
        },
        _ => unreachable!()
    }
}


fn convert_jump_if_not_zero_reg_to_reg(operands: Vec<Token>, _handled_size: u8, _label_registry: &mut LabelReferenceRegistry, _last_byte_code: Address, _line_number: usize, _unit_path: &Path, _line: &str) -> ByteCode {
    assert_exists!(ByteCodes::JUMP_IF_NOT_ZERO_REG_TO_REG);
    
    vec![
        extract!(operands[0], Register) as u8,
        extract!(operands[1], Register) as u8
    ]
}


fn convert_jump_if_not_zero_reg_to_addr_in_reg(operands: Vec<Token>, _handled_size: u8, _label_registry: &mut LabelReferenceRegistry, _last_byte_code: Address, _line_number: usize, _unit_path: &Path, _line: &str) -> ByteCode {
    assert_exists!(ByteCodes::JUMP_IF_NOT_ZERO_REG_TO_ADDR_IN_REG);
    
    vec![
        extract!(operands[0], AddressInRegister) as u8,
        extract!(operands[1], Register) as u8
    ]
}


fn convert_jump_if_not_zero_reg_to_const(mut operands: Vec<Token>, _handled_size: u8, label_registry: &mut LabelReferenceRegistry, last_byte_code: Address, line_number: usize, unit_path: &Path, line: &str) -> ByteCode {
    assert_exists!(ByteCodes::JUMP_IF_NOT_ZERO_REG_TO_CONST);

    let mut bytes = Vec::with_capacity(ADDRESS_SIZE + REGISTER_ID_SIZE);

    match &mut operands[0].value {
        TokenValue::Number(value) => {
            let repr = fit_into_bytes(*value, ADDRESS_SIZE as u8).unwrap_or_else(
                || error::number_out_of_range(unit_path, *value, ADDRESS_SIZE as u8, line_number, line)
            );
            bytes.extend(repr);
        },
        TokenValue::Label(label) => {
            label_registry.add_reference(mem::take(label), last_byte_code + bytes.len(), line_number);
            bytes.extend(LABEL_PLACEHOLDER);
        },
        _ => unreachable!()
    }

    let test_reg = extract!(operands[1], Register) as u8;
    bytes.push(test_reg);

    bytes
}


fn convert_jump_if_not_zero_reg_to_addr_literal(mut operands: Vec<Token>, handled_size: u8, label_registry: &mut LabelReferenceRegistry, last_byte_code: Address, line_number: usize, _unit_path: &Path, _line: &str) -> ByteCode {
    assert_exists!(ByteCodes::JUMP_IF_NOT_ZERO_REG_TO_ADDR_LITERAL);

    let mut bytes = Vec::with_capacity(1 + REGISTER_ID_SIZE + ADDRESS_SIZE);
    bytes.push(handled_size);

    match &mut operands[0].value {
        TokenValue::AddressLiteral(address) => bytes.extend(address.to_le_bytes()),
        TokenValue::AddressAtLabel(label) => {
            label_registry.add_reference(mem::take(label), last_byte_code + bytes.len(), line_number);
            bytes.extend(LABEL_PLACEHOLDER);
        },
        _ => unreachable!()
    }

    let test_reg = extract!(operands[1], Register) as u8;
    bytes.push(test_reg);

    bytes
}


fn convert_jump_if_zero_reg_to_reg(operands: Vec<Token>, _handled_size: u8, _label_registry: &mut LabelReferenceRegistry, _last_byte_code: Address, _line_number: usize, _unit_path: &Path, _line: &str) -> ByteCode {
    assert_exists!(ByteCodes::JUMP_IF_ZERO_REG_TO_REG);
    
    vec![
        extract!(operands[0], Register) as u8,
        extract!(operands[1], Register) as u8
    ]
}


fn convert_jump_if_zero_reg_to_addr_in_reg(operands: Vec<Token>, _handled_size: u8, _label_registry: &mut LabelReferenceRegistry, _last_byte_code: Address, _line_number: usize, _unit_path: &Path, _line: &str) -> ByteCode {
    assert_exists!(ByteCodes::JUMP_IF_ZERO_REG_TO_ADDR_IN_REG);

    vec![
        extract!(operands[0], AddressInRegister) as u8,
        extract!(operands[1], Register) as u8
    ]
}


fn convert_jump_if_zero_reg_to_const(mut operands: Vec<Token>, _handled_size: u8, label_registry: &mut LabelReferenceRegistry, last_byte_code: Address, line_number: usize, unit_path: &Path, line: &str) -> ByteCode {
    assert_exists!(ByteCodes::JUMP_IF_ZERO_REG_TO_CONST);

    let mut bytes = Vec::with_capacity(ADDRESS_SIZE + REGISTER_ID_SIZE);

    match &mut operands[0].value {
        TokenValue::Number(value) => {
            let repr = fit_into_bytes(*value, ADDRESS_SIZE as u8).unwrap_or_else(
                || error::number_out_of_range(unit_path, *value, ADDRESS_SIZE as u8, line_number, line)
            );
            bytes.extend(repr);
        },
        TokenValue::Label(label) => {
            label_registry.add_reference(mem::take(label), last_byte_code + bytes.len(), line_number);
            bytes.extend(LABEL_PLACEHOLDER);
        },
        _ => unreachable!()
    }

    let test_reg = extract!(operands[1], Register) as u8;
    bytes.push(test_reg);

    bytes
}


fn convert_jump_if_zero_reg_to_addr_literal(mut operands: Vec<Token>, handled_size: u8, label_registry: &mut LabelReferenceRegistry, last_byte_code: Address, line_number: usize, _unit_path: &Path, _line: &str) -> ByteCode {
    assert_exists!(ByteCodes::JUMP_IF_ZERO_REG_TO_ADDR_LITERAL);

    let mut bytes = Vec::with_capacity(1 + REGISTER_ID_SIZE + ADDRESS_SIZE);
    bytes.push(handled_size);

    match &mut operands[0].value {
        TokenValue::AddressLiteral(address) => bytes.extend(address.to_le_bytes()),
        TokenValue::AddressAtLabel(label) => {
            label_registry.add_reference(mem::take(label), last_byte_code + bytes.len(), line_number);
            bytes.extend(LABEL_PLACEHOLDER);
        },
        _ => unreachable!()
    }

    let test_reg = extract!(operands[1], Register) as u8;
    bytes.push(test_reg);

    bytes
}


fn convert_compare_reg_reg(operands: Vec<Token>, _handled_size: u8, _label_registry: &mut LabelReferenceRegistry, _last_byte_code: Address, _line_number: usize, _unit_path: &Path, _line: &str) -> ByteCode {
    assert_exists!(ByteCodes::COMPARE_REG_REG);
    
    vec![
        extract!(operands[0], Register) as u8,
        extract!(operands[1], Register) as u8
    ]
}


fn convert_compare_reg_addr_in_reg(operands: Vec<Token>, _handled_size: u8, _label_registry: &mut LabelReferenceRegistry, _last_byte_code: Address, _line_number: usize, _unit_path: &Path, _line: &str) -> ByteCode {
    vec![
        extract!(operands[0], Register) as u8,
        extract!(operands[1], AddressInRegister) as u8
    ]
}


fn convert_compare_reg_const(mut operands: Vec<Token>, handled_size: u8, label_registry: &mut LabelReferenceRegistry, last_byte_code: Address, line_number: usize, unit_path: &Path, line: &str) -> ByteCode {
    assert_exists!(ByteCodes::COMPARE_REG_CONST);

    let mut bytes = Vec::with_capacity(1 + REGISTER_ID_SIZE + handled_size as usize);
    bytes.push(handled_size);

    let left_reg = extract!(operands[0], Register) as u8;
    bytes.push(left_reg);

    match &mut operands[1].value {
        TokenValue::Number(value) => {
            let repr = fit_into_bytes(*value, handled_size).unwrap_or_else(
                || error::number_out_of_range(unit_path, *value, handled_size, line_number, line)
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
}


fn convert_compare_reg_addr_literal(mut operands: Vec<Token>, handled_size: u8, label_registry: &mut LabelReferenceRegistry, last_byte_code: Address, line_number: usize, _unit_path: &Path, _line: &str) -> ByteCode {
    assert_exists!(ByteCodes::COMPARE_REG_ADDR_LITERAL);

    let mut bytes = Vec::with_capacity(1 + REGISTER_ID_SIZE + ADDRESS_SIZE);
    bytes.push(handled_size);

    let left_reg = extract!(operands[0], Register) as u8;
    bytes.push(left_reg);

    match &mut operands[1].value {
        TokenValue::AddressLiteral(address) => bytes.extend(address.to_le_bytes()),
        TokenValue::AddressAtLabel(label) => {
            label_registry.add_reference(mem::take(label), last_byte_code + bytes.len(), line_number);
            bytes.extend(LABEL_PLACEHOLDER);
        },
        _ => unreachable!()
    }

    bytes
}


fn convert_compare_addr_in_reg_reg(operands: Vec<Token>, _handled_size: u8, _label_registry: &mut LabelReferenceRegistry, _last_byte_code: Address, _line_number: usize, _unit_path: &Path, _line: &str) -> ByteCode {
    assert_exists!(ByteCodes::COMPARE_ADDR_IN_REG_REG);
    
    vec![
        extract!(operands[0], AddressInRegister) as u8,
        extract!(operands[1], Register) as u8
    ]
}


fn convert_compare_addr_in_reg_addr_in_reg(operands: Vec<Token>, _handled_size: u8, _label_registry: &mut LabelReferenceRegistry, _last_byte_code: Address, _line_number: usize, _unit_path: &Path, _line: &str) -> ByteCode {
    assert_exists!(ByteCodes::COMPARE_ADDR_IN_REG_ADDR_IN_REG);
    
    vec![
        extract!(operands[0], AddressInRegister) as u8,
        extract!(operands[1], AddressInRegister) as u8
    ]
}


fn convert_compare_addr_in_reg_const(mut operands: Vec<Token>, handled_size: u8, label_registry: &mut LabelReferenceRegistry, last_byte_code: Address, line_number: usize, unit_path: &Path, line: &str) -> ByteCode {
    assert_exists!(ByteCodes::COMPARE_ADDR_IN_REG_CONST);

    let mut bytes = Vec::with_capacity(1 + REGISTER_ID_SIZE + handled_size as usize);
    bytes.push(handled_size);

    let left_reg = extract!(operands[0], AddressInRegister) as u8;
    bytes.push(left_reg);

    match &mut operands[1].value {
        TokenValue::Number(value) => {
            let repr = fit_into_bytes(*value, handled_size).unwrap_or_else(
                || error::number_out_of_range(unit_path, *value, handled_size, line_number, line)
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
}


fn convert_compare_addr_in_reg_addr_literal(mut operands: Vec<Token>, handled_size: u8, label_registry: &mut LabelReferenceRegistry, last_byte_code: Address, line_number: usize, _unit_path: &Path, _line: &str) -> ByteCode {
    assert_exists!(ByteCodes::COMPARE_ADDR_IN_REG_ADDR_LITERAL);

    let mut bytes = Vec::with_capacity(1 + REGISTER_ID_SIZE + ADDRESS_SIZE);
    bytes.push(handled_size);

    let left_reg = extract!(operands[0], AddressInRegister) as u8;
    bytes.push(left_reg);

    match &mut operands[1].value {
        TokenValue::AddressLiteral(address) => bytes.extend(address.to_le_bytes()),
        TokenValue::AddressAtLabel(label) => {
            label_registry.add_reference(mem::take(label), last_byte_code + bytes.len(), line_number);
            bytes.extend(LABEL_PLACEHOLDER);
        },
        _ => unreachable!()
    }

    bytes
}


fn convert_compare_const_reg(mut operands: Vec<Token>, handled_size: u8, label_registry: &mut LabelReferenceRegistry, last_byte_code: Address, line_number: usize, unit_path: &Path, line: &str) -> ByteCode {
    assert_exists!(ByteCodes::COMPARE_CONST_REG);

    let mut bytes = Vec::with_capacity(1 + handled_size as usize + REGISTER_ID_SIZE);
    bytes.push(handled_size);

    match &mut operands[0].value {
        TokenValue::Number(value) => {
            let repr = fit_into_bytes(*value, handled_size).unwrap_or_else(
                || error::number_out_of_range(unit_path, *value, handled_size, line_number, line)
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
}


fn convert_compare_const_addr_in_reg(mut operands: Vec<Token>, handled_size: u8, label_registry: &mut LabelReferenceRegistry, last_byte_code: Address, line_number: usize, unit_path: &Path, line: &str) -> ByteCode {
    assert_exists!(ByteCodes::COMPARE_CONST_ADDR_IN_REG);

    let mut bytes = Vec::with_capacity(1 + handled_size as usize + REGISTER_ID_SIZE);
    bytes.push(handled_size);

    match &mut operands[0].value {
        TokenValue::Number(value) => {
            let repr = fit_into_bytes(*value, handled_size).unwrap_or_else(
                || error::number_out_of_range(unit_path, *value, handled_size, line_number, line)
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
}


fn convert_compare_const_const(mut operands: Vec<Token>, handled_size: u8, label_registry: &mut LabelReferenceRegistry, last_byte_code: Address, line_number: usize, unit_path: &Path, line: &str) -> ByteCode {
    assert_exists!(ByteCodes::COMPARE_CONST_CONST);

    let mut bytes = Vec::with_capacity(1 + handled_size as usize + handled_size as usize);
    bytes.push(handled_size);

    match &mut operands[0].value {
        TokenValue::Number(value) => {
            let repr = fit_into_bytes(*value, handled_size).unwrap_or_else(
                || error::number_out_of_range(unit_path, *value, handled_size, line_number, line)
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
        TokenValue::Number(value) => {
            let repr = fit_into_bytes(*value, handled_size).unwrap_or_else(
                || error::number_out_of_range(unit_path, *value, handled_size, line_number, line)
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
}


fn convert_compare_const_addr_literal(mut operands: Vec<Token>, handled_size: u8, label_registry: &mut LabelReferenceRegistry, last_byte_code: Address, line_number: usize, unit_path: &Path, line: &str) -> ByteCode {
    assert_exists!(ByteCodes::COMPARE_CONST_ADDR_LITERAL);

    let mut bytes = Vec::with_capacity(1 + handled_size as usize + ADDRESS_SIZE);
    bytes.push(handled_size);

    match &mut operands[0].value {
        TokenValue::Number(value) => {
            let repr = fit_into_bytes(*value, handled_size).unwrap_or_else(
                || error::number_out_of_range(unit_path, *value, handled_size, line_number, line)
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
        TokenValue::AddressLiteral(address) => bytes.extend(address.to_le_bytes()),
        TokenValue::AddressAtLabel(label) => {
            label_registry.add_reference(mem::take(label), last_byte_code + bytes.len(), line_number);
            bytes.extend(LABEL_PLACEHOLDER)
        },
        _ => unreachable!()
    }

    bytes
}


fn convert_compare_addr_literal_reg(mut operands: Vec<Token>, handled_size: u8, label_registry: &mut LabelReferenceRegistry, last_byte_code: Address, line_number: usize, _unit_path: &Path, _line: &str) -> ByteCode {
    assert_exists!(ByteCodes::COMPARE_ADDR_LITERAL_REG);

    let mut bytes = Vec::with_capacity(1 + ADDRESS_SIZE + REGISTER_ID_SIZE);
    bytes.push(handled_size);

    match &mut operands[0].value {
        TokenValue::AddressLiteral(address) => bytes.extend(address.to_le_bytes()),
        TokenValue::AddressAtLabel(label) => {
            label_registry.add_reference(mem::take(label), last_byte_code + bytes.len(), line_number);
            bytes.extend(LABEL_PLACEHOLDER)
        },
        _ => unreachable!()
    }

    let right_reg = extract!(operands[1], Register) as u8;
    bytes.push(right_reg);

    bytes
}


fn convert_compare_addr_literal_addr_in_reg(mut operands: Vec<Token>, handled_size: u8, label_registry: &mut LabelReferenceRegistry, last_byte_code: Address, line_number: usize, _unit_path: &Path, _line: &str) -> ByteCode {
    assert_exists!(ByteCodes::COMPARE_ADDR_LITERAL_ADDR_IN_REG);

    let mut bytes = Vec::with_capacity(1 + ADDRESS_SIZE + REGISTER_ID_SIZE);
    bytes.push(handled_size);

    match &mut operands[0].value {
        TokenValue::AddressLiteral(address) => bytes.extend(address.to_le_bytes()),
        TokenValue::AddressAtLabel(label) => {
            label_registry.add_reference(mem::take(label), last_byte_code + bytes.len(), line_number);
            bytes.extend(LABEL_PLACEHOLDER)
        },
        _ => unreachable!()
    }

    let right_reg = extract!(operands[1], AddressInRegister) as u8;
    bytes.push(right_reg);

    bytes
}


fn convert_compare_addr_literal_const(mut operands: Vec<Token>, handled_size: u8, label_registry: &mut LabelReferenceRegistry, last_byte_code: Address, line_number: usize, unit_path: &Path, line: &str) -> ByteCode {
    assert_exists!(ByteCodes::COMPARE_ADDR_LITERAL_CONST);

    let mut bytes = Vec::with_capacity(1 + ADDRESS_SIZE + handled_size as usize);
    bytes.push(handled_size);

    match &mut operands[0].value {
        TokenValue::AddressLiteral(address) => bytes.extend(address.to_le_bytes()),
        TokenValue::AddressAtLabel(label) => {
            label_registry.add_reference(mem::take(label), last_byte_code + bytes.len(), line_number);
            bytes.extend(LABEL_PLACEHOLDER)
        },
        _ => unreachable!()
    }

    match &mut operands[1].value {
        TokenValue::Number(value) => {
            let repr = fit_into_bytes(*value, handled_size).unwrap_or_else(
                || error::number_out_of_range(unit_path, *value, handled_size, line_number, line)
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
}


fn convert_compare_addr_literal_addr_literal (mut operands: Vec<Token>, handled_size: u8, label_registry: &mut LabelReferenceRegistry, last_byte_code: Address, line_number: usize, _unit_path: &Path, _line: &str) -> ByteCode {
    assert_exists!(ByteCodes::COMPARE_ADDR_LITERAL_ADDR_LITERAL);

    let mut bytes = Vec::with_capacity(1 + ADDRESS_SIZE + ADDRESS_SIZE);
    bytes.push(handled_size);

    match &mut operands[0].value {
        TokenValue::AddressLiteral(address) => bytes.extend(address.to_le_bytes()),
        TokenValue::AddressAtLabel(label) => {
            label_registry.add_reference(mem::take(label), last_byte_code + bytes.len(), line_number);
            bytes.extend(LABEL_PLACEHOLDER)
        },
        _ => unreachable!()
    }

    match &mut operands[1].value {
        TokenValue::AddressLiteral(address) => bytes.extend(address.to_le_bytes()),
        TokenValue::AddressAtLabel(label) => {
            label_registry.add_reference(mem::take(label), last_byte_code + bytes.len(), line_number);
            bytes.extend(LABEL_PLACEHOLDER)
        },
        _ => unreachable!()
    }

    bytes
}


fn convert_print_signed (_operands: Vec<Token>, _handled_size: u8, _label_registry: &mut LabelReferenceRegistry, _last_byte_codee: Address, _line_number: usize, _unit_path: &Path, _line: &str) -> ByteCode {
    assert_exists!(ByteCodes::PRINT_SIGNED);

    Vec::new()
}


fn convert_print_unsigned (_operands: Vec<Token>, _handled_size: u8, _label_registry: &mut LabelReferenceRegistry, _last_byte_codee: Address, _line_number: usize, _unit_path: &Path, _line: &str) -> ByteCode {
    assert_exists!(ByteCodes::PRINT_UNSIGNED);

    Vec::new()
}

fn convert_print_char (_operands: Vec<Token>, _handled_size: u8, _label_registry: &mut LabelReferenceRegistry, _last_byte_codee: Address, _line_number: usize, _unit_path: &Path, _line: &str) -> ByteCode {
    assert_exists!(ByteCodes::PRINT_CHAR);

    Vec::new()
}


// ByteCodes::PRINT_STRING
fn convert_print_string (_operands: Vec<Token>, _handled_size: u8, _label_registry: &mut LabelReferenceRegistry, _last_byte_code: Address, _line_number: usize, _unit_path: &Path, _line: &str) -> ByteCode {
    assert_exists!(ByteCodes::PRINT_STRING);

    Vec::new()
}


// ByteCodes::INPUT_INT
fn convert_input_int (_operands: Vec<Token>, _handled_size: u8, _label_registry: &mut LabelReferenceRegistry, _last_byte_code: Address, _line_number: usize, _unit_path: &Path, _line: &str) -> ByteCode {
    assert_exists!(ByteCodes::INPUT_INT);
    
    Vec::new()
}


// ByteCodes::INPUT_STRING
fn convert_input_string (_operands: Vec<Token>, _handled_size: u8, _label_registry: &mut LabelReferenceRegistry, _last_byte_code: Address, _line_number: usize, _unit_path: &Path, _line: &str) -> ByteCode {
    assert_exists!(ByteCodes::INPUT_STRING);

    Vec::new()
}


// ByteCodes::EXIT
fn convert_exit (_operands: Vec<Token>, _handled_size: u8, _label_registry: &mut LabelReferenceRegistry, _last_byte_code: Address, _line_number: usize, _unit_path: &Path, _line: &str) -> ByteCode {
    assert_exists!(ByteCodes::EXIT);

    Vec::new()
}


/// The following functions are used to convert the operand tokens to bytes.
const INSTRUCTION_CONVERSION_TABLE: [ TokenConverter; BYTE_CODE_COUNT ] = [

    convert_add,
    convert_sub,
    convert_mul,
    convert_div,
    convert_mod,

    convert_inc_reg,
    convert_inc_addr_in_reg,
    convert_inc_addr_literal,

    convert_dec_reg,
    convert_dec_addr_in_reg,
    convert_dec_addr_literal,

    convert_no_operation,

    convert_move_into_reg_from_reg,
    convert_move_into_reg_from_addr_in_reg,
    convert_move_into_reg_from_const,
    convert_move_into_reg_from_addr_literal,
    convert_move_into_addr_in_reg_from_reg,
    convert_move_into_addr_in_reg_from_addr_in_reg,
    convert_move_into_addr_in_reg_from_const,
    convert_move_into_addr_in_reg_from_addr_literal,
    convert_move_into_addr_literal_from_reg,
    convert_move_into_addr_literal_from_addr_in_reg,
    convert_move_into_addr_literal_from_const,
    convert_move_into_addr_literal_from_addr_literal,

    convert_push_from_reg,
    convert_push_from_addr_in_reg,
    convert_push_from_const,
    convert_push_from_addr_literal,

    convert_pop_into_reg,
    convert_pop_into_addr_in_reg,
    convert_pop_into_addr_literal,

    // This is just a placeholder to make indices work
    convert_label,

    convert_jump_to_reg,
    convert_jump_to_addr_in_reg,
    convert_jump_to_const,
    convert_jump_to_addr_literal,

    convert_jump_if_not_zero_reg_to_reg,
    convert_jump_if_not_zero_reg_to_addr_in_reg,
    convert_jump_if_not_zero_reg_to_const,
    convert_jump_if_not_zero_reg_to_addr_literal,

    convert_jump_if_zero_reg_to_reg,
    convert_jump_if_zero_reg_to_addr_in_reg,
    convert_jump_if_zero_reg_to_const,
    convert_jump_if_zero_reg_to_addr_literal,

    convert_compare_reg_reg,
    convert_compare_reg_addr_in_reg,
    convert_compare_reg_const,
    convert_compare_reg_addr_literal,
    convert_compare_addr_in_reg_reg,
    convert_compare_addr_in_reg_addr_in_reg,
    convert_compare_addr_in_reg_const,
    convert_compare_addr_in_reg_addr_literal,
    convert_compare_const_reg,
    convert_compare_const_addr_in_reg,
    convert_compare_const_const,
    convert_compare_const_addr_literal,
    convert_compare_addr_literal_reg,
    convert_compare_addr_literal_addr_in_reg,
    convert_compare_addr_literal_const,
    convert_compare_addr_literal_addr_literal,

    convert_print_signed,
    convert_print_unsigned,
    convert_print_char,
    convert_print_string,

    convert_input_int,
    convert_input_string,

    convert_exit,

];

