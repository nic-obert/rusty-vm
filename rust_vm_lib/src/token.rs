use std::fmt;
use crate::registers::Registers;


#[derive(Debug)]
pub enum TokenValue {
    Register(Registers),
    AddressInRegister(Registers),
    Number(i64),
    AddressLiteral(usize),
    Label(String),
    Name(String),
    AddressGeneric(usize),
    CurrentPosition(usize),
    AddressAtIdentifier(String),
    Char(char),
}


#[derive(Debug, Clone, Copy)]

pub enum TokenTypes {
    Register = 0,
    AddressInRegister = 1,
    Number = 2,
    AddressLiteral = 3,
    Label = 4,
    Name = 5,
    AddressGeneric = 6,
    CurrentPosition = 7,
    AddressInRegisterIncomplete = 8,
}


impl TokenTypes {

    pub fn size(&self) -> u8 {
        match self {
            TokenTypes::Register => 1,
            TokenTypes::AddressInRegister => 1,
            TokenTypes::Number => 8, // Number is variable size, with 8 bytes being the default
            TokenTypes::AddressLiteral => 8,
            TokenTypes::Label => panic!("Label size is not defined"),
            TokenTypes::Name => panic!("Name size is not defined"),
            TokenTypes::AddressGeneric => panic!("AddressGeneric size is not defined"),
            TokenTypes::CurrentPosition => panic!("CurrentPosition size is not defined"),
            TokenTypes::AddressInRegisterIncomplete => panic!("AddressInRegisterIncomplete size is not defined"),
        }
    }

}


impl TokenValue {

    pub fn to_ordinal(&self) -> u8 {
        match self {
            TokenValue::Register(_) => 0,
            TokenValue::AddressInRegister(_) => 1,
            TokenValue::Number(_) => 2,
            TokenValue::AddressLiteral(_) => 3,
            TokenValue::Label(_) => 4,
            TokenValue::Name(_) => 5,
            TokenValue::AddressGeneric(_) => 6,
            TokenValue::CurrentPosition(_) => 7,
            TokenValue::AddressAtIdentifier(_) => 8,
            TokenValue::Char(_) => panic!("Char does not have an ordinal value"),
        }
    }
    
}


#[derive(Debug)]
pub struct Token {
    pub value: TokenValue
}


impl Token {

    pub fn new(value: TokenValue) -> Token {
        Token {
            value
        }
    }

}


// Implement printing for Token
impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.value {
            TokenValue::Register(reg) => write!(f, "REGISTER({})", reg),
            TokenValue::AddressInRegister(reg) => write!(f, "ADDRESS_IN_REGISTER({})", reg),
            TokenValue::Number(num) => write!(f, "NUMBER({})", num),
            TokenValue::AddressLiteral(num) => write!(f, "ADDRESS_LITERAL({})", num),
            TokenValue::Label(ref label) => write!(f, "LABEL({})", label),
            TokenValue::Name(ref name) => write!(f, "NAME({})", name),
            TokenValue::AddressGeneric(num) => write!(f, "ADDRESS_GENERIC({})", num),
            TokenValue::CurrentPosition(num) => write!(f, "CURRENT_POSITION({})", num),
            TokenValue::AddressAtIdentifier(ref name) => write!(f, "ADDRESS_IN_REGISTER_INCOMPLETE({})", name),
            TokenValue::Char(c) => write!(f, "CHAR({})", c),
        }
    }
}


impl fmt::Display for TokenTypes {
    
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TokenTypes::Register => write!(f, "REGISTER"),
            TokenTypes::AddressInRegister => write!(f, "ADDRESS_IN_REGISTER"),
            TokenTypes::Number => write!(f, "NUMBER"),
            TokenTypes::AddressLiteral => write!(f, "ADDRESS_LITERAL"),
            TokenTypes::Label => write!(f, "LABEL"),
            TokenTypes::Name => write!(f, "NAME"),
            TokenTypes::AddressGeneric => write!(f, "ADDRESS_GENERIC"),
            TokenTypes::CurrentPosition => write!(f, "CURRENT_POSITION"),
            TokenTypes::AddressInRegisterIncomplete => write!(f, "ADDRESS_IN_REGISTER_INCOMPLETE"),
        }
    }

}

