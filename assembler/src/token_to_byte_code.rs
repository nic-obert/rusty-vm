use rust_vm_lib::token::{Token, TokenValue};
use rust_vm_lib::byte_code::ByteCodes;
use bytes::{BytesMut, Bytes, BufMut};
use std::mem::size_of;


const SIZE_OF_REGISTER: usize = 1;
const SIZE_OF_ADDRESS: usize = 8;


/// Returns the number of bytes needed to represent the number.
pub const fn number_size(mut number: u64) -> usize {
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


/// The following functions are used to convert the operand tokens to bytes.
pub const INSTRUCTION_CONVERSION_TABLE:
    [ fn(&[Token], u8) -> Option<Bytes>; 44 ]
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
        if let TokenValue::Register(dest_reg) = operands[0].value {
            return Some(number_to_bytes(dest_reg as u64, SIZE_OF_REGISTER));
        }
        panic!("Invalid operands for instruction {}: {:#?}", ByteCodes::INC_REG, operands);
    },
    // ByteCodes::INC_ADDR_IN_REG
    | operands: &[Token], handled_size: u8 | {
        if let TokenValue::Register(dest_reg) = operands[0].value {
            let operand_bytes = number_to_bytes(dest_reg as u64, SIZE_OF_REGISTER);
            let mut bytes = BytesMut::with_capacity(1 + operand_bytes.len());
            bytes.put_u8(handled_size);
            bytes.put(operand_bytes);
            return Some(bytes.freeze());
        }
        panic!("Invalid operands for instruction {}: {:#?}", ByteCodes::INC_ADDR_IN_REG, operands);
    },
    // ByteCodes::INC_ADDR_LITERAL
    | operands: &[Token], handled_size: u8 | {
        if let TokenValue::AddressLiteral(address) = operands[0].value {
            let operand_bytes = number_to_bytes(address as u64, SIZE_OF_ADDRESS);
            let mut bytes = BytesMut::with_capacity(1 + operand_bytes.len());
            bytes.put_u8(handled_size);
            bytes.put(operand_bytes);
            return Some(bytes.freeze());
        }
        panic!("Invalid operands for instruction {}: {:#?}", ByteCodes::INC_ADDR_LITERAL, operands);
    },

    // ByteCodes::DEC_REG
    | operands: &[Token], _handled_size: u8 | {
        if let TokenValue::Register(dest_reg) = operands[0].value {
            return Some(number_to_bytes(dest_reg as u64, SIZE_OF_REGISTER));
        } 
        panic!("Invalid operands for instruction {}: {:#?}", ByteCodes::DEC_REG, operands);
    },
    // ByteCodes::DEC_ADDR_IN_REG
    | operands: &[Token], handled_size: u8 | {
        if let TokenValue::Register(dest_reg) = operands[0].value {
            let operand_bytes = number_to_bytes(dest_reg as u64, SIZE_OF_REGISTER);
            let mut bytes = BytesMut::with_capacity(1 + operand_bytes.len());
            bytes.put_u8(handled_size);
            bytes.put(operand_bytes);
            return Some(bytes.freeze());
        }
        panic!("Invalid operands for instruction {}: {:#?}", ByteCodes::DEC_ADDR_IN_REG, operands);
    },
    // ByteCodes::DEC_ADDR_LITERAL
    | operands: &[Token], handled_size: u8 | {
        if let TokenValue::AddressLiteral(address) = operands[0].value {
            let operand_bytes = number_to_bytes(address as u64, SIZE_OF_ADDRESS);
            let mut bytes = BytesMut::with_capacity(1 + operand_bytes.len());
            bytes.put_u8(handled_size);
            bytes.put(operand_bytes);
            return Some(bytes.freeze());
        } 
        panic!("Invalid operands for instruction {}: {:#?}", ByteCodes::DEC_ADDR_LITERAL, operands);
    },

    // No operation
    // ByteCodes::NOP
    | _operands: &[Token], _handled_size: u8 | {
        None
    },

    // Memory

    // ByteCodes::MOVE_INTO_REG_FROM_REG
    | operands: &[Token], _handled_size: u8 | {
        if let TokenValue::Register(dest_reg) = operands[0].value {
            if let TokenValue::Register(src_reg) = operands[1].value {
                let mut bytes = BytesMut::with_capacity(2);
                bytes.put_u8(dest_reg as u8);
                bytes.put_u8(src_reg as u8);
                return Some(bytes.freeze());
            }
        }
        panic!("Invalid operands for instruction {}: {:#?}", ByteCodes::MOVE_INTO_REG_FROM_REG, operands);
    },

    // ByteCodes::MOVE_INTO_REG_FROM_ADDR_IN_REG
    | operands: &[Token], handled_size: u8 | {
        if let TokenValue::Register(dest_reg) = operands[0].value {
            if let TokenValue::Register(src_reg) = operands[1].value {
                let mut bytes = BytesMut::with_capacity(3);
                bytes.put_u8(handled_size);
                bytes.put_u8(dest_reg as u8);
                bytes.put_u8(src_reg as u8);
                return Some(bytes.freeze());
            }
        }
        panic!("Invalid operands for instruction {}: {:#?}", ByteCodes::MOVE_INTO_REG_FROM_ADDR_IN_REG, operands);
    },

    // ByteCodes::MOVE_INTO_REG_FROM_CONST
    | operands: &[Token], handled_size: u8 | {
        if let TokenValue::Register(dest_reg) = operands[0].value {
            if let TokenValue::Number(address) = operands[1].value {
                let mut bytes = BytesMut::with_capacity(3);
                bytes.put_u8(handled_size);
                bytes.put_u8(dest_reg as u8);
                bytes.put_u64(address as u64);
                return Some(bytes.freeze());
            }
        }
        panic!("Invalid operands for instruction {}: {:#?}", ByteCodes::MOVE_INTO_REG_FROM_CONST, operands);
    },

    // ByteCodes::MOVE_INTO_REG_FROM_ADDR_LITERAL
    | operands: &[Token], handled_size: u8 | {
        if let TokenValue::Register(dest_reg) = operands[0].value {
            if let TokenValue::AddressLiteral(address) = operands[1].value {
                let mut bytes = BytesMut::with_capacity(2 + SIZE_OF_ADDRESS);
                bytes.put_u8(handled_size);
                bytes.put_u8(dest_reg as u8);
                bytes.put_u64(address as u64);
                return Some(bytes.freeze());
            }
        }
        panic!("Invalid operands for instruction {}: {:#?}", ByteCodes::MOVE_INTO_REG_FROM_ADDR_LITERAL, operands);
    },

    // ByteCodes::MOVE_INTO_ADDR_IN_REG_FROM_REG
    | operands: &[Token], handled_size: u8 | {
        if let TokenValue::Register(dest_reg) = operands[0].value {
            if let TokenValue::Register(src_reg) = operands[1].value {
                let mut bytes = BytesMut::with_capacity(3);
                bytes.put_u8(handled_size);
                bytes.put_u8(dest_reg as u8);
                bytes.put_u8(src_reg as u8);
                return Some(bytes.freeze());
            }
        }
        panic!("Invalid operands for instruction {}: {:#?}", ByteCodes::MOVE_INTO_ADDR_IN_REG_FROM_REG, operands);
    },

    // ByteCodes::MOVE_INTO_ADDR_IN_REG_FROM_ADDR_IN_REG
    | operands: &[Token], handled_size: u8 | {
        if let TokenValue::Register(dest_reg) = operands[0].value {
            if let TokenValue::Register(src_reg) = operands[1].value {
                let mut bytes = BytesMut::with_capacity(3);
                bytes.put_u8(handled_size);
                bytes.put_u8(dest_reg as u8);
                bytes.put_u8(src_reg as u8);
                return Some(bytes.freeze());
            }
        }
        panic!("Invalid operands for instruction {}: {:#?}", ByteCodes::MOVE_INTO_ADDR_IN_REG_FROM_ADDR_IN_REG, operands);
    },

    // ByteCodes::MOVE_INTO_ADDR_IN_REG_FROM_CONST
    | operands: &[Token], handled_size: u8 | {
        if let TokenValue::Register(dest_reg) = operands[0].value {
            if let TokenValue::Number(value) = operands[1].value {
                let mut bytes = BytesMut::with_capacity(2 + SIZE_OF_ADDRESS);
                bytes.put_u8(handled_size);
                bytes.put_u8(dest_reg as u8);
                bytes.put_u64(value as u64);
                return Some(bytes.freeze());
            }
        }
        panic!("Invalid operands for instruction {}: {:#?}", ByteCodes::MOVE_INTO_ADDR_IN_REG_FROM_CONST, operands);
    },

    // ByteCodes::MOVE_INTO_ADDR_IN_REG_FROM_ADDR_LITERAL
    | operands: &[Token], handled_size: u8 | {
        if let TokenValue::Register(dest_reg) = operands[0].value {
            if let TokenValue::AddressLiteral(value) = operands[1].value {
                let mut bytes = BytesMut::with_capacity(2 + SIZE_OF_ADDRESS);
                bytes.put_u8(handled_size);
                bytes.put_u8(dest_reg as u8);
                bytes.put_u64(value as u64);
                return Some(bytes.freeze());
            }
        }
        panic!("Invalid operands for instruction {}: {:#?}", ByteCodes::MOVE_INTO_ADDR_IN_REG_FROM_ADDR_LITERAL, operands);
    },

    // ByteCodes::MOVE_INTO_ADDR_LITERAL_FROM_REG
    | operands: &[Token], handled_size: u8 | {
        if let TokenValue::AddressLiteral(dest_address) = operands[0].value {
            if let TokenValue::Register(src_reg) = operands[1].value {
                let mut bytes = BytesMut::with_capacity(2 + SIZE_OF_ADDRESS);
                bytes.put_u8(handled_size);
                bytes.put_u64(dest_address as u64);
                bytes.put_u8(src_reg as u8);
                return Some(bytes.freeze());
            }
        }
        panic!("Invalid operands for instruction {}: {:#?}", ByteCodes::MOVE_INTO_ADDR_LITERAL_FROM_REG, operands);
    },

    // ByteCodes::MOVE_INTO_ADDR_LITERAL_FROM_ADDR_IN_REG
    | operands: &[Token], handled_size: u8 | {
        if let TokenValue::AddressLiteral(dest_address) = operands[0].value {
            if let TokenValue::Register(src_reg) = operands[1].value {
                let mut bytes = BytesMut::with_capacity(2 + SIZE_OF_ADDRESS);
                bytes.put_u8(handled_size);
                bytes.put_u64(dest_address as u64);
                bytes.put_u8(src_reg as u8);
                return Some(bytes.freeze());
            }
        }
        panic!("Invalid operands for instruction {}: {:#?}", ByteCodes::MOVE_INTO_ADDR_LITERAL_FROM_ADDR_IN_REG, operands);
    },

    // ByteCodes::MOVE_INTO_ADDR_LITERAL_FROM_CONST
    | operands: &[Token], handled_size: u8 | {
        if let TokenValue::AddressLiteral(dest_address) = operands[0].value {
            if let TokenValue::Number(value) = operands[1].value {
                let mut bytes = BytesMut::with_capacity(1 + SIZE_OF_ADDRESS + size_of::<u64>());
                bytes.put_u8(handled_size);
                bytes.put_u64(dest_address as u64);
                bytes.put_u64(value as u64);
                return Some(bytes.freeze());
            }
        }
        panic!("Invalid operands for instruction {}: {:#?}", ByteCodes::MOVE_INTO_ADDR_LITERAL_FROM_CONST, operands);
    },

    // ByteCodes::MOVE_INTO_ADDR_LITERAL_FROM_ADDR_LITERAL
    | operands: &[Token], handled_size: u8 | {
        if let TokenValue::AddressLiteral(dest_address) = operands[0].value {
            if let TokenValue::AddressLiteral(src_address) = operands[1].value {
                let mut bytes = BytesMut::with_capacity(1 + 2*SIZE_OF_ADDRESS);
                bytes.put_u8(handled_size);
                bytes.put_u64(dest_address as u64);
                bytes.put_u64(src_address as u64);
                return Some(bytes.freeze());
            }
        }
        panic!("Invalid operands for instruction {}: {:#?}", ByteCodes::MOVE_INTO_ADDR_LITERAL_FROM_ADDR_LITERAL, operands);
    },

    // ByteCodes::PUSH_FROM_REG
    | operands: &[Token], _handled_size: u8 | {
        if let TokenValue::Register(src_reg) = operands[0].value {
            let mut bytes = BytesMut::with_capacity(1);
            bytes.put_u8(src_reg as u8);
            return Some(bytes.freeze());
        }
        panic!("Invalid operands for instruction {}: {:#?}", ByteCodes::PUSH_FROM_REG, operands);
    },

    // ByteCodes::PUSH_FROM_ADDR_IN_REG
    | operands: &[Token], handled_size: u8 | {
        if let TokenValue::Register(src_reg) = operands[0].value {
            let mut bytes = BytesMut::with_capacity(2);
            bytes.put_u8(handled_size);
            bytes.put_u8(src_reg as u8);
            return Some(bytes.freeze());
        }
        panic!("Invalid operands for instruction {}: {:#?}", ByteCodes::PUSH_FROM_ADDR_IN_REG, operands);
    },

    // ByteCodes::PUSH_FROM_CONST
    | operands: &[Token], handled_size: u8 | {
        if let TokenValue::Number(value) = operands[0].value {
            let mut bytes = BytesMut::with_capacity(1 + size_of::<u64>());
            bytes.put_u8(handled_size);
            bytes.put_u64(value as u64);
            return Some(bytes.freeze());
        }
        panic!("Invalid operands for instruction {}: {:#?}", ByteCodes::PUSH_FROM_CONST, operands);
    },

    // ByteCodes::PUSH_FROM_ADDR_LITERAL
    | operands: &[Token], handled_size: u8 | {
        if let TokenValue::AddressLiteral(value) = operands[0].value {
            let mut bytes = BytesMut::with_capacity(1 + SIZE_OF_ADDRESS);
            bytes.put_u8(handled_size);
            bytes.put_u64(value as u64);
            return Some(bytes.freeze());
        }
        panic!("Invalid operands for instruction {}: {:#?}", ByteCodes::PUSH_FROM_ADDR_LITERAL, operands);
    },

    // ByteCodes::POP_INTO_REG
    | operands: &[Token], _handled_size: u8 | {
        if let TokenValue::Register(dest_reg) = operands[0].value {
            let mut bytes = BytesMut::with_capacity(1);
            bytes.put_u8(dest_reg as u8);
            return Some(bytes.freeze());
        }
        panic!("Invalid operands for instruction {}: {:#?}", ByteCodes::POP_INTO_REG, operands);
    },

    // ByteCodes::POP_INTO_ADDR_IN_REG
    | operands: &[Token], handled_size: u8 | {
        if let TokenValue::Register(dest_reg) = operands[0].value {
            let mut bytes = BytesMut::with_capacity(2);
            bytes.put_u8(handled_size);
            bytes.put_u8(dest_reg as u8);
            return Some(bytes.freeze());
        }
        panic!("Invalid operands for instruction {}: {:#?}", ByteCodes::POP_INTO_ADDR_IN_REG, operands);
    },

    // ByteCodes::POP_INTO_ADDR_LITERAL
    | operands: &[Token], handled_size: u8 | {
        if let TokenValue::AddressLiteral(dest_address) = operands[0].value {
            let mut bytes = BytesMut::with_capacity(1 + SIZE_OF_ADDRESS);
            bytes.put_u8(handled_size);
            bytes.put_u64(dest_address as u64);
            return Some(bytes.freeze());
        }
        panic!("Invalid operands for instruction {}: {:#?}", ByteCodes::POP_INTO_ADDR_LITERAL, operands);
    },

    // Control flow

    // ByteCodes::LABEL
    | _operands: &[Token], _handled_size: u8 | {
        panic!("Label instructions shouldn't be converted into byte code");
    },

    // ByteCodes::JUMP
    | operands: &[Token], _handled_size: u8 | {
        if let TokenValue::AddressLiteral(target) = operands[0].value {
            let mut bytes = BytesMut::with_capacity(SIZE_OF_ADDRESS);
            bytes.put_u64(target as u64);
            return Some(bytes.freeze());
        }
        panic!("Invalid operands for instruction {}: {:#?}", ByteCodes::JUMP, operands);
    },

    // ByteCodes::JUMP_IF_TRUE_REG
    | operands: &[Token], _handled_size: u8 | {
        if let TokenValue::AddressLiteral(target) = operands[0].value {
            if let TokenValue::Register(check_reg) = operands[1].value {
                let mut bytes = BytesMut::with_capacity(SIZE_OF_ADDRESS + SIZE_OF_REGISTER);
                bytes.put_u64(target as u64);
                bytes.put_u8(check_reg as u8);
                return Some(bytes.freeze());
            }
        }
        panic!("Invalid operands for instruction {}: {:#?}", ByteCodes::JUMP_IF_TRUE_REG, operands);
    },

    // ByteCodes::JUMP_IF_FALSE_REG
    | operands: &[Token], _handled_size: u8 | {
        if let TokenValue::AddressLiteral(target) = operands[0].value {
            if let TokenValue::Register(check_reg) = operands[1].value {
                let mut bytes = BytesMut::with_capacity(SIZE_OF_ADDRESS + SIZE_OF_REGISTER);
                bytes.put_u64(target as u64);
                bytes.put_u8(check_reg as u8);
                return Some(bytes.freeze());
            }
        }
        panic!("Invalid operands for instruction {}: {:#?}", ByteCodes::JUMP_IF_FALSE_REG, operands);
    }, 

    // Comparison

    // ByteCodes::COMPARE_REG_REG
    | operands: &[Token], _handled_size: u8 | {
        if let TokenValue::Register(left_reg) = operands[0].value {
            if let TokenValue::Register(right_reg) = operands[1].value {
                let mut bytes = BytesMut::with_capacity(SIZE_OF_REGISTER * 2);
                bytes.put_u8(left_reg as u8);
                bytes.put_u8(right_reg as u8);
                return Some(bytes.freeze());
            }
        }
        panic!("Invalid operands for instruction {}: {:#?}", ByteCodes::COMPARE_REG_REG, operands);
    },

    // ByteCodes::COMPARE_REG_CONST
    | operands: &[Token], _handled_size: u8 | {
        if let TokenValue::Register(left_reg) = operands[0].value {
            if let TokenValue::Number(right_value) = operands[1].value {
                let mut bytes = BytesMut::with_capacity(SIZE_OF_REGISTER + size_of::<u64>());
                bytes.put_u8(left_reg as u8);
                bytes.put_u64(right_value as u64);
                return Some(bytes.freeze());
            }
        }
        panic!("Invalid operands for instruction {}: {:#?}", ByteCodes::COMPARE_REG_CONST, operands);
    },

    // ByteCodes::COMPARE_CONST_REG
    | operands: &[Token], _handled_size: u8 | {
        if let TokenValue::Number(left_value) = operands[0].value {
            if let TokenValue::Register(right_reg) = operands[1].value {
                let mut bytes = BytesMut::with_capacity(SIZE_OF_REGISTER + size_of::<u64>());
                bytes.put_u8(right_reg as u8);
                bytes.put_u64(left_value as u64);
                return Some(bytes.freeze());
            }
        }
        panic!("Invalid operands for instruction {}: {:#?}", ByteCodes::COMPARE_CONST_REG, operands);
    },

    // ByteCodes::COMPARE_CONST_CONST
    | operands: &[Token], _handled_size: u8 | {
        if let TokenValue::Number(left_value) = operands[0].value {
            if let TokenValue::Number(right_value) = operands[1].value {
                let mut bytes = BytesMut::with_capacity(size_of::<u64>() * 2);
                bytes.put_u64(left_value as u64);
                bytes.put_u64(right_value as u64);
                return Some(bytes.freeze());
            }
        }
        panic!("Invalid operands for instruction {}: {:#?}", ByteCodes::COMPARE_CONST_CONST, operands);
    },

    // Interrupts

    // ByteCodes::PRINT
    | _operands: &[Token], _handled_size: u8 | {
        None
    },

    // ByteCodes::PRINT_STRING
    | _operands: &[Token], _handled_size: u8 | {
        None
    },

    // ByteCodes::INPUT_INT
    | _operands: &[Token], _handled_size: u8 | {
        None
    },

    // ByteCodes::INPUT_STRING
    | _operands: &[Token], _handled_size: u8 | {
        None
    },

    // ByteCodes::EXIT
    | _operands: &[Token], _handled_size: u8 | {
        None
    }

];

