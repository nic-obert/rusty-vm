use std::io::Write;
use std::io;

use rust_vm_lib::registers::{Registers, REGISTER_COUNT, self};
use rust_vm_lib::byte_code::{ByteCodes, BYTE_CODE_COUNT};
use rust_vm_lib::vm::{Address, ADDRESS_SIZE};

use crate::memory::{Memory, Size, Byte};
use crate::errors::ErrorCodes;
use crate::cli_parser::ExecutionMode;



/// Converts a byte array to an integer
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


/// Interprets the given bytes as an address
/// 
/// The byte array must be 8 bytes long
#[inline(always)]
fn bytes_as_address(bytes: &[Byte]) -> Address {
    unsafe {
        *(bytes.as_ptr() as *const Address)
    }
}


pub struct Processor {

    registers: [i64; REGISTER_COUNT],
    memory: Memory,

}


impl Processor {

    pub fn new(stack_size: Size, video_size: Size) -> Self {
        Self {
            registers: [0; REGISTER_COUNT],
            memory: Memory::new(stack_size, video_size),
        }
    }


    /// Execute the given bytecode
    pub fn execute(&mut self, byte_code: &[Byte], mode: ExecutionMode) {

        // Load the program into memory
        self.push_stack_bytes(byte_code);

        // Set the program counter to the start of the program

        if byte_code.len() < ADDRESS_SIZE {
            panic!("Bytecode is too small to contain a start address: minimum required size is {} bytes, got {}", ADDRESS_SIZE, byte_code.len());
        }

        let program_start: Address = bytes_as_address(&byte_code[byte_code.len() - ADDRESS_SIZE..]);
        self.set_register(Registers::PROGRAM_COUNTER, program_start as i64);

        match mode {
            ExecutionMode::Normal => self.run(),
            ExecutionMode::Verbose => self.run_verbose(),
            ExecutionMode::Interactive => self.run_interactive(byte_code.len()),
        }
            
    }


    /// Move the program counter by the given offset.
    #[inline(always)]
    fn move_pc(&mut self, offset: i64) {
        self.registers[Registers::PROGRAM_COUNTER as usize] += offset;
    }


    /// Get the program counter
    #[inline(always)]
    fn get_pc(&self) -> Address {
        self.registers[Registers::PROGRAM_COUNTER as usize] as Address
    }


    /// Get the stack pointer
    #[inline(always)]
    fn get_sp(&self) -> Address {
        self.registers[Registers::STACK_POINTER as usize] as Address
    }


    /// Get the value of the given register
    #[inline(always)]
    fn get_register(&self, register: Registers) -> i64 {
        self.registers[register as usize]
    }


    /// Get a mutable reference to the given register
    #[inline(always)]
    fn get_register_mut(&mut self, register: Registers) -> &mut i64 {
        &mut self.registers[register as usize]
    }


    /// Set the value of the given register
    #[inline(always)]
    fn set_register(&mut self, register: Registers, value: i64) {
        self.registers[register as usize] = value;
    }


    /// Set the error register
    #[inline(always)]
    fn set_error(&mut self, error: ErrorCodes) {
        self.registers[Registers::ERROR as usize] = error as i64;
    }


    /// Update the arithmetical register flags based on the given operation result
    #[inline(always)]
    fn set_arithmetical_flags(&mut self, result: i64, remainder: i64) {
        self.set_register(Registers::ZERO_FLAG, (result == 0) as i64);
        self.set_register(Registers::SIGN_FLAG, (result < 0) as i64);
        self.set_register(Registers::REMAINDER_FLAG, remainder);
    }


    /// Get the next address in the bytecode
    #[inline(always)]
    fn get_next_address(&mut self) -> Address {
        bytes_as_address(
            self.get_next_bytes(ADDRESS_SIZE)
        )
    }


    /// Increment the `size`-sized value at the given address
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
            _ => panic!("Invalid size for incrementing bytes: {}", size),
        }
    }
    
    
    /// Decrement the `size`-sized value at the given address
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
            _ => panic!("Invalid size for decrementing bytes: {}", size),
        }
    }


    /// Copy the bytes onto the stack
    fn push_stack_bytes(&mut self, bytes: &[Byte]) {
        // Copy the bytes onto the stack
        self.memory.set_bytes(
            self.get_register(Registers::STACK_POINTER) as Size,
            bytes,
        );
        // Move the stack pointer
        self.registers[Registers::STACK_POINTER as usize] += bytes.len() as i64;
    }


    /// Copy the bytes at the given address onto the stack
    fn push_stack_from_address(&mut self, src_address: Address, size: Size) {
        // Get the tos
        let dest_address = self.get_register(Registers::STACK_POINTER) as Size;
        // Copy the bytes onto the stack
        self.memory.memcpy(dest_address, src_address, size);
        // Move the stack pointer
        *self.get_register_mut(Registers::STACK_POINTER) += size as i64;
    }


    /// Push an 8-byte value onto the stack
    fn push_stack(&mut self, value: i64) {
        self.push_stack_bytes(&value.to_le_bytes());
    }


    /// Pop `size` bytes from the stack
    fn pop_stack_bytes(&mut self, size: Size) -> &[Byte] {
        // Move the stack pointer
        *self.get_register_mut(Registers::STACK_POINTER) -= size as i64;

        // Return the tos
        self.memory.get_bytes(
            self.get_register(Registers::STACK_POINTER) as Size,
            size,
        )
    }
        
    
    /// Get the next `size` bytes in the bytecode
    fn get_next_bytes(&mut self, size: Size) -> &[Byte] {
        let pc = self.get_pc();
        self.move_pc(size as i64);
        self.memory.get_bytes(pc, size)
    }


    /// Get the next byte in the bytecode
    fn get_next_byte(&mut self) -> Byte {
        let pc = self.get_pc();
        self.move_pc(1);
        self.memory.get_byte(pc)
    }


    fn run(&mut self) {
        loop {
            let opcode = ByteCodes::from(self.get_next_byte());
            self.handle_instruction(opcode);
        }
    }


    fn run_interactive(&mut self, byte_code_size: usize) {

        println!("Running VM in interactive mode");
        println!("Byte code size is {} bytes", byte_code_size);
        println!("Start address is: {}", self.get_pc());
        println!("");

        loop {
            let opcode = ByteCodes::from(self.get_next_byte());

            println!("PC: {}, opcode: {}", self.get_pc(), opcode);
            println!("Registers: {}", self.display_registers());

            let max_stack_view_range = 32;
            // Don't print the program text section
            let lower_bound = if self.get_sp() - byte_code_size > max_stack_view_range { self.get_sp() - max_stack_view_range } else { byte_code_size };
            println!(
                "Stack: #{} {:?} #{}",
                lower_bound, &self.memory.get_raw()[lower_bound .. self.get_sp()], self.get_sp()
            );

            io::stdin().read_line(&mut String::new()).unwrap();

            self.handle_instruction(opcode);
        }
    }


    fn display_registers(&self) -> String {
        (0..REGISTER_COUNT).map(
            |i| format!("{}: {}, ", registers::REGISTER_NAMES[i], self.registers[i])
        ).collect()
    }


    fn run_verbose(&mut self) {
        loop {
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
        *self.get_register_mut(Registers::R1) += self.get_register(Registers::R2);
        self.set_arithmetical_flags(
            self.get_register(Registers::R1), 
            0
        )
    }


    fn handle_sub(&mut self) {
        *self.get_register_mut(Registers::R1) -= self.get_register(Registers::R2);
        self.set_arithmetical_flags(
            self.get_register(Registers::R1), 
            0
        )
    }


    fn handle_mul(&mut self) {
        *self.get_register_mut(Registers::R1) *= self.get_register(Registers::R2);
        self.set_arithmetical_flags(
            self.get_register(Registers::R1), 
            0
        )
    }


    fn handle_div(&mut self) {
        let remainder = self.get_register(Registers::R1) % self.get_register(Registers::R2);
        *self.get_register_mut(Registers::R1) /= self.get_register(Registers::R2);
        self.set_arithmetical_flags(
            self.get_register(Registers::R1), 
            remainder as i64
        )
    }


    fn handle_mod(&mut self) {
        *self.get_register_mut(Registers::R1) %= self.get_register(Registers::R2);
        self.set_arithmetical_flags(
            self.get_register(Registers::R1), 
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
            _ => panic!("Invalid size for move instruction: {}", handled_size),
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

        self.push_stack_from_address(src_address, size as Size);
    }


    fn handle_push_from_const(&mut self) {
        let size = self.get_next_byte();

        // Hack to get around the borrow checker
        self.push_stack_from_address(self.get_pc(), size as Size);
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
    fn handle_label(&mut self) { 
        unreachable!()
    }


    #[inline(always)]
    fn jump_to(&mut self, address: Address) {
        self.set_register(Registers::PROGRAM_COUNTER, address as i64);
    }


    fn handle_jump_to_reg(&mut self) {
        let address_reg = Registers::from(self.get_next_byte());
        let jump_address = self.get_register(address_reg) as Address;

        self.jump_to(jump_address);
    }


    fn handle_jump_to_addr_in_reg(&mut self) {
        let address_reg = Registers::from(self.get_next_byte());
        let address = self.get_register(address_reg) as Address;
        let jump_address = bytes_as_address(
            self.memory.get_bytes(address, ADDRESS_SIZE)
        );

        self.jump_to(jump_address);
    }


    fn handle_jump_to_const(&mut self) {
        let jump_address = self.get_next_address();

        self.jump_to(jump_address);
    }


    fn handle_jump_to_addr_literal(&mut self) {
        let address = self.get_next_address();
        let jump_address = bytes_as_address(
            self.memory.get_bytes(address, ADDRESS_SIZE)
        );

        self.jump_to(jump_address);
    }

    
    fn handle_jump_if_not_zero_reg_to_reg(&mut self) {
        let address_reg = Registers::from(self.get_next_byte());
        let jump_address = self.get_register(address_reg) as Address;

        let test_reg = Registers::from(self.get_next_byte());

        if self.get_register(test_reg) != 0 {
            self.jump_to(jump_address);
        }
    }


    fn handle_jump_if_not_zero_to_addr_in_reg(&mut self) {
        let address_reg = Registers::from(self.get_next_byte());
        let address = self.get_register(address_reg) as Address;
        let jump_address = unsafe {
            *(self.memory.get_bytes(address, ADDRESS_SIZE).as_ptr() as *const Address)
        };

        let test_reg = Registers::from(self.get_next_byte());

        if self.get_register(test_reg) != 0 {
            self.jump_to(jump_address);
        }
    }


    fn handle_jump_if_not_zero_to_const(&mut self) {
        let jump_address = self.get_next_address();

        let test_reg = Registers::from(self.get_next_byte());

        if self.get_register(test_reg) != 0 {
            self.jump_to(jump_address);
        }
    }


    fn handle_jump_if_not_zero_to_addr_literal(&mut self) {
        let address = self.get_next_address();
        let jump_address = bytes_as_address(
            self.memory.get_bytes(address, ADDRESS_SIZE)
        );

        let test_reg = Registers::from(self.get_next_byte());

        if self.get_register(test_reg) != 0 {
            self.jump_to(jump_address);
        }
    }

    
    fn handle_jump_if_zero_to_reg(&mut self) {
        let address_reg = Registers::from(self.get_next_byte());
        let jump_address = self.get_register(address_reg) as Address;

        let test_reg = Registers::from(self.get_next_byte());

        if self.get_register(test_reg) == 0 {
            self.jump_to(jump_address);
        }
    }


    fn handle_jump_if_zero_to_addr_in_reg(&mut self) {
        let address_reg = Registers::from(self.get_next_byte());
        let address = self.get_register(address_reg) as Address;
        let jump_address = bytes_as_address(
            self.memory.get_bytes(address, ADDRESS_SIZE)
        );

        let test_reg = Registers::from(self.get_next_byte());

        if self.get_register(test_reg) == 0 {
            self.jump_to(jump_address);
        }
    }


    fn handle_jump_if_zero_to_const(&mut self) {
        let jump_address = self.get_next_address();

        let test_reg = Registers::from(self.get_next_byte());

        if self.get_register(test_reg) == 0 {
            self.jump_to(jump_address);
        }
    }


    fn handle_jump_if_zero_to_addr_literal(&mut self) {
        let address = self.get_next_address();
        let jump_address = bytes_as_address(
            self.memory.get_bytes(address, ADDRESS_SIZE)
        );

        let test_reg = Registers::from(self.get_next_byte());

        if self.get_register(test_reg) == 0 {
            self.jump_to(jump_address);
        }
    }


    fn handle_call_reg(&mut self) {
        let address_reg = Registers::from(self.get_next_byte());
        let jump_address = self.get_register(address_reg) as Address;

        // Push the return address onto the stack
        self.push_stack(self.get_pc() as i64);

        // Jump to the subroutine
        self.jump_to(jump_address);
    }


    fn handle_call_addr_in_reg(&mut self) {
        let address_reg = Registers::from(self.get_next_byte());
        let address = self.get_register(address_reg) as Address;
        let jump_address = bytes_as_address(
            self.memory.get_bytes(address, ADDRESS_SIZE)
        );

        // Push the return address onto the stack
        self.push_stack(self.get_pc() as i64);

        // Jump to the subroutine
        self.jump_to(jump_address);
    }


    fn handle_call_const(&mut self) {
        let jump_address = self.get_next_address();

        // Push the return address onto the stack
        self.push_stack(self.get_pc() as i64);

        // Jump to the subroutine
        self.jump_to(jump_address);
    }


    fn handle_call_addr_literal(&mut self) {
        let address = self.get_next_address();
        let jump_address = bytes_as_address(
            self.memory.get_bytes(address, ADDRESS_SIZE)
        );

        // Push the return address onto the stack
        self.push_stack(self.get_pc() as i64);

        // Jump to the subroutine
        self.jump_to(jump_address);
    }


    fn handle_return(&mut self) {
        // Get the return address from the stack
        let return_address = bytes_as_address(
            self.pop_stack_bytes(ADDRESS_SIZE)
        );

        // Jump to the return address
        self.jump_to(return_address);
    }


    fn handle_compare_reg_reg(&mut self) {
        let left_reg = Registers::from(self.get_next_byte());

        let right_reg = Registers::from(self.get_next_byte());
    
        self.set_arithmetical_flags(
            self.get_register(left_reg) - self.get_register(right_reg),
            0
        );
    }


    fn handle_compare_reg_addr_in_reg(&mut self) {
        let size = self.get_next_byte();

        let left_reg = Registers::from(self.get_next_byte());
        let left_value = self.get_register(left_reg);

        let right_address_reg = Registers::from(self.get_next_byte());
        let right_address = self.get_register(right_address_reg) as Address;
        let right_value = bytes_to_int(self.memory.get_bytes(right_address, size as usize), size);

        self.set_arithmetical_flags(
            left_value - right_value,
            0
        );
    }


    fn handle_compare_reg_const(&mut self) {
        let size = self.get_next_byte();

        let left_reg = Registers::from(self.get_next_byte());
        let left_value = self.get_register(left_reg);

        let right_value = bytes_to_int(self.get_next_bytes(size as usize), size);

        self.set_arithmetical_flags(
            left_value - right_value,
            0
        );
    }


    fn handle_compare_reg_addr_literal(&mut self) {
        let size = self.get_next_byte();

        let left_reg = Registers::from(self.get_next_byte());
        let left_value = self.get_register(left_reg);

        let right_address = self.get_next_address();
        let right_value = bytes_to_int(self.memory.get_bytes(right_address, size as usize), size);

        self.set_arithmetical_flags(
            left_value - right_value,
            0
        );
    }


    fn handle_compare_addr_in_reg_reg(&mut self) {
        let size = self.get_next_byte();

        let left_address_reg = Registers::from(self.get_next_byte());
        let left_address = self.get_register(left_address_reg) as Address;
        let left_value = bytes_to_int(self.memory.get_bytes(left_address, ADDRESS_SIZE), size);
        
        let right_reg = Registers::from(self.get_next_byte());
        let right_value = self.get_register(right_reg);

        self.set_arithmetical_flags(
            left_value - right_value,
            0
        );
    }


    fn handle_compare_addr_in_reg_addr_in_reg(&mut self) {
        let size = self.get_next_byte();

        let left_address_reg = Registers::from(self.get_next_byte());
        let left_address = self.get_register(left_address_reg) as Address;
        let left_value = bytes_to_int(self.memory.get_bytes(left_address, ADDRESS_SIZE), size);
        
        let right_address_reg = Registers::from(self.get_next_byte());
        let right_address = self.get_register(right_address_reg) as Address;
        let right_value = bytes_to_int(self.memory.get_bytes(right_address, size as usize), size);

        self.set_arithmetical_flags(
            left_value - right_value,
            0
        );
    }


    fn handle_compare_addr_in_reg_const(&mut self) {
        let size = self.get_next_byte();

        let left_address_reg = Registers::from(self.get_next_byte());
        let left_address = self.get_register(left_address_reg) as Address;
        let left_value = bytes_to_int(self.memory.get_bytes(left_address, ADDRESS_SIZE), size);
       
        let right_value = bytes_to_int(self.get_next_bytes(size as usize), size);

        self.set_arithmetical_flags(
            left_value - right_value,
            0
        );
    }


    fn handle_compare_addr_in_reg_addr_literal(&mut self) {
        let size = self.get_next_byte();

        let left_address_reg = Registers::from(self.get_next_byte());
        let left_address = self.get_register(left_address_reg) as Address;
        let left_value = bytes_to_int(self.memory.get_bytes(left_address, ADDRESS_SIZE), size);
       
        let right_address = self.get_next_address();
        let right_value = bytes_to_int(self.memory.get_bytes(right_address, size as usize), size);

        self.set_arithmetical_flags(
            left_value - right_value,
            0
        );
    }


    fn handle_compare_const_reg(&mut self) {
        let size = self.get_next_byte();

        let left_address = self.get_next_bytes(size as Size);
        let left_value = bytes_to_int(left_address, size);

        let right_reg = Registers::from(self.get_next_byte());
        let right_value = self.get_register(right_reg);
        
        self.set_arithmetical_flags(
            left_value - right_value,
            0
        );
    }


    fn handle_compare_const_addr_in_reg(&mut self) {
        let size = self.get_next_byte();

        let left_address = self.get_next_bytes(size as Size);
        let left_value = bytes_to_int(left_address, size);

        let right_address_reg = Registers::from(self.get_next_byte());
        let right_address = self.get_register(right_address_reg) as Address;
        let right_value = bytes_to_int(self.memory.get_bytes(right_address, size as usize), size);

        self.set_arithmetical_flags(
            left_value - right_value,
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


    fn handle_compare_const_addr_literal(&mut self) {
        let size = self.get_next_byte();

        let left_address = self.get_next_bytes(size as Size);
        let left_value = bytes_to_int(left_address, size);

        let right_address = self.get_next_address();
        let right_value = bytes_to_int(self.memory.get_bytes(right_address, size as usize), size);

        self.set_arithmetical_flags(
            left_value - right_value,
            0
        );
    }


    fn handle_compare_addr_literal_reg(&mut self) {
        let size = self.get_next_byte();

        let left_address = self.get_next_address();
        let left_value = bytes_to_int(self.memory.get_bytes(left_address, size as usize), size);

        let right_reg = Registers::from(self.get_next_byte());
        let right_value = self.get_register(right_reg);

        self.set_arithmetical_flags(
            left_value - right_value,
            0
        );
    }


    fn handle_compare_addr_literal_addr_in_reg(&mut self) {
        let size = self.get_next_byte();

        let left_address = self.get_next_address();
        let left_value = bytes_to_int(self.memory.get_bytes(left_address, size as usize), size);

        let right_address_reg = Registers::from(self.get_next_byte());
        let right_address = self.get_register(right_address_reg) as Address;
        let right_value = bytes_to_int(self.memory.get_bytes(right_address, size as usize), size);

        self.set_arithmetical_flags(
            left_value - right_value,
            0
        );
    }


    fn handle_compare_addr_literal_const(&mut self) {
        let size = self.get_next_byte();

        let left_address = self.get_next_address();
        let left_value = bytes_to_int(self.memory.get_bytes(left_address, size as usize), size);

        let right_address = self.get_next_bytes(size as Size);
        let right_value = bytes_to_int(right_address, size);
        
        self.set_arithmetical_flags(
            left_value - right_value,
            0
        );
    }


    fn handle_compare_addr_literal_addr_literal(&mut self) {
        let size = self.get_next_byte();

        let left_address = self.get_next_address();
        let left_value = bytes_to_int(self.memory.get_bytes(left_address, size as usize), size);

        let right_address = self.get_next_address();
        let right_value = bytes_to_int(self.memory.get_bytes(right_address, size as usize), size);
        
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
        io::stdout().flush().expect("Failed to flush stdout");
    }


    fn handle_print_char(&mut self) {
        let value = self.get_register(Registers::PRINT);
        print!("{}", value as u8 as char);
        io::stdout().flush().expect("Failed to flush stdout");
    }


    fn strlen(&self, address: Address) -> Size {
        let mut length = 0;
        let mut byte = self.memory.get_byte(address);

        while byte != 0 {
            length += 1;
            byte = self.memory.get_byte(address + length);
        }

        length
    }


    fn handle_print_string(&mut self) {
        let string_address = self.get_register(Registers::PRINT) as Address;
        let length = self.strlen(string_address);
        let bytes = self.memory.get_bytes(string_address, length as usize);

        io::stdout().write(bytes).expect("Failed to write to stdout");
        io::stdout().flush().expect("Failed to flush stdout");
    }


    fn handle_input_int(&mut self) {
        let mut input = String::new();
        match io::stdin().read_line(&mut input) {
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
        match io::stdin().read_line(&mut input) {
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
        let exit_code = self.get_register(Registers::EXIT);
        std::process::exit(exit_code as i32);
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

        Self::handle_jump_to_reg,
        Self::handle_jump_to_addr_in_reg,
        Self::handle_jump_to_const,
        Self::handle_jump_to_addr_literal,
        Self::handle_jump_if_not_zero_reg_to_reg,
        Self::handle_jump_if_not_zero_to_addr_in_reg,
        Self::handle_jump_if_not_zero_to_const,
        Self::handle_jump_if_not_zero_to_addr_literal,
        Self::handle_jump_if_zero_to_reg,
        Self::handle_jump_if_zero_to_addr_in_reg,
        Self::handle_jump_if_zero_to_const,
        Self::handle_jump_if_zero_to_addr_literal,

        Self::handle_call_reg,
        Self::handle_call_addr_in_reg,
        Self::handle_call_const,
        Self::handle_call_addr_literal,
        Self::handle_return,

        Self::handle_compare_reg_reg,
        Self::handle_compare_reg_addr_in_reg,
        Self::handle_compare_reg_const,
        Self::handle_compare_reg_addr_literal,
        Self::handle_compare_addr_in_reg_reg,
        Self::handle_compare_addr_in_reg_addr_in_reg,
        Self::handle_compare_addr_in_reg_const,
        Self::handle_compare_addr_in_reg_addr_literal,
        Self::handle_compare_const_reg,
        Self::handle_compare_const_addr_in_reg,
        Self::handle_compare_const_const,
        Self::handle_compare_const_addr_literal,
        Self::handle_compare_addr_literal_reg,
        Self::handle_compare_addr_literal_addr_in_reg,
        Self::handle_compare_addr_literal_const,
        Self::handle_compare_addr_literal_addr_literal,

        Self::handle_print_signed,
        Self::handle_print_unsigned,
        Self::handle_print_char,
        Self::handle_print_string,

        Self::handle_input_int,
        Self::handle_input_string,

        Self::handle_exit,
    ];


}

