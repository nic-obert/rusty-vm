use std::fmt;

use crate::registers::Registers;


#[derive(Debug)]
pub enum NumberFormat {
    Decimal,
    Hexadecimal,
    Binary,
    Float { decimal: String },

    Unknown,
}


#[derive(Debug)]
pub enum NumberSign {
    Positive,
    Negative,
}


#[derive(Debug)]
pub enum TokenValue {
    Register(Registers),
    AddressInRegister(Registers),
    Number { value: i64, sign: NumberSign, format: NumberFormat },
    AddressLiteral { value: usize, format: NumberFormat },
    Label(String),
    AddressAtLabel(String),
    Name(String),
    AddressGeneric(),
    AddressAtIdentifier(String),
    Char(char),
}


impl TokenValue {

    /// Converts the token value enum to an ordinal value to be used in lookup tables
    pub fn to_ordinal(&self) -> u8 {
        match self {
            TokenValue::Register(_) => 0,
            TokenValue::AddressInRegister(_) => 1,
            TokenValue::Number { .. } => 2,
            TokenValue::AddressLiteral { .. } => 3,
            TokenValue::Label(_) => 4,
            TokenValue::AddressAtLabel(_) => 5,
            _ => unreachable!()
        }
    }
    
}


#[derive(Debug, Clone, Copy)]

pub enum TokenTypes {
    Register = 0,
    AddressInRegister = 1,
    Number = 2,
    AddressLiteral = 3,
    Label = 4,
    AddressAtLabel = 5,
    Name = 6,
    AddressGeneric = 7,
}


impl TokenTypes {

    pub fn from_ordinal(ordinal: u8) -> TokenTypes {
        match ordinal {
            0 => TokenTypes::Register,
            1 => TokenTypes::AddressInRegister,
            2 => TokenTypes::Number,
            3 => TokenTypes::AddressLiteral,
            4 => TokenTypes::Label,
            5 => TokenTypes::AddressAtLabel,
            _ => panic!("Invalid ordinal value for token type"),
        }
    }

    /// Return the size in bytes needed to represent the value in the bytecode
    pub fn size(&self) -> u8 {
        match self {
            TokenTypes::Register => 1,
            TokenTypes::AddressInRegister => 1,
            TokenTypes::Number => 8, // Number is variable size, with 8 bytes being the default
            TokenTypes::AddressLiteral => 8,
            TokenTypes::Label => unreachable!(),
            TokenTypes::Name => unreachable!(),
            TokenTypes::AddressGeneric => unreachable!(),
            TokenTypes::AddressAtLabel => unreachable!(),
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


impl fmt::Display for Token {

    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.value {
            TokenValue::Register(reg) => write!(f, "REGISTER({})", reg),
            TokenValue::AddressInRegister(reg) => write!(f, "ADDRESS_IN_REGISTER({})", reg),
            TokenValue::Number { value, .. } => write!(f, "NUMBER({})", value),
            TokenValue::AddressLiteral { value, .. } => write!(f, "ADDRESS_LITERAL({})", value),
            TokenValue::Label(ref label) => write!(f, "LABEL({})", label),
            TokenValue::Name(ref name) => write!(f, "NAME({})", name),
            TokenValue::AddressGeneric() => write!(f, "ADDRESS_GENERIC"),
            TokenValue::AddressAtIdentifier(ref name) => write!(f, "ADDRESS_IN_REGISTER_INCOMPLETE({})", name),
            TokenValue::Char(c) => write!(f, "CHAR({})", c),
            TokenValue::AddressAtLabel(ref label) => write!(f, "ADDRESS_AT_LABEL({})", label),
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
            TokenTypes::AddressAtLabel => write!(f, "ADDRESS_AT_LABEL"),
            TokenTypes::Name => write!(f, "NAME"),
            TokenTypes::AddressGeneric => panic!("AddressGeneric does not have a display value"),
        }
    }

}

