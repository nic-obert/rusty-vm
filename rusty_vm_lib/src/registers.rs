use std::fmt;

pub const REGISTER_ID_SIZE: usize = 1;
pub type RegisterContentType = u64;
pub const REGISTER_SIZE: usize = std::mem::size_of::<RegisterContentType>();


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

    STACK_TOP_POINTER,
    STACK_BASE_POINTER,
    PROGRAM_COUNTER,

    ZERO_FLAG,
    SIGN_FLAG,
    REMAINDER_FLAG,
    CARRY_FLAG,
    OVERFLOW_FLAG,
}


pub const REGISTER_COUNT: usize = {
    assert!((Registers::OVERFLOW_FLAG as usize) < 256);
    Registers::OVERFLOW_FLAG as usize + 1
};


impl Registers {

    pub fn to_bytes(&self) -> [u8; REGISTER_ID_SIZE] {
        [*self as u8]
    }


    /// Return the register given its name.
    pub fn from_name(name: &str) -> Option<Self> {
        Some(match name {
            "r1" => Registers::R1,
            "r2" => Registers::R2,
            "r3" => Registers::R3,
            "r4" => Registers::R4,
            "r5" => Registers::R5,
            "r6" => Registers::R6,
            "r7" => Registers::R7,
            "r8" => Registers::R8,
            "exit" => Registers::EXIT,
            "input" => Registers::INPUT,
            "error" => Registers::ERROR,
            "print" => Registers::PRINT,
            "stp" => Registers::STACK_TOP_POINTER,
            "sbp" => Registers::STACK_BASE_POINTER,
            "pc" => Registers::PROGRAM_COUNTER,
            "zf" => Registers::ZERO_FLAG,
            "sf" => Registers::SIGN_FLAG,
            "rf" => Registers::REMAINDER_FLAG,
            "cf" => Registers::CARRY_FLAG,
            "of" => Registers::OVERFLOW_FLAG,

            _ => return None
        })
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

    "stp",
    "sbp",
    "pc",

    "zf",
    "sf",
    "rf",
    "cf",
    "of",
];


#[cfg(test)]
mod tests {

    use super::*;


    #[test]
    fn register_names_consistency() {
        for (i, name) in REGISTER_NAMES.iter().enumerate() {
            assert_eq!(Registers::from(i as u8).to_string(), *name);
        }
    }

}

