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
    ("add", None, None),
    ("sub", None, None),
    ("mul", None, None),
    ("div", None, None),

    ("inc", None, Some(vec![
        Argument::new(TokenTypes::Register)
    ])),
    ("inc", None, Some(vec![
        Argument::new(TokenTypes::AddressInRegister)
    ])),
    ("inc", None, Some(vec![
        Argument::new(TokenTypes::AddressLiteral)
    ])),

    ("dec", None, Some(vec![
        Argument::new(TokenTypes::Register)
    ])),
    ("dec", None, Some(vec![
        Argument::new(TokenTypes::AddressInRegister)
    ])),
    ("dec", None, Some(vec![
        Argument::new(TokenTypes::AddressLiteral)
    ])),

    ("nop", None, None),

    ("mov", None, Some(vec![
        Argument::new(TokenTypes::Register),
        Argument::new(TokenTypes::Register)
    ])),
    ("mov", None, Some(vec![
        Argument::new(TokenTypes::Register),
        Argument::new(TokenTypes::AddressInRegister)
    ])),
    ("mov", Some(vec![1]), Some(vec![
        Argument::new(TokenTypes::Register),
        Argument::new(TokenTypes::AddressInRegister)
    ])),
    ("mov", None, Some(vec![
        Argument::new(TokenTypes::Register),
        Argument::new(TokenTypes::AddressLiteral)
    ])),
    ("mov", None, Some(vec![
        Argument::new(TokenTypes::AddressInRegister),
        Argument::new(TokenTypes::Register)
    ])),
    ("mov", None, Some(vec![
        Argument::new(TokenTypes::AddressInRegister),
        Argument::new(TokenTypes::AddressInRegister)
    ])),
    ("mov", Some(vec![1]), Some(vec![
        Argument::new(TokenTypes::AddressInRegister),
        Argument::new(TokenTypes::Number)
    ])),
    ("mov", None, Some(vec![
        Argument::new(TokenTypes::AddressInRegister),
        Argument::new(TokenTypes::AddressLiteral)
    ])),
    ("mov", None, Some(vec![
        Argument::new(TokenTypes::AddressLiteral),
        Argument::new(TokenTypes::Register)
    ])),
    ("mov", None, Some(vec![
        Argument::new(TokenTypes::AddressLiteral),
        Argument::new(TokenTypes::AddressInRegister)
    ])),
    ("mov", Some(vec![1]), Some(vec![
        Argument::new(TokenTypes::AddressLiteral),
        Argument::new(TokenTypes::Number)
    ])),
    ("mov", None, Some(vec![
        Argument::new(TokenTypes::AddressLiteral),
        Argument::new(TokenTypes::AddressLiteral)
    ])),

    ("push", None, Some(vec![
        Argument::new(TokenTypes::Register)
    ])),
    ("push", None, Some(vec![
        Argument::new(TokenTypes::AddressInRegister)
    ])),
    ("push", Some(vec![0]), Some(vec![
        Argument::new(TokenTypes::Number)
    ])),
    ("push", None, Some(vec![
        Argument::new(TokenTypes::AddressLiteral)
    ])),

    ("pop", None, Some(vec![
        Argument::new(TokenTypes::Register)
    ])),
    ("pop", None, Some(vec![
        Argument::new(TokenTypes::AddressInRegister)
    ])),
    ("pop", Some(vec![0]), Some(vec![
        Argument::new(TokenTypes::Number)
    ])),
    ("pop", None, Some(vec![
        Argument::new(TokenTypes::AddressLiteral)
    ])),

    ("@", None, None), // Doesn't get used, but it's here for keeping the index correct

    ("jmp", None, Some(vec![
        Argument::new(TokenTypes::AddressLiteral)
    ])),
    ("jmpnz", None, Some(vec![
        Argument::new(TokenTypes::AddressLiteral),
        Argument::new(TokenTypes::Register)
    ])),
    ("jmpz", None, Some(vec![
        Argument::new(TokenTypes::AddressLiteral),
        Argument::new(TokenTypes::Register)
    ])),

    ("cmp", None, Some(vec![
        Argument::new(TokenTypes::Register),
        Argument::new(TokenTypes::Register)
    ])),
    ("cmp", Some(vec![1]), Some(vec![
        Argument::new(TokenTypes::Register),
        Argument::new(TokenTypes::Number)
    ])),
    ("cmp", Some(vec![0]), Some(vec![
        Argument::new(TokenTypes::Number),
        Argument::new(TokenTypes::Register)
    ])),
    ("cmp", Some(vec![0, 1]), Some(vec![
        Argument::new(TokenTypes::Number),
        Argument::new(TokenTypes::Number)
    ])),

    ("print", None, None),
    ("prints", None, None),

    ("ini", None, None),
    ("ins", None, None),

    ("exit", None, None)

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

