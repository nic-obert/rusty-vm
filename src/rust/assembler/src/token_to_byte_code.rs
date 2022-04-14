use shared::token::{Token, TokenValue};
use bytes::{BytesMut, Bytes};


static SIZE_OF_REGISTER: u8 = 1;
static SIZE_OF_ADDRESS: u8 = 8;


/// Returns the number of bytes needed to represent the number.
pub fn number_size(number: i32) -> usize {
    if number == 0 {
        return 1;
    }

    let size: usize = 0;
    while number != 0 {
        number = number / 256;
        size += 1;
    }

    size
}


/// Returns the bytes representation of the number.
pub fn number_to_bytes(number: i32, size: usize) -> Bytes {
    
    if number_size(number) > size {
        panic!("Number is too big to fit in {} bytes", size);
    }

    let mut value = BytesMut::with_capacity(size);
    for _ in i..size {
        value.put_u8(number % 256);
        number /= 256;
    }

    value.freeze()
}


pub fn sized_operator_bytes_handled(operator: &str) -> u8 {
    /// Returns the number of bytes a sized operator handles.
    /// The output size should always be representable in a single byte.
    operator[operator.len() - 1].to_digit(10).unwrap() as u8
}


/// The following functions are used to convert the operand tokens to bytes.
pub static INSTRUCTION_CONVERSION_TABLE:
    [ fn(&[Token], u8) -> Option<Bytes> ]
= [

    // Arithmetic

    // ByteCodes::ADD
    | operands: &[Token], handled_size: u8 | {
        None
    },
    // ByteCodes::SUB
    | operands: &[Token], handled_size: u8 | {
        None
    },
    // ByteCodes::MUL
    | operands: &[Token], handled_size: u8 | {
        None
    },
    // ByteCodes::DIV
    | operands: &[Token], handled_size: u8 | {
        None
    },
    // ByteCodes::MOD
    | operands: &[Token], handled_size: u8 | {
        None
    },

    // ByteCodes::INC_REG
    | operands: &[Token], handled_size: u8 | {
        let TokenValue::Register(register) = operands[0].value;
        number_to_bytes(register as i32, SIZE_OF_REGISTER)
    },
    // ByteCodes::INC_ADDR_IN_REG
    | operands: &[Token], handled_size: u8 | {
        let TokenValue::Register(register) = operands[0].value;
        let operand_bytes = number_to_bytes(register, SIZE_OF_REGISTER);
        let mut bytes = BytesMut::with_capacity(1 + operand_bytes.len());
        bytes.put_u8(handled_size);
        bytes.put(operand_bytes);
        bytes.freeze()
    },
    // ByteCodes::INC_ADDR_LITERAL
    | operands: &[Token], handled_size: u8 | {
        let TokenValue::Literal(literal) = operands[0].value;
        let operand_bytes = number_to_bytes(literal, SIZE_OF_ADDRESS);
        let mut bytes = BytesMut::with_capacity(1 + operand_bytes.len());
        bytes.put_u8(handled_size);
        bytes.put(operand_bytes);
        bytes.freeze()
    },

    // ByteCodes::DEC_REG
    | operands: &[Token], handled_size: u8 | {
        let TokenValue::Register(register) = operands[0].value;
        number_to_bytes(register as i32, SIZE_OF_REGISTER)
    },
    // ByteCodes::DEC_ADDR_IN_REG
    | operands: &[Token], handled_size: u8 | {
        let TokenValue::Register(register) = operands[0].value;
        let operand_bytes = number_to_bytes(register, SIZE_OF_REGISTER);
        let mut bytes = BytesMut::with_capacity(1 + operand_bytes.len());
        bytes.put_u8(handled_size);
        bytes.put(operand_bytes);
        bytes.freeze()
    },
    // ByteCodes::DEC_ADDR_LITERAL
    | operands: &[Token], handled_size: u8 | {
        let TokenValue::Literal(literal) = operands[0].value;
        let operand_bytes = number_to_bytes(literal, SIZE_OF_ADDRESS);
        let mut bytes = BytesMut::with_capacity(1 + operand_bytes.len());
        bytes.put_u8(handled_size);
        bytes.put(operand_bytes);
        bytes.freeze()
    },

    // No operation
    // ByteCodes::NOP
    | operands: &[Token], handled_size: u8 | {
        None
    },

    // Memory

    // ByteCodes::MOVE_REG_REG
    | operands: &[Token], handled_size: u8 | {
        let TokenValue::Register(reg1) = operands[0].value;
        let TokenValue::Register(reg2) = operands[1].value;
        let mut bytes = BytesMut::with_capacity(2);
        bytes.put_u8(reg1);
        bytes.put_u8(reg2);
        bytes.freeze()
    },

];


