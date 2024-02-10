use rusty_vm_lib::registers::{REGISTER_COUNT, RegisterContentType, Registers};
use rusty_vm_lib::vm::{ErrorCodes, Address};


pub struct CPURegisters([RegisterContentType; REGISTER_COUNT]);


impl CPURegisters {

    pub fn new() -> Self {
        CPURegisters([0; REGISTER_COUNT])
    }


    /// Get the value of the given register
    #[inline(always)]
    pub fn get(&self, register: Registers) -> u64 {
        self.0[register as usize]
    }


    /// Set the value of the given register
    #[inline(always)]
    pub fn set(&mut self, register: Registers, value: u64) {
        self.0[register as usize] = value;
    }


    /// Set the error register
    #[inline(always)]
    pub fn set_error(&mut self, error: ErrorCodes) {
        self.0[Registers::ERROR as usize] = error as u64;
    }


    /// Increment the program counter by the given offset.
    #[inline(always)]
    pub fn inc_pc(&mut self, offset: usize) {
        self.0[Registers::PROGRAM_COUNTER as usize] += offset as u64;
    }


    /// Get the program counter
    #[inline(always)]
    pub fn pc(&self) -> Address {
        self.0[Registers::PROGRAM_COUNTER as usize] as Address
    }


    /// Get the stack base pointer
    #[inline(always)]
    pub fn stack_base(&self) -> Address {
        self.0[Registers::STACK_BASE_POINTER as usize] as Address
    }


    /// Get the stack top pointer
    #[inline(always)]
    pub fn stack_top(&self) -> Address {
        self.0[Registers::STACK_TOP_POINTER as usize] as Address
    }


    pub fn iter(&self) -> std::slice::Iter<'_, u64> {
        self.0.iter()
    }

}

