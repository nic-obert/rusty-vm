use std::fmt;


#[allow(dead_code, non_camel_case_types)]
#[derive(Clone, Copy, Debug)]
pub enum Registers {
    A = 0,
    B,
    C,
    D,

    EXIT,
    INPUT,
    ERROR,
    PRINT,

    STACK_POINTER,
    PROGRAM_COUNTER,

    ZERO_FLAG,
    SIGN_FLAG,
    REMAINDER_FLAG,
}


impl fmt::Display for Registers {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", REGISTER_NAMES[*self as usize])
    }
}


const REGISTER_NAMES: [&str; 13] = [
    "a",
    "b",
    "c",
    "d",
    "exit",
    "input",
    "error",
    "print",

    "sp",
    "pc",

    "zf",
    "sf",
    "rf",
];


pub fn get_register(name: &str) -> Option<Registers> {
    match name {
        "a" => Some(Registers::A),
        "b" => Some(Registers::B),
        "c" => Some(Registers::C),
        "d" => Some(Registers::D),
        "exit" => Some(Registers::EXIT),
        "input" => Some(Registers::INPUT),
        "error" => Some(Registers::ERROR),
        "print" => Some(Registers::PRINT),
        "sp" => Some(Registers::STACK_POINTER),
        "pc" => Some(Registers::PROGRAM_COUNTER),
        "zf" => Some(Registers::ZERO_FLAG),
        "sf" => Some(Registers::SIGN_FLAG),
        "rf" => Some(Registers::REMAINDER_FLAG),
        _ => None
    }
}


