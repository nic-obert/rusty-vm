
use std::io::Write;

use rust_vm_lib::registers::{Registers, REGISTER_COUNT};
use rust_vm_lib::byte_code::{ByteCodes, BYTE_CODE_COUNT};
use rust_vm_lib::vm::{Address, ADDRESS_SIZE};
use crate::memory::{Memory, Size, Byte};
use crate::errors::ErrorCodes;


fn bytes_to_int(bytes: &[Byte], handled_size: Byte) -> i64 {
    match handled_size {
        1 => bytes[0] as i64,
        2 => unsafe {
            *(bytes.as_ptr() as *const u16) as i64
        },
        4 => unsafe {
            *(bytes.as_ptr() as *const u32) as i64
        },
        8 => unsafe {
            *(bytes.as_ptr() as *const u64) as i64
        },
        _ => panic!("Invalid size for compare: {}", handled_size),
    }
}


pub struct Processor {

    registers: [i64; REGISTER_COUNT],

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
        // Load the bytecode into memory
        self.push_stack_bytes(byte_code);

        self.running = true;
        if verbose {
            self.run_verbose();
        } else {
            self.run();
        }

        // Return the error code of the program at exit
        ErrorCodes::from(self.registers[Registers::ERROR as usize] as u8)
    }


    /// Move the program counter by the given offset.
    #[inline(always)]
    fn move_pc(&mut self, offset: i64) {
        self.registers[Registers::PROGRAM_COUNTER as usize] = self.registers[Registers::PROGRAM_COUNTER as usize] + offset;
    }


    /// Get the program counter
    #[inline(always)]
    fn get_pc(&self) -> Address {
        self.registers[Registers::PROGRAM_COUNTER as usize] as Address
    }


    #[inline(always)]
    fn get_register(&self, register: Registers) -> i64 {
        self.registers[register as usize]
    }


    #[inline(always)]
    fn get_register_mut(&mut self, register: Registers) -> &mut i64 {
        &mut self.registers[register as usize]
    }


    #[inline(always)]
    fn set_register(&mut self, register: Registers, value: i64) {
        self.registers[register as usize] = value;
    }


    #[inline(always)]
    fn set_error(&mut self, error: ErrorCodes) {
        self.registers[Registers::ERROR as usize] = error as i64;
    }


    #[inline(always)]
    fn set_arithmetical_flags(&mut self, result: i64, remainder: i64) {
        self.set_register(Registers::ZERO_FLAG, (result == 0) as i64);
        self.set_register(Registers::SIGN_FLAG, (result < 0) as i64);
        self.set_register(Registers::REMAINDER_FLAG, remainder);
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
        *self.get_register_mut(Registers::STACK_POINTER) += bytes.len() as i64;
    }


    fn push_stack_from_address(&mut self, src_address: Address, size: Size) {
        let dest_address = self.get_register(Registers::STACK_POINTER) as Size;
        self.memory.memcpy(dest_address, src_address, size);
        *self.get_register_mut(Registers::STACK_POINTER) += size as i64;
    }


    fn push_stack(&mut self, value: i64) {
        self.push_stack_bytes(&value.to_le_bytes());
    }


    fn pop_stack_bytes(&mut self, size: Size) -> &[Byte] {
        *self.get_register_mut(Registers::STACK_POINTER) -= size as i64;

        self.memory.get_bytes(
            self.get_register(Registers::STACK_POINTER) as Size,
            size,
        )
    }
        
    
    fn get_next_bytes(&mut self, size: Size) -> &[Byte] {
        let pc = self.get_pc();
        self.move_pc(size as i64);
        self.memory.get_bytes(pc, size)
    }


    fn get_next_byte(&mut self) -> Byte {
        let pc = self.get_pc();
        self.move_pc(1);
        self.memory.get_byte(pc)
    }


    fn run(&mut self) {
        while self.running {
            let opcode = ByteCodes::from(self.get_next_byte());
            self.handle_instruction(opcode);
        }
    }


    fn run_verbose(&mut self) {
        while self.running {
            let opcode = ByteCodes::from(self.get_next_byte());
            println!("PC: {}, opcode: {}", self.get_pc(), opcode);
            self.handle_instruction(opcode);
        }
    }

    
    #[inline(always)]
    fn handle_instruction(&mut self, opcode: ByteCodes) {
        Self::INSTRUCTION_HANDLER_TABLE[opcode as usize](self);
    }


    // Instruction handlers


    fn handle_add(&mut self) {
        *self.get_register_mut(Registers::A) += self.get_register(Registers::B);
        self.set_arithmetical_flags(
            self.get_register(Registers::A), 
            0
        )
    }


    fn handle_sub(&mut self) {
        *self.get_register_mut(Registers::A) -= self.get_register(Registers::B);
        self.set_arithmetical_flags(
            self.get_register(Registers::A), 
            0
        )
    }


    fn handle_mul(&mut self) {
        *self.get_register_mut(Registers::A) *= self.get_register(Registers::B);
        self.set_arithmetical_flags(
            self.get_register(Registers::A), 
            0
        )
    }


    fn handle_div(&mut self) {
        let remainder = self.get_register(Registers::A) % self.get_register(Registers::B);
        *self.get_register_mut(Registers::A) /= self.get_register(Registers::B);
        self.set_arithmetical_flags(
            self.get_register(Registers::A), 
            remainder as i64
        )
    }


    fn handle_mod(&mut self) {
        *self.get_register_mut(Registers::A) %= self.get_register(Registers::B);
        self.set_arithmetical_flags(
            self.get_register(Registers::A), 
            0
        )
    }


    fn handle_inc_reg(&mut self) {
        let dest_reg = Registers::from(self.get_next_byte());
        *self.get_register_mut(dest_reg) += 1;
        self.set_arithmetical_flags(
            self.get_register(dest_reg), 
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
            self.get_register(dest_reg), 
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


    fn handle_move_into_reg_from_reg(&mut self) {
        let dest_reg = Registers::from(self.get_next_byte());
        let source_reg = Registers::from(self.get_next_byte());
        self.set_register(dest_reg, self.get_register(source_reg));
    }


    fn move_bytes_into_register(&mut self, src_address: Address, dest_reg: Registers, handled_size: Byte) {
        let bytes = self.memory.get_bytes(src_address, handled_size as usize);

        match handled_size {
            1 => unsafe {
                self.set_register(dest_reg, *(bytes.as_ptr() as *const u8) as i64);
            },
            2 => unsafe {
                self.set_register(dest_reg, *(bytes.as_ptr() as *const u16) as i64);
            },
            4 => unsafe {
                self.set_register(dest_reg, *(bytes.as_ptr() as *const u32) as i64);
            },
            8 => unsafe {
                self.set_register(dest_reg, *(bytes.as_ptr() as *const u64) as i64);
            },
            _ => panic!("Invalid size for move instruction"),
        }
    }


    fn handle_move_into_reg_from_addr_in_reg(&mut self) {
        let size = self.get_next_byte();
        let dest_reg = Registers::from(self.get_next_byte());
        let address_reg = Registers::from(self.get_next_byte());
        let src_address = self.get_register(address_reg) as Address;

        self.move_bytes_into_register(src_address, dest_reg, size);
    }


    fn handle_move_into_reg_from_const(&mut self) {
        let size = self.get_next_byte();
        let dest_reg = Registers::from(self.get_next_byte());
        let src_address = self.get_pc();

        self.move_bytes_into_register(src_address, dest_reg, size);
        self.move_pc(size as i64);
    }


    fn handle_move_into_reg_from_addr_literal(&mut self) {
        let size = self.get_next_byte();
        let dest_reg = Registers::from(self.get_next_byte());
        let src_address = self.get_next_address();

        self.move_bytes_into_register(src_address, dest_reg, size);
    }


    fn move_from_register_into_address(&mut self, src_reg: Registers, dest_address: Address, handled_size: Byte) {
        let value = self.get_register(src_reg);
        let dest_bytes = self.memory.get_bytes_mut(dest_address, handled_size as usize);

        match handled_size {
            1 => unsafe {
                *(dest_bytes.as_mut_ptr() as *mut u8) = value as u8;
            },
            2 => unsafe {
                *(dest_bytes.as_mut_ptr() as *mut u16) = value as u16;
            },
            4 => unsafe {
                *(dest_bytes.as_mut_ptr() as *mut u32) = value as u32;
            },
            8 => unsafe {
                *(dest_bytes.as_mut_ptr() as *mut u64) = value as u64;
            },
            _ => panic!("Invalid size for move instruction"),
        }
    }


    fn handle_move_into_addr_in_reg_from_reg(&mut self) {
        let size = self.get_next_byte();
        let dest_address_reg = Registers::from(self.get_next_byte());
        let src_reg = Registers::from(self.get_next_byte());
        let dest_address = self.get_register(dest_address_reg) as Address;

        self.move_from_register_into_address(src_reg, dest_address, size);
    }


    fn handle_move_into_addr_in_reg_from_addr_in_reg(&mut self) {
        let size = self.get_next_byte();
        let dest_address_reg = Registers::from(self.get_next_byte());
        let src_address_reg = Registers::from(self.get_next_byte());
        let dest_address = self.get_register(dest_address_reg) as Address;
        let src_address = self.get_register(src_address_reg) as Address;
        
        self.memory.memcpy(src_address, dest_address, size as Size);
    }


    fn handle_move_into_addr_in_reg_from_const(&mut self) {
        let size = self.get_next_byte();
        let dest_address_reg = Registers::from(self.get_next_byte());
        let dest_address = self.get_register(dest_address_reg) as Address;
        let src_address = self.get_pc();
        
        self.memory.memcpy(src_address, dest_address, size as Size);
        self.move_pc(size as i64);
    }


    fn handle_move_into_addr_in_reg_from_addr_literal(&mut self) {
        let size = self.get_next_byte();
        let dest_address_reg = Registers::from(self.get_next_byte());
        let dest_address = self.get_register(dest_address_reg) as Address;
        let src_address = self.get_next_address();

        self.memory.memcpy(src_address, dest_address, size as Size);
    }


    fn handle_move_into_addr_literal_from_reg(&mut self) {
        let size = self.get_next_byte();
        let dest_address = self.get_next_address();
        let src_reg = Registers::from(self.get_next_byte());

        self.move_from_register_into_address(src_reg, dest_address, size);
    }


    fn handle_move_into_addr_literal_from_addr_in_reg(&mut self) {
        let size = self.get_next_byte();
        let dest_address = self.get_next_address();
        let src_address_reg = Registers::from(self.get_next_byte());
        let src_address = self.get_register(src_address_reg) as Address;

        self.memory.memcpy(src_address, dest_address, size as Size);
    }


    fn handle_move_into_addr_literal_from_const(&mut self) {
        let size = self.get_next_byte();
        let dest_address = self.get_next_address();
        let src_address = self.get_pc();

        self.memory.memcpy(src_address, dest_address, size as Size);
        self.move_pc(size as i64);
    }


    fn handle_move_into_addr_literal_from_addr_literal(&mut self) {
        let size = self.get_next_byte();
        let dest_address = self.get_next_address();
        let src_address = self.get_next_address();

        self.memory.memcpy(src_address, dest_address, size as Size);
    }


    fn handle_push_from_reg(&mut self) {
        let src_reg = Registers::from(self.get_next_byte());
        self.push_stack(self.get_register(src_reg));
    }


    fn handle_push_from_addr_in_reg(&mut self) {
        let size = self.get_next_byte();
        let src_address_reg = Registers::from(self.get_next_byte());
        let src_address = self.get_register(src_address_reg) as Address;

        self.push_stack_from_address(src_address, size as usize);
    }


    fn handle_push_from_const(&mut self) {
        let size = self.get_next_byte();
        let src_address = self.get_pc();

        self.push_stack_from_address(src_address, size as Size);
        self.move_pc(size as i64);
    }


    fn handle_push_from_addr_literal(&mut self) {
        let size = self.get_next_byte();
        let src_address = self.get_next_address();

        self.push_stack_from_address(src_address, size as Size);
    }


    fn handle_pop_into_reg(&mut self) {
        let size = self.get_next_byte();
        let dest_reg = Registers::from(self.get_next_byte());
        let bytes = self.pop_stack_bytes(size as Size);

        // Access registers directly to get around the borrow checker
        match size {
            1 => self.registers[dest_reg as usize] = bytes[0] as i64,
            2 => unsafe {
                self.registers[dest_reg as usize] = *(bytes.as_ptr() as *const u16) as i64;
            },
            4 => unsafe {
                self.registers[dest_reg as usize] = *(bytes.as_ptr() as *const u32) as i64;
            },
            8 => unsafe {
                self.registers[dest_reg as usize] = *(bytes.as_ptr() as *const u64) as i64;
            },
            _ => panic!("Invalid size for pop: {}", size),
        }
    }


    fn handle_pop_into_addr_in_reg(&mut self) {
        let size = self.get_next_byte();
        let dest_address_reg = Registers::from(self.get_next_byte());
        let dest_address = self.get_register(dest_address_reg) as Address;
        let src_address = self.get_pc();

        self.memory.memcpy(src_address, dest_address, size as Size);
        self.move_pc(size as i64);
    }


    fn handle_pop_into_addr_literal(&mut self) {
        let size = self.get_next_byte();
        let dest_address = self.get_next_address();
        let src_address = self.get_pc();

        self.memory.memcpy(src_address, dest_address, size as Size);
        self.move_pc(size as i64);
    }


    /// This function is never used, it's only a placeholder to make the lookup table work
    fn handle_label(&mut self) { }


    fn handle_jump(&mut self) {
        let address = self.get_next_address();
        self.set_register(Registers::PROGRAM_COUNTER, address as i64);
    }


    fn handle_jump_if_not_zero_reg(&mut self) {
        let address = self.get_next_address();
        let test_reg = Registers::from(self.get_next_byte());

        if self.get_register(test_reg) != 0 {
            self.set_register(Registers::PROGRAM_COUNTER, address as i64);
        }
    }


    fn handle_jump_if_zero_reg(&mut self) {
        let address = self.get_next_address();
        let test_reg = Registers::from(self.get_next_byte());

        if self.get_register(test_reg) == 0 {
            self.set_register(Registers::PROGRAM_COUNTER, address as i64);
        }
    }


    fn handle_compare_reg_reg(&mut self) {
        let left_reg = Registers::from(self.get_next_byte());
        let right_reg = Registers::from(self.get_next_byte());
    
        self.set_arithmetical_flags(
            self.get_register(left_reg) - self.get_register(right_reg),
            0
        );
    }


    fn handle_compare_reg_const(&mut self) {
        let size = self.get_next_byte();
        let left_reg = Registers::from(self.get_next_byte());
        let right_address = self.get_next_bytes(size as Size);
        let right_value = bytes_to_int(right_address, size);
        
        self.set_arithmetical_flags(
            self.get_register(left_reg) - right_value,
            0
        );
    }


    fn handle_compare_const_reg(&mut self) {
        let size = self.get_next_byte();
        let left_address = self.get_next_bytes(size as Size);
        let left_value = bytes_to_int(left_address, size);
        let right_reg = Registers::from(self.get_next_byte());
        
        self.set_arithmetical_flags(
            left_value - self.get_register(right_reg),
            0
        );
    }


    fn handle_compare_const_const(&mut self) {
        let size = self.get_next_byte();
        let left_address = self.get_next_bytes(size as Size);
        let left_value = bytes_to_int(left_address, size);
        let right_address = self.get_next_bytes(size as Size);
        let right_value = bytes_to_int(right_address, size);
        
        self.set_arithmetical_flags(
            left_value - right_value,
            0
        );
    }


    fn handle_print_signed(&mut self) {
        let value = self.get_register(Registers::PRINT);
        print!("{}", value);
        std::io::stdout().flush().expect("Failed to flush stdout");
    }


    fn handle_print_unsigned(&mut self) {
        let value = self.get_register(Registers::PRINT) as u64;
        print!("{}", value);
        std::io::stdout().flush().expect("Failed to flush stdout");
    }


    fn handle_print_char(&mut self) {
        let value = self.get_register(Registers::PRINT);
        print!("{}", value as u8 as char);
        std::io::stdout().flush().expect("Failed to flush stdout");
    }


    fn handle_print_string(&mut self) {
        let mut address = self.get_register(Registers::PRINT) as Address;
        let mut byte = self.memory.get_byte(address);

        while byte != 0 {
            print!("{}", byte as u8 as char);
            address += 1;
            byte = self.memory.get_byte(address);
        }
        std::io::stdout().flush().expect("Failed to flush stdout");
    }


    fn handle_input_int(&mut self) {
        let mut input = String::new();
        match std::io::stdin().read_line(&mut input) {
            Ok(bytes_read) => {

                // Check for EOF errors
                if bytes_read == 0 {
                    self.set_error(ErrorCodes::EndOfFile);
                    return;
                }

                match input.parse::<i64>() {
                    Ok(value) => {
                        self.set_register(Registers::INPUT, value);
                        self.set_error(ErrorCodes::NoError);
                    },
                    Err(_) => {
                        self.set_error(ErrorCodes::InvalidInput);
                    }
                }
            },
            Err(_) => {
                self.set_error(ErrorCodes::GenericError);
            },
        }
    }


    fn handle_input_string(&mut self) {
        let mut input = String::new();
        match std::io::stdin().read_line(&mut input) {
            Ok(bytes_read) => {

                // Check for EOF errors
                if bytes_read == 0 {
                    self.set_error(ErrorCodes::EndOfFile);
                    return;
                }

                self.set_register(Registers::INPUT, input.len() as i64);
                self.push_stack_bytes(input.as_bytes());
                self.set_error(ErrorCodes::NoError); 
            },
            Err(_) => {
                self.set_error(ErrorCodes::GenericError);
            },
        }
    }


    fn handle_exit(&mut self) {
        self.running = false;
    }


    const INSTRUCTION_HANDLER_TABLE: [ fn(&mut Self); BYTE_CODE_COUNT ] = [
        Self::handle_add,
        Self::handle_sub,
        Self::handle_mul,
        Self::handle_div,
        Self::handle_mod,

        Self::handle_inc_reg,
        Self::handle_inc_addr_in_reg,
        Self::handle_inc_addr_literal,

        Self::handle_dec_reg,
        Self::handle_dec_addr_in_reg,
        Self::handle_dec_addr_literal,

        Self::handle_no_operation,

        Self::handle_move_into_reg_from_reg,
        Self::handle_move_into_reg_from_addr_in_reg,
        Self::handle_move_into_reg_from_const,
        Self::handle_move_into_reg_from_addr_literal,
        Self::handle_move_into_addr_in_reg_from_reg,
        Self::handle_move_into_addr_in_reg_from_addr_in_reg,
        Self::handle_move_into_addr_in_reg_from_const,
        Self::handle_move_into_addr_in_reg_from_addr_literal,
        Self::handle_move_into_addr_literal_from_reg,
        Self::handle_move_into_addr_literal_from_addr_in_reg,
        Self::handle_move_into_addr_literal_from_const,
        Self::handle_move_into_addr_literal_from_addr_literal,

        Self::handle_push_from_reg,
        Self::handle_push_from_addr_in_reg,
        Self::handle_push_from_const,
        Self::handle_push_from_addr_literal,

        Self::handle_pop_into_reg,
        Self::handle_pop_into_addr_in_reg,
        Self::handle_pop_into_addr_literal,

        Self::handle_label,

        Self::handle_jump,
        Self::handle_jump_if_not_zero_reg,
        Self::handle_jump_if_zero_reg,

        Self::handle_compare_reg_reg,
        Self::handle_compare_reg_const,
        Self::handle_compare_const_reg,
        Self::handle_compare_const_const,

        Self::handle_print_signed,
        Self::handle_print_unsigned,
        Self::handle_print_char,
        Self::handle_print_string,

        Self::handle_input_int,
        Self::handle_input_string,

        Self::handle_exit,
    ];


}

