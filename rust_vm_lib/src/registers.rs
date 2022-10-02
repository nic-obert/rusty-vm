use std::fmt;


pub const REGISTER_COUNT: usize = 13;

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


impl std::convert::From<u8> for Registers {

    fn from(value: u8) -> Self {
        if value < REGISTER_COUNT as u8 {
            unsafe { std::mem::transmute(value) }
        } else {
            panic!("Invalid register number: {}", value);
        }
    }

}


impl fmt::Display for Registers {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", REGISTER_NAMES[*self as usize])
    }
}


pub const REGISTER_NAMES: [&str; REGISTER_COUNT] = [
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


