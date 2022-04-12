use std::fmt;
use registers::Registers;


pub static TOKEN_NAMES_TABLE: [&str; 8] = [
    "REGISTER",
    "ADDRESS_IN_REGISTER",
    "NUMBER",
    "ADDRESS_LITERAL",
    "LABEL",
    "NAME",
    "ADDRESS_GENERIC",
    "CURRENT_POSITION"
];


// TODO: check if this is correct
pub enum TokenValue {
    Register(Registers),
    AddressInRegister(Registers),
    Number(i32),
    AddressLiteral(i32),
    Label(String),
    Name(String),
    AddressGeneric(i32),
    CurrentPosition(i32)
}


pub struct Token {
    pub value: TokenValue
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
            TokenValue::CurrentPosition(num) => write!(f, "CURRENT_POSITION({})", num)
        }
    }
}


