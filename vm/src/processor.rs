
use rust_vm_lib::registers::{Registers, REGISTER_COUNT};
use crate::memory::{Memory, Size, Byte, Address, ADDRESS_SIZE};
use crate::errors::ErrorCodes;


pub struct Processor {

    registers: [u64; REGISTER_COUNT],

    memory: Memory,

    running: bool,

}


impl Processor {

    pub fn new(stack_size: Size, video_size: Size) -> Self {
        Self {
            registers: [0; REGISTER_COUNT],
            memory: Memory::new(stack_size, video_size),
            running: false,
        }
    }


    pub fn execute(&mut self, byte_code: &[Byte], verbose: bool) -> ErrorCodes {
        todo!()
    }


    #[inline(always)]
    fn get_register(&self, register: Registers) -> u64 {
        self.registers[register as usize]
    }


    #[inline(always)]
    fn get_register_mut(&mut self, register: Registers) -> &mut u64 {
        &mut self.registers[register as usize]
    }


    #[inline(always)]
    fn set_register(&mut self, register: Registers, value: u64) {
        self.registers[register as usize] = value;
    }


    #[inline(always)]
    fn set_arithmetical_flags(&mut self, result: i64, remainder: i64) {
        self.set_register(Registers::ZERO_FLAG, (result == 0) as u64);
        self.set_register(Registers::SIGN_FLAG, (result < 0) as u64);
        self.set_register(Registers::REMAINDER_FLAG, remainder as u64);
    }


    #[inline(always)]
    fn get_next_address(&mut self) -> Address {
        unsafe {
            * (self.get_next_bytes(ADDRESS_SIZE).as_ptr() as *const Address)
        }
    }


    fn increment_bytes(&mut self, address: Address, size: Byte) {
        let bytes = self.memory.get_bytes_mut(address, size as Size);

        match size {
            1 => unsafe {
                *(bytes.as_mut_ptr() as *mut u8) += 1;
                let result = *(bytes.as_ptr() as *const i8);
                self.set_arithmetical_flags(
                    result as i64,
                    0
                );
            },
            2 => unsafe {
                *(bytes.as_mut_ptr() as *mut u16) += 1;
                let result = *(bytes.as_ptr() as *const i16);
                self.set_arithmetical_flags(
                    result as i64,
                    0
                );
            },
            4 => unsafe {
                *(bytes.as_mut_ptr() as *mut u32) += 1;
                let result = *(bytes.as_ptr() as *const i32);
                self.set_arithmetical_flags(
                    result as i64,
                    0
                );
            },
            8 => unsafe {
                *(bytes.as_mut_ptr() as *mut u64) += 1;
                let result = *(bytes.as_ptr() as *const i64);
                self.set_arithmetical_flags(
                    result,
                    0
                );
            },
            _ => panic!("Invalid size for incrementing bytes"),
        }
    }
    
    
    fn decrement_bytes(&mut self, address: Address, size: Byte) {
        let bytes = self.memory.get_bytes_mut(address, size as Size);

        match size {
            1 => unsafe {
                *(bytes.as_mut_ptr() as *mut u8) -= 1;
                let result = *(bytes.as_ptr() as *const i8);
                self.set_arithmetical_flags(
                    result as i64,
                    0
                );
            },
            2 => unsafe {
                *(bytes.as_mut_ptr() as *mut u16) -= 1;
                let result = *(bytes.as_ptr() as *const i16);
                self.set_arithmetical_flags(
                    result as i64,
                    0
                );
            },
            4 => unsafe {
                *(bytes.as_mut_ptr() as *mut u32) -= 1;
                let result = *(bytes.as_ptr() as *const i32);
                self.set_arithmetical_flags(
                    result as i64,
                    0
                );
            },
            8 => unsafe {
                *(bytes.as_mut_ptr() as *mut u64) -= 1;
                let result = *(bytes.as_ptr() as *const i64);
                self.set_arithmetical_flags(
                    result,
                    0
                );
            },
            _ => panic!("Invalid size for decrementing bytes"),
        }
    }


    fn push_stack_bytes(&mut self, bytes: &[Byte]) {
        self.memory.set_bytes(
            self.get_register(Registers::STACK_POINTER) as Size,
            bytes,
        );
        *self.get_register_mut(Registers::STACK_POINTER) += bytes.len() as u64;
    }


    fn push_stack(&mut self, value: u64) {
        self.push_stack_bytes(&value.to_le_bytes());
    }


    fn pop_stack_bytes(&mut self, size: Size) -> &[Byte] {
        *self.get_register_mut(Registers::STACK_POINTER) -= size as u64;

        self.memory.get_bytes(
            self.get_register(Registers::STACK_POINTER) as Size,
            size,
        )
    }
        
    
    fn get_next_bytes(&mut self, size: Size) -> &[Byte] {
        let pc = self.get_register(Registers::PROGRAM_COUNTER) as Size;
        *self.get_register_mut(Registers::PROGRAM_COUNTER) += size as u64;
        self.memory.get_bytes(pc, size)
    }


    fn get_next_byte(&mut self) -> Byte {
        let pc = self.get_register(Registers::PROGRAM_COUNTER) as Size;
        *self.get_register_mut(Registers::PROGRAM_COUNTER) += 1;
        self.memory.get_byte(pc)
    }


    fn run(&mut self) {
        todo!()
    }


    fn run_verbose(&mut self) {
        todo!()
    }


    fn handle_add(&mut self) {
        *self.get_register_mut(Registers::A) += self.get_register(Registers::B);
        self.set_arithmetical_flags(
            self.get_register(Registers::A) as i64, 
            0
        )
    }


    fn handle_sub(&mut self) {
        *self.get_register_mut(Registers::A) -= self.get_register(Registers::B);
        self.set_arithmetical_flags(
            self.get_register(Registers::A) as i64, 
            0
        )
    }


    fn handle_mul(&mut self) {
        *self.get_register_mut(Registers::A) *= self.get_register(Registers::B);
        self.set_arithmetical_flags(
            self.get_register(Registers::A) as i64, 
            0
        )
    }


    fn handle_div(&mut self) {
        let remainder = self.get_register(Registers::A) % self.get_register(Registers::B);
        *self.get_register_mut(Registers::A) /= self.get_register(Registers::B);
        self.set_arithmetical_flags(
            self.get_register(Registers::A) as i64, 
            remainder as i64
        )
    }


    fn handle_mod(&mut self) {
        *self.get_register_mut(Registers::A) %= self.get_register(Registers::B);
        self.set_arithmetical_flags(
            self.get_register(Registers::A) as i64, 
            0
        )
    }


    fn handle_inc_reg(&mut self) {
        let dest_reg = Registers::from(self.get_next_byte());
        *self.get_register_mut(dest_reg) += 1;
        self.set_arithmetical_flags(
            self.get_register(dest_reg) as i64, 
            0
        )
    }


    fn handle_inc_addr_in_reg(&mut self) {
        let size = self.get_next_byte();
        let address_reg = Registers::from(self.get_next_byte());
        let address: Address = self.get_register(address_reg) as Address;
        
        self.increment_bytes(address, size);
    }


    fn handle_inc_addr_literal(&mut self) {
        let size = self.get_next_byte();
        let dest_address = self.get_next_address();
        
        self.increment_bytes(dest_address, size);
    }


    fn handle_dec_reg(&mut self) {
        let dest_reg = Registers::from(self.get_next_byte());
        *self.get_register_mut(dest_reg) -= 1;
        self.set_arithmetical_flags(
            self.get_register(dest_reg) as i64, 
            0
        )
    }


    fn handle_dec_addr_in_reg(&mut self) {
        let size = self.get_next_byte();
        let address_reg = Registers::from(self.get_next_byte());
        let address: Address = self.get_register(address_reg) as Address;
        
        self.decrement_bytes(address, size);
    }


    fn handle_dec_addr_literal(&mut self) {
        let size = self.get_next_byte();
        let dest_address = self.get_next_address();
        
        self.decrement_bytes(dest_address, size);
    }


    fn handle_no_operation(&mut self) {
        // Do nothing
    }

}

