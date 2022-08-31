use std::fmt;
use crate::registers::Registers;


// TODO: check if this is correct
#[allow(dead_code)]
pub enum TokenValue {
    Register(Registers),
    AddressInRegister(Registers),
    Number(i64),
    AddressLiteral(u64),
    Label(String),
    Name(String),
    AddressGeneric(u64),
    CurrentPosition(u64),
    AddressInRegisterIncomplete(String),
}


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


