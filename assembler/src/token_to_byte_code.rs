use crate::token::{Token, TokenValue};
use bytes::{BytesMut, Bytes, BufMut};


static SIZE_OF_REGISTER: usize = 1;
static SIZE_OF_ADDRESS: usize = 8;


/// Returns the number of bytes needed to represent the number.
pub fn number_size(mut number: u64) -> usize {
    if number == 0 {
        return 1;
    }

    let mut size: usize = 0;
    while number != 0 {
        number = number / 256;
        size += 1;
    }

    size
}


/// Returns the bytes representation of the number.
pub fn number_to_bytes(mut number: u64, size: usize) -> Bytes {
    
    if number_size(number) > size {
        panic!("Number is too big to fit in {} bytes", size);
    }

    let mut value = BytesMut::with_capacity(size);
    for _ in 0..size {
        value.put_u8((number % 256).try_into().unwrap());
        number /= 256;
    }

    value.freeze()
}


/// Returns the number of bytes a sized operator handles.
/// The output size should always be representable in a single byte.
pub fn sized_operator_bytes_handled(operator: &str) -> u8 {
    operator.chars().nth(operator.len() - 1).unwrap().to_digit(10).unwrap() as u8
}


/// The following functions are used to convert the operand tokens to bytes.
pub static INSTRUCTION_CONVERSION_TABLE:
    [ fn(&[Token], u8) -> Option<Bytes>; 13 ]
= [

    // Arithmetic

    // ByteCodes::ADD
    | _operands: &[Token], _handled_size: u8 | {
        None
    },
    // ByteCodes::SUB
    | _operands: &[Token], _handled_size: u8 | {
        None
    },
    // ByteCodes::MUL
    | _operands: &[Token], _handled_size: u8 | {
        None
    },
    // ByteCodes::DIV
    | _operands: &[Token], _handled_size: u8 | {
        None
    },
    // ByteCodes::MOD
    | _operands: &[Token], _handled_size: u8 | {
        None
    },

    // ByteCodes::INC_REG
    | operands: &[Token], _handled_size: u8 | {
        let TokenValue::Register(register) = operands[0].value;
        Some(number_to_bytes(register as u64, SIZE_OF_REGISTER))
    },
    // ByteCodes::INC_ADDR_IN_REG
    | operands: &[Token], handled_size: u8 | {
        let TokenValue::Register(register) = operands[0].value;
        let operand_bytes = number_to_bytes(register as u64, SIZE_OF_REGISTER);
        let mut bytes = BytesMut::with_capacity(1 + operand_bytes.len());
        bytes.put_u8(handled_size);
        bytes.put(operand_bytes);
        Some(bytes.freeze())
    },
    // ByteCodes::INC_ADDR_LITERAL
    | operands: &[Token], handled_size: u8 | {
        let TokenValue::AddressLiteral(literal) = operands[0].value;
        let operand_bytes = number_to_bytes(literal, SIZE_OF_ADDRESS);
        let mut bytes = BytesMut::with_capacity(1 + operand_bytes.len());
        bytes.put_u8(handled_size);
        bytes.put(operand_bytes);
        Some(bytes.freeze())
    },

    // ByteCodes::DEC_REG
    | operands: &[Token], handled_size: u8 | {
        let TokenValue::Register(register) = operands[0].value;
        Some(number_to_bytes(register as u64, SIZE_OF_REGISTER))
    },
    // ByteCodes::DEC_ADDR_IN_REG
    | operands: &[Token], handled_size: u8 | {
        let TokenValue::Register(register) = operands[0].value;
        let operand_bytes = number_to_bytes(register as u64, SIZE_OF_REGISTER);
        let mut bytes = BytesMut::with_capacity(1 + operand_bytes.len());
        bytes.put_u8(handled_size);
        bytes.put(operand_bytes);
        Some(bytes.freeze())
    },
    // ByteCodes::DEC_ADDR_LITERAL
    | operands: &[Token], handled_size: u8 | {
        let TokenValue::AddressLiteral(literal) = operands[0].value;
        let operand_bytes = number_to_bytes(literal, SIZE_OF_ADDRESS);
        let mut bytes = BytesMut::with_capacity(1 + operand_bytes.len());
        bytes.put_u8(handled_size);
        bytes.put(operand_bytes);
        Some(bytes.freeze())
    },

    // No operation
    // ByteCodes::NOP
    | _operands: &[Token], _handled_size: u8 | {
        None
    },

    // Memory

    // ByteCodes::MOVE_REG_REG
    | operands: &[Token], _handled_size: u8 | {
        let TokenValue::Register(reg1) = operands[0].value;
        let TokenValue::Register(reg2) = operands[1].value;
        let mut bytes = BytesMut::with_capacity(2);
        bytes.put_u8(reg1 as u8);
        bytes.put_u8(reg2 as u8);
        Some(bytes.freeze())
    },

];


