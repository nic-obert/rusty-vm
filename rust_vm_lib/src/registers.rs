use std::fmt;

use crate::vm::ADDRESS_SIZE;


pub const REGISTER_ID_SIZE: usize = 1;
pub const REGISTER_SIZE: usize = ADDRESS_SIZE;

pub const REGISTER_COUNT: usize = 17;


#[allow(dead_code, non_camel_case_types)]
#[derive(Clone, Copy, Debug)]
pub enum Registers {
    R1 = 0,
    R2,
    R3,
    R4,
    R5,
    R6,
    R7,
    R8,

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


impl Registers {

    pub fn to_bytes(&self) -> [u8; REGISTER_ID_SIZE] {
        [*self as u8]
    }

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
    "r1",
    "r2",
    "r3",
    "r4",
    "r5",
    "r6",
    "r7",
    "r8",
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
        "r1" => Some(Registers::R1),
        "r2" => Some(Registers::R2),
        "r3" => Some(Registers::R3),
        "r4" => Some(Registers::R4),
        "r5" => Some(Registers::R5),
        "r6" => Some(Registers::R6),
        "r7" => Some(Registers::R7),
        "r8" => Some(Registers::R8),
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


