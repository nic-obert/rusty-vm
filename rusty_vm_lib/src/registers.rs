use std::mem;
use std::fmt;

pub const REGISTER_ID_SIZE: usize = 1;
pub type RegisterContentType = u64;
pub const REGISTER_SIZE: usize = std::mem::size_of::<RegisterContentType>();


macro_rules! declare_registers {
    ($($name:ident $repr:ident),+) => {

#[allow(dead_code, non_camel_case_types)]
#[derive(Clone, Copy, Debug)]
pub enum Registers {
    $(
        $name,
    )+
}

pub const REGISTER_COUNT: usize = mem::variant_count::<Registers>();

pub const GENERAL_PURPOSE_REGISTER_COUNT: usize = {
    assert!((Registers::R8 as usize) < 8);
    Registers::R8 as usize + 1
};

impl Registers {

    pub  const fn to_bytes(&self) -> [u8; REGISTER_ID_SIZE] {
        [*self as u8]
    }


    /// Return the register given its name.
    pub fn from_name(name: &str) -> Option<Self> {
        Some(match name {
            $(stringify!($repr) => Self::$name,)+

            _ => return None
        })
    }

    pub const fn name(&self) -> &'static str {
        match self {
            $(Self::$name => stringify!($repr)),+
        }
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
        write!(f, "{}", self.name())
    }
}

    };
}

declare_registers! {

    R1 r1,
    R2 r2,
    R3 r3,
    R4 r4,
    R5 r5,
    R6 r6,
    R7 r7,
    R8 r8,

    EXIT exit,
    INPUT input,
    ERROR error,
    PRINT print,

    STACK_TOP_POINTER stp,
    PROGRAM_COUNTER pc,

    ZERO_FLAG zf,
    SIGN_FLAG sf,
    REMAINDER_FLAG rf,
    CARRY_FLAG cf,
    OVERFLOW_FLAG of

}

