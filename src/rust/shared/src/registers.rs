use std::fmt;


#[derive(Clone, Copy)]
pub enum Registers {
    A,
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


pub static REGISTER_NAMES: [&str; 13] = [
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


pub fn get_register(name: &str) -> Registers {
    match name {
        "a" => Registers::A,
        "b" => Registers::B,
        "c" => Registers::C,
        "d" => Registers::D,
        "exit" => Registers::EXIT,
        "input" => Registers::INPUT,
        "error" => Registers::ERROR,
        "print" => Registers::PRINT,
        "sp" => Registers::STACK_POINTER,
        "pc" => Registers::PROGRAM_COUNTER,
        "zf" => Registers::ZERO_FLAG,
        "sf" => Registers::SIGN_FLAG,
        "rf" => Registers::REMAINDER_FLAG,
        _ => panic!("Unknown register name: {}", name)
    }
}


