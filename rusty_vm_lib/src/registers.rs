use std::mem;
use std::fmt;

use static_assertions::const_assert_eq;

use crate::vm::{REGISTER_ID_SIZE, ErrorCodes, Address};


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

    pub const fn to_bytes(&self) -> [u8; REGISTER_ID_SIZE] {
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

};
}


const_assert_eq!(mem::size_of::<Registers>(), REGISTER_ID_SIZE);


declare_registers! {

    R1 r1,
    R2 r2,
    R3 r3,
    R4 r4,
    R5 r5,
    R6 r6,
    R7 r7,
    R8 r8,

    INPUT input,
    ERROR error,
    PRINT print,
    INTERRUPT int,

    STACK_TOP_POINTER stp,
    PROGRAM_COUNTER pc,
    STACK_FRAME_BASE_POINTER sbp,
    PROGRAM_END_POINTER pep,

    ZERO_FLAG zf,
    SIGN_FLAG sf,
    REMAINDER_FLAG rf,
    CARRY_FLAG cf,
    OVERFLOW_FLAG of

}


impl std::convert::From<u8> for Registers {
    fn from(value: u8) -> Self {
        if value < REGISTER_COUNT as u8 {
            unsafe { std::mem::transmute::<u8, Registers>(value) }
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

#[derive(Clone, Debug, Default)]
pub struct CPURegisters([RegisterContentType; REGISTER_COUNT]);

impl CPURegisters {

    pub fn as_bytes(&self) -> &[u8] {
        unsafe {
            std::slice::from_raw_parts(
                self.0.as_ptr().cast::<u8>(),
                mem::size_of::<CPURegisters>()
            )
        }
    }

    /// Get the value of the given register
    #[inline]
    pub fn get(&self, register: Registers) -> u64 {
        self.0[register as usize]
    }


    /// Get the first `n` bytes of the given register by &-masking
    #[inline]
    pub fn get_masked(&self, register: Registers, n: u8) -> u64 {
        self.get(register)
        & (u64::MAX >> ((mem::size_of::<u64>() as u8 - n) * 8))
    }


    /// Set the value of the given register
    #[inline]
    pub fn set(&mut self, register: Registers, value: u64) {
        self.0[register as usize] = value;
    }


    /// Set the error register
    #[inline]
    pub fn set_error(&mut self, error: ErrorCodes) {
        self.0[Registers::ERROR as usize] = error as u64;
    }


    /// Increment the program counter by the given offset.
    #[inline]
    pub fn inc_pc(&mut self, offset: usize) {
        self.0[Registers::PROGRAM_COUNTER as usize] += offset as u64;
    }


    /// Get the program counter
    #[inline]
    pub fn pc(&self) -> Address {
        self.get(Registers::PROGRAM_COUNTER) as Address
    }


    /// Get the stack top pointer
    #[inline]
    pub fn stack_top(&self) -> Address {
        self.get(Registers::STACK_TOP_POINTER) as Address
    }


    pub fn iter(&self) -> std::slice::Iter<'_, u64> {
        self.0.iter()
    }

}


#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn cpu_registers_as_bytes() {

        let mut regs = CPURegisters::default();

        regs.set(Registers::R1, 57812);
        regs.set(Registers::R2, 0);

        let bytes = regs.as_bytes();

        let r1 = u64::from_le_bytes(bytes[0..8].try_into().unwrap());
        let r2 = u64::from_le_bytes(bytes[8..16].try_into().unwrap());

        assert_eq!(r1, 57812);
        assert_eq!(r2, 0);
    }

}
