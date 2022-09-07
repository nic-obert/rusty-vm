use rust_vm_lib::token::TokenTypes;
use rust_vm_lib::registers::REGISTER_NAMES;
use lazy_static::lazy_static;


#[derive(Clone)]
pub struct Argument {
    pub size: u8,
    pub kind: TokenTypes
}

impl Argument {

    fn new(kind: TokenTypes) -> Argument {
        Argument {
            size: kind.size(),
            kind
        }
    }

}


lazy_static! {

pub static ref DISASSEMBLY_TABLE: 
    [ (&'static str, Option<Vec<u8>>, Option<Vec<Argument>>); 44 ]
= [
    ("add", None, None), // ByteCodes::ADD
    ("sub", None, None), // ByteCodes::SUB
    ("mul", None, None), // ByteCodes::MUL
    ("div", None, None), // ByteCodes::DIV
    ("mod", None, None), // ByteCodes::MOD

    ("inc", None, Some(vec![
        Argument::new(TokenTypes::Register) // ByteCodes::INC_REG
    ])),
    ("inc", None, Some(vec![
        Argument::new(TokenTypes::AddressInRegister) // ByteCodes::INC_ADDR_IN_REG
    ])),
    ("inc", None, Some(vec![
        Argument::new(TokenTypes::AddressLiteral) // ByteCodes::INC_ADDR_LITERAL
    ])),

    ("dec", None, Some(vec![
        Argument::new(TokenTypes::Register) // ByteCodes::DEC_REG
    ])),
    ("dec", None, Some(vec![
        Argument::new(TokenTypes::AddressInRegister) // ByteCodes::DEC_ADDR_IN_REG
    ])),
    ("dec", None, Some(vec![
        Argument::new(TokenTypes::AddressLiteral) // ByteCodes::DEC_ADDR_LITERAL
    ])),

    ("nop", None, None), // ByteCodes::NO_OPERATION

    ("mov", None, Some(vec![ // ByteCodes::MOVE_INTO_REG_FROM_REG
        Argument::new(TokenTypes::Register), 
        Argument::new(TokenTypes::Register)
    ])),
    ("mov", None, Some(vec![ // ByteCodes::MOVE_INTO_REG_FROM_ADDR_IN_REG
        Argument::new(TokenTypes::Register),
        Argument::new(TokenTypes::AddressInRegister)
    ])),
    ("mov", Some(vec![1]), Some(vec![ // ByteCodes::MOVE_INTO_REG_FROM_CONST
        Argument::new(TokenTypes::Register),
        Argument::new(TokenTypes::Number)
    ])),
    ("mov", None, Some(vec![ // ByteCodes::MOVE_INTO_REG_FROM_ADDR_LITERAL
        Argument::new(TokenTypes::Register),
        Argument::new(TokenTypes::AddressLiteral)
    ])),
    ("mov", None, Some(vec![ // ByteCodes::MOVE_INTO_ADDR_IN_REG_FROM_REG
        Argument::new(TokenTypes::AddressInRegister),
        Argument::new(TokenTypes::Register)
    ])),
    ("mov", None, Some(vec![ // ByteCodes::MOVE_INTO_ADDR_IN_REG_FROM_ADDR_IN_REG
        Argument::new(TokenTypes::AddressInRegister),
        Argument::new(TokenTypes::AddressInRegister)
    ])),
    ("mov", Some(vec![1]), Some(vec![ // ByteCodes::MOVE_INTO_ADDR_IN_REG_FROM_CONST
        Argument::new(TokenTypes::AddressInRegister),
        Argument::new(TokenTypes::Number)
    ])),
    ("mov", None, Some(vec![ // ByteCodes::MOVE_INTO_ADDR_IN_REG_FROM_ADDR_LITERAL
        Argument::new(TokenTypes::AddressInRegister),
        Argument::new(TokenTypes::AddressLiteral)
    ])),
    ("mov", None, Some(vec![ // ByteCodes::MOVE_INTO_ADDR_LITERAL_FROM_REG
        Argument::new(TokenTypes::AddressLiteral),
        Argument::new(TokenTypes::Register)
    ])),
    ("mov", None, Some(vec![ // ByteCodes::MOVE_INTO_ADDR_LITERAL_FROM_ADDR_IN_REG
        Argument::new(TokenTypes::AddressLiteral),
        Argument::new(TokenTypes::AddressInRegister)
    ])),
    ("mov", Some(vec![1]), Some(vec![ // ByteCodes::MOVE_INTO_ADDR_LITERAL_FROM_CONST
        Argument::new(TokenTypes::AddressLiteral),
        Argument::new(TokenTypes::Number)
    ])),
    ("mov", None, Some(vec![ // ByteCodes::MOVE_INTO_ADDR_LITERAL_FROM_ADDR_LITERAL
        Argument::new(TokenTypes::AddressLiteral),
        Argument::new(TokenTypes::AddressLiteral)
    ])),

    ("push", None, Some(vec![ // ByteCodes::PUSH_FROM_REG
        Argument::new(TokenTypes::Register)
    ])),
    ("push", None, Some(vec![ // ByteCodes::PUSH_FROM_ADDR_IN_REG
        Argument::new(TokenTypes::AddressInRegister)
    ])),
    ("push", Some(vec![0]), Some(vec![ // ByteCodes::PUSH_FROM_CONST
        Argument::new(TokenTypes::Number)
    ])),
    ("push", None, Some(vec![ // ByteCodes::PUH_FROM_ADDR_LITERAL
        Argument::new(TokenTypes::AddressLiteral)
    ])),

    ("pop", None, Some(vec![ // ByteCodes::POP_INTO_REG
        Argument::new(TokenTypes::Register)
    ])),
    ("pop", None, Some(vec![ // ByteCodes::POP_INTO_ADDR_IN_REG
        Argument::new(TokenTypes::AddressInRegister)
    ])),
    ("pop", None, Some(vec![ // ByteCodes::POP_INTO_ADDR_LITERAL
        Argument::new(TokenTypes::AddressLiteral)
    ])),

    // Doesn't get used, but it's here for keeping the index correct
    ("@", None, None), // ByteCodes::LABEL

    ("jmp", None, Some(vec![ // ByteCodes::JUMP
        Argument::new(TokenTypes::AddressLiteral)
    ])),
    ("jmpnz", None, Some(vec![ // ByteCodes::JUMP_IF_TRUE_REG
        Argument::new(TokenTypes::AddressLiteral),
        Argument::new(TokenTypes::Register)
    ])),
    ("jmpz", None, Some(vec![ // ByteCodes::JUMP_IF_FALSE_REG
        Argument::new(TokenTypes::AddressLiteral),
        Argument::new(TokenTypes::Register)
    ])),

    ("cmp", None, Some(vec![ // ByteCodes::COMPARE_REG_REG
        Argument::new(TokenTypes::Register),
        Argument::new(TokenTypes::Register)
    ])),
    ("cmp", Some(vec![1]), Some(vec![ // ByteCodes::COMPARE_REG_CONST
        Argument::new(TokenTypes::Register),
        Argument::new(TokenTypes::Number)
    ])),
    ("cmp", Some(vec![0]), Some(vec![ // ByteCodes::COMPARE_CONST_REG
        Argument::new(TokenTypes::Number),
        Argument::new(TokenTypes::Register)
    ])),
    ("cmp", Some(vec![0, 1]), Some(vec![ // ByteCodes::COMPARE_CONST_CONST
        Argument::new(TokenTypes::Number),
        Argument::new(TokenTypes::Number)
    ])),

    ("print", None, None), // ByteCodes::PRINT
    ("prints", None, None), // ByteCodes::PRINT_STRING

    ("ini", None, None), // ByteCodes::INPUT_INT
    ("ins", None, None), // ByteCodes::INPUT_STRING

    ("exit", None, None) // ByteCodes::EXIT

];


pub static ref OPERATOR_DISASSEMBLY_TABLE:
    [ fn(&[u8]) -> String; 4 ]
= [

    | byte_code | {
        REGISTER_NAMES.get(byte_code[0] as usize).unwrap_or_else(
            || panic!("Invalid register index: {}", byte_code[0])
        ).to_string()
    },

    | byte_code | {
        format!("[{}]", REGISTER_NAMES.get(byte_code[0] as usize).unwrap_or_else(
            || panic!("Invalid register index: {}", byte_code[0])
        ))
    },

    | byte_code | {
        let mut number = 0;
        for byte in byte_code.iter() {
            number = (number << 8) | *byte as u32;
        }
        number.to_string()
    },

    | byte_code | {
        let mut number = 0;
        for byte in byte_code.iter() {
            number = (number << 8) | *byte as u32;
        }
        format!("[0x {:x}]", number)
    }

];


}

