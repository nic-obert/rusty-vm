use rust_vm_lib::token::{Token, TokenValue};
use rust_vm_lib::byte_code::ByteCodes;
use bytes::{BytesMut, Bytes, BufMut};


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


/// Returns the bytes representation of the number using little endian.
pub fn number_to_bytes(mut number: u64, size: usize) -> Result<Bytes, String> {
    
    if number_size(number) > size {
        return Err(format!("Number {} is too big to fit in {} bytes", number, size));
    }

    let mut value = BytesMut::with_capacity(size);
    // Perform a reverse loop for little endian
    for _ in 0..size {
        value.put_u8((number % 256) as u8);
        number /= 256;
    }

    Ok(value.freeze())
}


/// The following functions are used to convert the operand tokens to bytes.
pub const INSTRUCTION_CONVERSION_TABLE:
    [ fn(&[Token], u8) -> Result<Option<Bytes>, String>; 44 ]
= [

    // Arithmetic

    // ByteCodes::ADD
    | _operands: &[Token], _handled_size: u8 | {
        Ok(None)
    },
    // ByteCodes::SUB
    | _operands: &[Token], _handled_size: u8 | {
        Ok(None)
    },
    // ByteCodes::MUL
    | _operands: &[Token], _handled_size: u8 | {
        Ok(None)
    },
    // ByteCodes::DIV
    | _operands: &[Token], _handled_size: u8 | {
        Ok(None)
    },
    // ByteCodes::MOD
    | _operands: &[Token], _handled_size: u8 | {
        Ok(None)
    },

    // ByteCodes::INC_REG
    | operands: &[Token], _handled_size: u8 | {
        if let TokenValue::Register(dest_reg) = operands[0].value {
            return Ok(Some(number_to_bytes(dest_reg as u64, SIZE_OF_REGISTER)?));
        }
        Err(format!("{} expects a register as its first operand.", ByteCodes::INC_REG))
    },
    // ByteCodes::INC_ADDR_IN_REG
    | operands: &[Token], handled_size: u8 | {
        if let TokenValue::AddressInRegister(dest_reg) = operands[0].value {
            let operand_bytes = number_to_bytes(dest_reg as u64, SIZE_OF_REGISTER)?;
            let mut bytes = BytesMut::with_capacity(1 + operand_bytes.len());
            bytes.put_u8(handled_size);
            bytes.put(operand_bytes);
            return Ok(Some(bytes.freeze()));
        }
        Err(format!("{} expects a register as its first operand.", ByteCodes::INC_ADDR_IN_REG))
    },
    // ByteCodes::INC_ADDR_LITERAL
    | operands: &[Token], handled_size: u8 | {
        if let TokenValue::AddressLiteral(address) = operands[0].value {
            let operand_bytes = number_to_bytes(address as u64, SIZE_OF_ADDRESS)?;
            let mut bytes = BytesMut::with_capacity(1 + operand_bytes.len());
            bytes.put_u8(handled_size);
            bytes.put(operand_bytes);
            return Ok(Some(bytes.freeze()));
        }
        Err(format!("{} expects a register as its first operand.", ByteCodes::INC_ADDR_LITERAL))
    },

    // ByteCodes::DEC_REG
    | operands: &[Token], _handled_size: u8 | {
        if let TokenValue::Register(dest_reg) = operands[0].value {
            return Ok(Some(number_to_bytes(dest_reg as u64, SIZE_OF_REGISTER)?));
        } 
        Err(format!("{} expects a register as its first operand.", ByteCodes::DEC_REG))
    },
    // ByteCodes::DEC_ADDR_IN_REG
    | operands: &[Token], handled_size: u8 | {
        if let TokenValue::AddressInRegister(dest_reg) = operands[0].value {
            let operand_bytes = number_to_bytes(dest_reg as u64, SIZE_OF_REGISTER)?;
            let mut bytes = BytesMut::with_capacity(1 + operand_bytes.len());
            bytes.put_u8(handled_size);
            bytes.put(operand_bytes);
            return Ok(Some(bytes.freeze()));
        }
        Err(format!("{} expects a register as its first operand.", ByteCodes::DEC_ADDR_IN_REG))
    },
    // ByteCodes::DEC_ADDR_LITERAL
    | operands: &[Token], handled_size: u8 | {
        if let TokenValue::AddressLiteral(address) = operands[0].value {
            let operand_bytes = number_to_bytes(address as u64, SIZE_OF_ADDRESS)?;
            let mut bytes = BytesMut::with_capacity(1 + operand_bytes.len());
            bytes.put_u8(handled_size);
            bytes.put(operand_bytes);
            return Ok(Some(bytes.freeze()));
        } 
        Err(format!("{} expects a register as its first operand.", ByteCodes::DEC_ADDR_LITERAL))
    },

    // No operation
    // ByteCodes::NOP
    | _operands: &[Token], _handled_size: u8 | {
        Ok(None)
    },

    // Memory

    // ByteCodes::MOVE_INTO_REG_FROM_REG
    | operands: &[Token], _handled_size: u8 | {
        if let TokenValue::Register(dest_reg) = operands[0].value {
            if let TokenValue::Register(src_reg) = operands[1].value {
                let mut bytes = BytesMut::with_capacity(2 * SIZE_OF_REGISTER);
                bytes.put_u8(dest_reg as u8);
                bytes.put_u8(src_reg as u8);
                return Ok(Some(bytes.freeze()));
            }
        }
        Err(format!("{} expects two registers as its operands.", ByteCodes::MOVE_INTO_REG_FROM_REG))
    },

    // ByteCodes::MOVE_INTO_REG_FROM_ADDR_IN_REG
    | operands: &[Token], handled_size: u8 | {
        if let TokenValue::Register(dest_reg) = operands[0].value {
            if let TokenValue::AddressInRegister(src_reg) = operands[1].value {
                let mut bytes = BytesMut::with_capacity(1 + 2 * SIZE_OF_REGISTER);
                bytes.put_u8(handled_size);
                bytes.put_u8(dest_reg as u8);
                bytes.put_u8(src_reg as u8);
                return Ok(Some(bytes.freeze()));
            }
        }
        Err(format!("{} expects a register and an address in a register as its operands.", ByteCodes::MOVE_INTO_REG_FROM_ADDR_IN_REG))
    },

    // ByteCodes::MOVE_INTO_REG_FROM_CONST
    | operands: &[Token], handled_size: u8 | {
        if let TokenValue::Register(dest_reg) = operands[0].value {
            if let TokenValue::Number(number) = operands[1].value {
                let mut bytes = BytesMut::with_capacity(1 + SIZE_OF_REGISTER + handled_size as usize);
                bytes.put_u8(handled_size);
                bytes.put_u8(dest_reg as u8);
                bytes.extend(number_to_bytes(number as u64, handled_size as usize));
                return Ok(Some(bytes.freeze()));
            }
        }
        Err(format!("{} expects a register and a number as its operands.", ByteCodes::MOVE_INTO_REG_FROM_CONST))
    },

    // ByteCodes::MOVE_INTO_REG_FROM_ADDR_LITERAL
    | operands: &[Token], handled_size: u8 | {
        if let TokenValue::Register(dest_reg) = operands[0].value {
            if let TokenValue::AddressLiteral(address) = operands[1].value {
                let mut bytes = BytesMut::with_capacity(1 + SIZE_OF_REGISTER + SIZE_OF_ADDRESS);
                bytes.put_u8(handled_size);
                bytes.put_u8(dest_reg as u8);
                bytes.put_u64_le(address as u64);
                return Ok(Some(bytes.freeze()));
            }
        }
        Err(format!("{} expects a register and an address as its operands.", ByteCodes::MOVE_INTO_REG_FROM_ADDR_LITERAL))
    },

    // ByteCodes::MOVE_INTO_ADDR_IN_REG_FROM_REG
    | operands: &[Token], handled_size: u8 | {
        if let TokenValue::AddressInRegister(dest_reg) = operands[0].value {
            if let TokenValue::Register(src_reg) = operands[1].value {
                let mut bytes = BytesMut::with_capacity(1 + 2 * SIZE_OF_REGISTER);
                bytes.put_u8(handled_size);
                bytes.put_u8(dest_reg as u8);
                bytes.put_u8(src_reg as u8);
                return Ok(Some(bytes.freeze()));
            }
        }
        Err(format!("{} expects a register and an address in a register as its operands.", ByteCodes::MOVE_INTO_ADDR_IN_REG_FROM_REG))
    },

    // ByteCodes::MOVE_INTO_ADDR_IN_REG_FROM_ADDR_IN_REG
    | operands: &[Token], handled_size: u8 | {
        if let TokenValue::AddressInRegister(dest_reg) = operands[0].value {
            if let TokenValue::AddressInRegister(src_reg) = operands[1].value {
                let mut bytes = BytesMut::with_capacity(1 + 2 * SIZE_OF_REGISTER);
                bytes.put_u8(handled_size);
                bytes.put_u8(dest_reg as u8);
                bytes.put_u8(src_reg as u8);
                return Ok(Some(bytes.freeze()));
            }
        }
        Err(format!("{} expects two addresses in registers as its operands.", ByteCodes::MOVE_INTO_ADDR_IN_REG_FROM_ADDR_IN_REG))
    },

    // ByteCodes::MOVE_INTO_ADDR_IN_REG_FROM_CONST
    | operands: &[Token], handled_size: u8 | {
        if let TokenValue::AddressInRegister(dest_reg) = operands[0].value {
            if let TokenValue::Number(number) = operands[1].value {
                let mut bytes = BytesMut::with_capacity(1 + SIZE_OF_ADDRESS + handled_size as usize);
                bytes.put_u8(handled_size);
                bytes.put_u8(dest_reg as u8);
                bytes.extend(number_to_bytes(number as u64, handled_size as usize));
                return Ok(Some(bytes.freeze()));
            }
        }
        Err(format!("{} expects an address in a register and a number as its operands.", ByteCodes::MOVE_INTO_ADDR_IN_REG_FROM_CONST))
    },

    // ByteCodes::MOVE_INTO_ADDR_IN_REG_FROM_ADDR_LITERAL
    | operands: &[Token], handled_size: u8 | {
        if let TokenValue::AddressInRegister(dest_reg) = operands[0].value {
            if let TokenValue::AddressLiteral(value) = operands[1].value {
                let mut bytes = BytesMut::with_capacity(1 + SIZE_OF_REGISTER + SIZE_OF_ADDRESS);
                bytes.put_u8(handled_size);
                bytes.put_u8(dest_reg as u8);
                bytes.put_u64_le(value as u64);
                return Ok(Some(bytes.freeze()));
            }
        }
        Err(format!("{} expects an address in a register and an address as its operands.", ByteCodes::MOVE_INTO_ADDR_IN_REG_FROM_ADDR_LITERAL))
    },

    // ByteCodes::MOVE_INTO_ADDR_LITERAL_FROM_REG
    | operands: &[Token], handled_size: u8 | {
        if let TokenValue::AddressLiteral(dest_address) = operands[0].value {
            if let TokenValue::Register(src_reg) = operands[1].value {
                let mut bytes = BytesMut::with_capacity(1 + SIZE_OF_REGISTER + SIZE_OF_ADDRESS);
                bytes.put_u8(handled_size);
                bytes.put_u64_le(dest_address as u64);
                bytes.put_u8(src_reg as u8);
                return Ok(Some(bytes.freeze()));
            }
        }
        Err(format!("{} expects an address and a register as its operands.", ByteCodes::MOVE_INTO_ADDR_LITERAL_FROM_REG))
    },

    // ByteCodes::MOVE_INTO_ADDR_LITERAL_FROM_ADDR_IN_REG
    | operands: &[Token], handled_size: u8 | {
        if let TokenValue::AddressLiteral(dest_address) = operands[0].value {
            if let TokenValue::AddressInRegister(src_reg) = operands[1].value {
                let mut bytes = BytesMut::with_capacity(1 + SIZE_OF_REGISTER + SIZE_OF_ADDRESS);
                bytes.put_u8(handled_size);
                bytes.put_u64_le(dest_address as u64);
                bytes.put_u8(src_reg as u8);
                return Ok(Some(bytes.freeze()));
            }
        }
        Err(format!("{} expects an address and an address in a register as its operands.", ByteCodes::MOVE_INTO_ADDR_LITERAL_FROM_ADDR_IN_REG))
    },

    // ByteCodes::MOVE_INTO_ADDR_LITERAL_FROM_CONST
    | operands: &[Token], handled_size: u8 | {
        if let TokenValue::AddressLiteral(dest_address) = operands[0].value {
            if let TokenValue::Number(number) = operands[1].value {
                let mut bytes = BytesMut::with_capacity(1 + SIZE_OF_ADDRESS + handled_size as usize);
                bytes.put_u8(handled_size);
                bytes.put_u64_le(dest_address as u64);
                bytes.extend(number_to_bytes(number as u64, handled_size as usize));
                return Ok(Some(bytes.freeze()));
            }
        }
        Err(format!("{} expects an address and a number as its operands.", ByteCodes::MOVE_INTO_ADDR_LITERAL_FROM_CONST))
    },

    // ByteCodes::MOVE_INTO_ADDR_LITERAL_FROM_ADDR_LITERAL
    | operands: &[Token], handled_size: u8 | {
        if let TokenValue::AddressLiteral(dest_address) = operands[0].value {
            if let TokenValue::AddressLiteral(src_address) = operands[1].value {
                let mut bytes = BytesMut::with_capacity(1 + 2 * SIZE_OF_ADDRESS);
                bytes.put_u8(handled_size);
                bytes.put_u64_le(dest_address as u64);
                bytes.put_u64_le(src_address as u64);
                return Ok(Some(bytes.freeze()));
            }
        }
        Err(format!("{} expects two addresses as its operands.", ByteCodes::MOVE_INTO_ADDR_LITERAL_FROM_ADDR_LITERAL))
    },

    // ByteCodes::PUSH_FROM_REG
    | operands: &[Token], _handled_size: u8 | {
        if let TokenValue::Register(src_reg) = operands[0].value {
            let mut bytes = BytesMut::with_capacity(1);
            bytes.put_u8(src_reg as u8);
            return Ok(Some(bytes.freeze()));
        }
        Err(format!("{} expects a register as its operand.", ByteCodes::PUSH_FROM_REG))
    },

    // ByteCodes::PUSH_FROM_ADDR_IN_REG
    | operands: &[Token], handled_size: u8 | {
        if let TokenValue::AddressInRegister(src_reg) = operands[0].value {
            let mut bytes = BytesMut::with_capacity(1 + SIZE_OF_REGISTER);
            bytes.put_u8(handled_size);
            bytes.put_u8(src_reg as u8);
            return Ok(Some(bytes.freeze()));
        }
        Err(format!("{} expects an address in a register as its operand.", ByteCodes::PUSH_FROM_ADDR_IN_REG))
    },

    // ByteCodes::PUSH_FROM_CONST
    | operands: &[Token], handled_size: u8 | {
        if let TokenValue::Number(value) = operands[0].value {
            let mut bytes = BytesMut::with_capacity(1 + handled_size as usize);
            bytes.put_u8(handled_size);
            bytes.extend(number_to_bytes(value as u64, handled_size as usize)?);
            return Ok(Some(bytes.freeze()));
        }
        Err(format!("{} expects a number as its operand.", ByteCodes::PUSH_FROM_CONST))
    },

    // ByteCodes::PUSH_FROM_ADDR_LITERAL
    | operands: &[Token], handled_size: u8 | {
        if let TokenValue::AddressLiteral(value) = operands[0].value {
            let mut bytes = BytesMut::with_capacity(1 + SIZE_OF_ADDRESS);
            bytes.put_u8(handled_size);
            bytes.put_u64_le(value as u64);
            return Ok(Some(bytes.freeze()));
        }
        Err(format!("{} expects an address as its operand.", ByteCodes::PUSH_FROM_ADDR_LITERAL))
    },

    // ByteCodes::POP_INTO_REG
    | operands: &[Token], _handled_size: u8 | {
        if let TokenValue::Register(dest_reg) = operands[0].value {
            let mut bytes = BytesMut::with_capacity(1);
            bytes.put_u8(dest_reg as u8);
            return Ok(Some(bytes.freeze()));
        }
        Err(format!("{} expects a register as its operand.", ByteCodes::POP_INTO_REG))
    },

    // ByteCodes::POP_INTO_ADDR_IN_REG
    | operands: &[Token], handled_size: u8 | {
        if let TokenValue::AddressInRegister(dest_reg) = operands[0].value {
            let mut bytes = BytesMut::with_capacity(1 + SIZE_OF_REGISTER);
            bytes.put_u8(handled_size);
            bytes.put_u8(dest_reg as u8);
            return Ok(Some(bytes.freeze()));
        }
        Err(format!("{} expects an address in a register as its operand.", ByteCodes::POP_INTO_ADDR_IN_REG))
    },

    // ByteCodes::POP_INTO_ADDR_LITERAL
    | operands: &[Token], handled_size: u8 | {
        if let TokenValue::AddressLiteral(dest_address) = operands[0].value {
            let mut bytes = BytesMut::with_capacity(1 + SIZE_OF_ADDRESS);
            bytes.put_u8(handled_size);
            bytes.put_u64_le(dest_address as u64);
            return Ok(Some(bytes.freeze()));
        }
        Err(format!("{} expects an address as its operand.", ByteCodes::POP_INTO_ADDR_LITERAL))
    },

    // Control flow

    // ByteCodes::LABEL
    | _operands: &[Token], _handled_size: u8 | {
        panic!("Label instructions shouldn't be converted into byte code. This is a bug.");
    },

    // ByteCodes::JUMP
    | operands: &[Token], _handled_size: u8 | {
        if let TokenValue::AddressLiteral(target) = operands[0].value {
            let mut bytes = BytesMut::with_capacity(SIZE_OF_ADDRESS);
            bytes.put_u64_le(target as u64);
            return Ok(Some(bytes.freeze()));
        }
        Err(format!("{} expects an address as its operand.", ByteCodes::JUMP))
    },

    // ByteCodes::JUMP_IF_TRUE_REG
    | operands: &[Token], _handled_size: u8 | {
        if let TokenValue::AddressLiteral(target) = operands[0].value {
            if let TokenValue::Register(check_reg) = operands[1].value {
                let mut bytes = BytesMut::with_capacity(SIZE_OF_ADDRESS + SIZE_OF_REGISTER);
                bytes.put_u64_le(target as u64);
                bytes.put_u8(check_reg as u8);
                return Ok(Some(bytes.freeze()));
            }
        }
        Err(format!("{} expects an address and a register as its operands.", ByteCodes::JUMP_IF_TRUE_REG))
    },

    // ByteCodes::JUMP_IF_FALSE_REG
    | operands: &[Token], _handled_size: u8 | {
        if let TokenValue::AddressLiteral(target) = operands[0].value {
            if let TokenValue::Register(check_reg) = operands[1].value {
                let mut bytes = BytesMut::with_capacity(SIZE_OF_ADDRESS + SIZE_OF_REGISTER);
                bytes.put_u64_le(target as u64);
                bytes.put_u8(check_reg as u8);
                return Ok(Some(bytes.freeze()));
            }
        }
        Err(format!("{} expects an address and a register as its operands.", ByteCodes::JUMP_IF_FALSE_REG))
    }, 

    // Comparison

    // ByteCodes::COMPARE_REG_REG
    | operands: &[Token], _handled_size: u8 | {
        if let TokenValue::Register(left_reg) = operands[0].value {
            if let TokenValue::Register(right_reg) = operands[1].value {
                let mut bytes = BytesMut::with_capacity(2 * SIZE_OF_REGISTER);
                bytes.put_u8(left_reg as u8);
                bytes.put_u8(right_reg as u8);
                return Ok(Some(bytes.freeze()));
            }
        }
        Err(format!("{} expects two registers as its operands.", ByteCodes::COMPARE_REG_REG))
    },

    // ByteCodes::COMPARE_REG_CONST
    | operands: &[Token], handled_size: u8 | {
        if let TokenValue::Register(left_reg) = operands[0].value {
            if let TokenValue::Number(right_value) = operands[1].value {
                let mut bytes = BytesMut::with_capacity(1 + SIZE_OF_REGISTER + handled_size as usize);
                bytes.put_u8(handled_size);
                bytes.put_u8(left_reg as u8);
                bytes.extend(number_to_bytes(right_value as u64, handled_size as usize));
                return Ok(Some(bytes.freeze()));
            }
        }
        Err(format!("{} expects a register and a constant as its operands.", ByteCodes::COMPARE_REG_CONST))
    },

    // ByteCodes::COMPARE_CONST_REG
    | operands: &[Token], handled_size: u8 | {
        if let TokenValue::Number(left_value) = operands[0].value {
            if let TokenValue::Register(right_reg) = operands[1].value {
                let mut bytes = BytesMut::with_capacity(1 + SIZE_OF_REGISTER + handled_size as usize);
                bytes.put_u8(handled_size);
                bytes.put_u8(right_reg as u8);
                bytes.extend(number_to_bytes(left_value as u64, handled_size as usize));
                return Ok(Some(bytes.freeze()));
            }
        }
        Err(format!("{} expects a constant and a register as its operands.", ByteCodes::COMPARE_CONST_REG))
    },

    // ByteCodes::COMPARE_CONST_CONST
    | operands: &[Token], handled_size: u8 | {
        if let TokenValue::Number(left_value) = operands[0].value {
            if let TokenValue::Number(right_value) = operands[1].value {
                let mut bytes = BytesMut::with_capacity(1 + handled_size as usize * 2);
                bytes.put_u8(handled_size);
                bytes.extend(number_to_bytes(left_value as u64, handled_size as usize)?);
                bytes.extend(number_to_bytes(right_value as u64, handled_size as usize)?);
                return Ok(Some(bytes.freeze()));
            }
        }
        Err(format!("{} expects two constants as its operands.", ByteCodes::COMPARE_CONST_CONST))
    },

    // Interrupts

    // ByteCodes::PRINT
    | _operands: &[Token], _handled_size: u8 | {
        Ok(None)
    },

    // ByteCodes::PRINT_STRING
    | _operands: &[Token], _handled_size: u8 | {
        Ok(None)
    },

    // ByteCodes::INPUT_INT
    | _operands: &[Token], _handled_size: u8 | {
        Ok(None)
    },

    // ByteCodes::INPUT_STRING
    | _operands: &[Token], _handled_size: u8 | {
        Ok(None)
    },

    // ByteCodes::EXIT
    | _operands: &[Token], _handled_size: u8 | {
        Ok(None)
    }

];

