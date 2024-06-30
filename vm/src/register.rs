use std::mem;

use rusty_vm_lib::registers::{REGISTER_COUNT, RegisterContentType, Registers};
use rusty_vm_lib::vm::{ErrorCodes, Address};


pub struct CPURegisters([RegisterContentType; REGISTER_COUNT]);

impl CPURegisters {

    pub fn new() -> Self {
        CPURegisters([0; REGISTER_COUNT])
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

