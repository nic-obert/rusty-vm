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
    AddressInRegisterIncomplete(String),
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
            TokenValue::AddressInRegisterIncomplete(_) => 8,
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
            TokenValue::AddressInRegisterIncomplete(ref name) => write!(f, "ADDRESS_IN_REGISTER_INCOMPLETE({})", name),
        }
    }
}

