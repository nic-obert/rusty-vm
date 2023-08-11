use std::io::Write;
use std::io;

use assert_exists::assert_exists;

use rust_vm_lib::registers::{Registers, REGISTER_COUNT, self};
use rust_vm_lib::byte_code::{ByteCodes, BYTE_CODE_COUNT};
use rust_vm_lib::vm::{Address, ADDRESS_SIZE, ErrorCodes};

use crate::memory::{Memory, Size, Byte};
use crate::cli_parser::ExecutionMode;


/// Return whether the most significant bit of the given value is set
#[inline(always)]
fn is_msb_set(value: u64) -> bool {
    value & (1 << 63) != 0
}


/// Converts a byte array to an integer
fn bytes_to_int(bytes: &[Byte], handled_size: Byte) -> u64 {
    match handled_size {
        1 => bytes[0] as u64,
        2 => unsafe {
            *(bytes.as_ptr() as *const u16) as u64
        },
        4 => unsafe {
            *(bytes.as_ptr() as *const u32) as u64
        },
        8 => unsafe {
            *(bytes.as_ptr() as *const u64) as u64
        },
        _ => panic!("Invalid number size: {}", handled_size),
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

    registers: [u64; REGISTER_COUNT],
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
        self.set_register(Registers::PROGRAM_COUNTER, program_start as u64);

        match mode {
            ExecutionMode::Normal => self.run(),
            ExecutionMode::Verbose => self.run_verbose(),
            ExecutionMode::Interactive => self.run_interactive(byte_code.len()),
        }
            
    }


    /// Return the length of a null-terminated string
    fn strlen(&self, address: Address) -> usize {
        let mut length = 0;
        let mut byte = self.memory.get_byte(address);

        while byte != 0 {
            length += 1;
            byte = self.memory.get_byte(address + length);
        }

        length
    }


    /// Move the program counter by the given offset.
    #[inline(always)]
    fn move_pc(&mut self, offset: i64) {
        self.set_register(
            Registers::PROGRAM_COUNTER, 
            (self.get_pc() as i64 + offset) as u64
        );
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
    fn get_register(&self, register: Registers) -> u64 {
        self.registers[register as usize]
    }


    /// Get a mutable reference to the given register
    #[inline(always)]
    fn get_register_mut(&mut self, register: Registers) -> &mut u64 {
        &mut self.registers[register as usize]
    }


    /// Set the value of the given register
    #[inline(always)]
    fn set_register(&mut self, register: Registers, value: u64) {
        self.registers[register as usize] = value;
    }


    /// Set the error register
    #[inline(always)]
    fn set_error(&mut self, error: ErrorCodes) {
        self.registers[Registers::ERROR as usize] = error as u64;
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

        let result = match size {
            1 => unsafe {
                *(bytes.as_mut_ptr() as *mut u8) += 1;
                *(bytes.as_ptr() as *const u8) as u64
            },
            2 => unsafe {
                *(bytes.as_mut_ptr() as *mut u16) += 1;
                *(bytes.as_ptr() as *const u16) as u64
            },
            4 => unsafe {
                *(bytes.as_mut_ptr() as *mut u32) += 1;
                *(bytes.as_ptr() as *const u32) as u64
            },
            8 => unsafe {
                *(bytes.as_mut_ptr() as *mut u64) += 1;
                *(bytes.as_ptr() as *const u64)
            },
            _ => panic!("Invalid size for incrementing bytes: {}", size),
        };

        self.set_arithmetical_flags(
            result == 0,
            is_msb_set(result),
            0,
            false,
            false
        );
    }
    
    
    /// Decrement the `size`-sized value at the given address
    fn decrement_bytes(&mut self, address: Address, size: Byte) {
        let bytes = self.memory.get_bytes_mut(address, size as Size);

        let result = match size {
            1 => unsafe {
                *(bytes.as_mut_ptr() as *mut u8) -= 1;
                *(bytes.as_ptr() as *const u8) as u64
            },
            2 => unsafe {
                *(bytes.as_mut_ptr() as *mut u16) -= 1;
                *(bytes.as_ptr() as *const u16) as u64
            },
            4 => unsafe {
                *(bytes.as_mut_ptr() as *mut u32) -= 1;
                *(bytes.as_ptr() as *const u32) as u64
            },
            8 => unsafe {
                *(bytes.as_mut_ptr() as *mut u64) -= 1;
                *(bytes.as_ptr() as *const u64)
            },
            _ => panic!("Invalid size for incrementing bytes: {}", size),
        };

        self.set_arithmetical_flags(
            result == 0,
            is_msb_set(result),
            0,
            false,
            false
        );
    }


    /// Mobe a number of bytes from the given address into the given register
    fn move_bytes_into_register(&mut self, src_address: Address, dest_reg: Registers, handled_size: Byte) {
        let bytes = self.memory.get_bytes(src_address, handled_size as usize);
        self.set_register(
            dest_reg,
            bytes_to_int(bytes, handled_size)
        );
    }


    /// Jump (set the pc) to the given address
    #[inline(always)]
    fn jump_to(&mut self, address: Address) {
        self.set_register(Registers::PROGRAM_COUNTER, address as u64);
    }


    /// Copy the bytes onto the stack
    fn push_stack_bytes(&mut self, bytes: &[Byte]) {
        // Copy the bytes onto the stack
        self.memory.set_bytes(
            self.get_register(Registers::STACK_POINTER) as Size,
            bytes,
        );
        // Move the stack pointer
        self.registers[Registers::STACK_POINTER as usize] += bytes.len() as u64;
    }


    /// Copy the bytes at the given address onto the stack
    fn push_stack_from_address(&mut self, src_address: Address, size: Size) {
        // Get the tos
        let dest_address = self.get_register(Registers::STACK_POINTER) as Size;
        // Copy the bytes onto the stack
        self.memory.memcpy(dest_address, src_address, size);
        // Move the stack pointer
        *self.get_register_mut(Registers::STACK_POINTER) += size as u64;
    }


    /// Push an 8-byte value onto the stack
    fn push_stack(&mut self, value: u64) {
        self.push_stack_bytes(&value.to_le_bytes());
    }


    /// Pop `size` bytes from the stack
    fn pop_stack_bytes(&mut self, size: Size) -> &[Byte] {
        // Move the stack pointer
        *self.get_register_mut(Registers::STACK_POINTER) -= size as u64;

        // Return the tos
        self.memory.get_bytes(
            self.get_register(Registers::STACK_POINTER) as Size,
            size,
        )
    }


    /// Move a number of bytes from the given register into the given address
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


    /// Set the arithmetical flags
    fn set_arithmetical_flags(&mut self, zf: bool, sf: bool, rf: u64, cf: bool, of: bool) {
        self.set_register(Registers::ZERO_FLAG, zf as u64);
        self.set_register(Registers::SIGN_FLAG, sf as u64);
        self.set_register(Registers::REMAINDER_FLAG, rf);
        self.set_register(Registers::CARRY_FLAG, cf as u64);
        self.set_register(Registers::OVERFLOW_FLAG, of as u64);
    }


    // Instruction handlers


    fn handle_add(&mut self) {
        assert_exists!(ByteCodes::ADD);

        let r1 = self.get_register(Registers::R1);
        let r2 = self.get_register(Registers::R2);
        
        let (result, carry) = match r1.checked_add(r2) {
            Some(result) => (result, false),
            None => (r1.saturating_add(r2), true)
        };

        self.set_register(Registers::R1, result);

        self.set_arithmetical_flags(
            result == 0,
            is_msb_set(result),
            0,
            carry,
            carry ^ is_msb_set(result)
        );
    }


    fn handle_sub(&mut self) {
        assert_exists!(ByteCodes::SUB);
        
        let r1 = self.get_register(Registers::R1);
        let r2 = self.get_register(Registers::R2);

        let (result, carry) = match r1.checked_sub(r2) {
            Some(result) => (result, false),
            None => (r1.saturating_add(r2), true)
        };

        self.set_register(Registers::R1, result);

        self.set_arithmetical_flags(
            result == 0,
            is_msb_set(result),
            0,
            carry,
            carry ^ is_msb_set(result)
        );
    }


    fn handle_mul(&mut self) {
        assert_exists!(ByteCodes::MUL);
        
        let r1 = self.get_register(Registers::R1);
        let r2 = self.get_register(Registers::R2);

        let (result, carry) = match r1.checked_mul(r2) {
            Some(result) => (result, false),
            None => (r1.saturating_add(r2), true)
        };

        self.set_register(Registers::R1, result);

        self.set_arithmetical_flags(
            result == 0,
            is_msb_set(result),
            0,
            carry,
            carry ^ is_msb_set(result)
        );
    }


    fn handle_div(&mut self) {
        assert_exists!(ByteCodes::DIV);

        let r1 = self.get_register(Registers::R1);
        let r2 = self.get_register(Registers::R2);

        if r2 == 0 {
            self.set_error(ErrorCodes::ZeroDivision);
            return;
        }

        // Assume no carry or overflow
        let result = r1 / r2;
        
        self.set_register(Registers::R1, result);

        self.set_arithmetical_flags(
            result == 0,
            is_msb_set(result),
            r1 % r2,
            false,
            false
        );
    }


    fn handle_mod(&mut self) {
        assert_exists!(ByteCodes::MOD);
        
        let r1 = self.get_register(Registers::R1);
        let r2 = self.get_register(Registers::R2);

        if r2 == 0 {
            self.set_error(ErrorCodes::ZeroDivision);
            return;
        }

        let result = r1 % r2;

        self.set_register(Registers::R1, result);

        self.set_arithmetical_flags(
            result == 0,
            is_msb_set(result),
            0,
            false,
            false
        );
    }


    fn handle_inc_reg(&mut self) {
        assert_exists!(ByteCodes::INC_REG);

        let dest_reg = Registers::from(self.get_next_byte());
        let value = self.get_register(dest_reg);

        let (result, carry) = match value.checked_add(1) {
            Some(result) => (result, false),
            None => (value.saturating_add(1), true)
        };

        self.set_register(dest_reg, result);

        self.set_arithmetical_flags(
            result == 0,
            is_msb_set(result),
            0,
            carry,
            carry ^ is_msb_set(result)
        );
    }


    fn handle_inc_addr_in_reg(&mut self) {
        assert_exists!(ByteCodes::INC_ADDR_IN_REG);

        let size = self.get_next_byte();
        let address_reg = Registers::from(self.get_next_byte());
        let address: Address = self.get_register(address_reg) as Address;
        
        self.increment_bytes(address, size);
    }


    fn handle_inc_addr_literal(&mut self) {
        assert_exists!(ByteCodes::INC_ADDR_LITERAL);

        let size = self.get_next_byte();
        let dest_address = self.get_next_address();
        
        self.increment_bytes(dest_address, size);
    }


    fn handle_dec_reg(&mut self) {
        assert_exists!(ByteCodes::DEC_REG);

        let dest_reg = Registers::from(self.get_next_byte());
        let value = self.get_register(dest_reg);

        let (result, carry) = match value.checked_sub(1) {
            Some(result) => (result, false),
            None => (value.saturating_sub(1), true)
        };

        self.set_register(dest_reg, result);

        self.set_arithmetical_flags(
            result == 0,
            is_msb_set(result),
            0,
            carry,
            carry ^ is_msb_set(result)
        );
    }


    fn handle_dec_addr_in_reg(&mut self) {
        assert_exists!(ByteCodes::DEC_ADDR_IN_REG);

        let size = self.get_next_byte();
        let address_reg = Registers::from(self.get_next_byte());
        let address: Address = self.get_register(address_reg) as Address;
        
        self.decrement_bytes(address, size);
    }


    fn handle_dec_addr_literal(&mut self) {
        assert_exists!(ByteCodes::DEC_ADDR_LITERAL);

        let size = self.get_next_byte();
        let dest_address = self.get_next_address();
        
        self.decrement_bytes(dest_address, size);
    }


    fn handle_no_operation(&mut self) {
        assert_exists!(ByteCodes::NO_OPERATION);

        // Do nothing
    }


    fn handle_move_into_reg_from_reg(&mut self) {
        assert_exists!(ByteCodes::MOVE_INTO_REG_FROM_REG);

        let dest_reg = Registers::from(self.get_next_byte());
        let source_reg = Registers::from(self.get_next_byte());
        self.set_register(dest_reg, self.get_register(source_reg));
    }


    fn handle_move_into_reg_from_addr_in_reg(&mut self) {
        assert_exists!(ByteCodes::MOVE_INTO_REG_FROM_ADDR_IN_REG);

        let size = self.get_next_byte();
        let dest_reg = Registers::from(self.get_next_byte());
        let address_reg = Registers::from(self.get_next_byte());
        let src_address = self.get_register(address_reg) as Address;

        self.move_bytes_into_register(src_address, dest_reg, size);
    }


    fn handle_move_into_reg_from_const(&mut self) {
        assert_exists!(ByteCodes::MOVE_INTO_REG_FROM_CONST);

        let size = self.get_next_byte();
        let dest_reg = Registers::from(self.get_next_byte());
        let src_address = self.get_pc();

        self.move_bytes_into_register(src_address, dest_reg, size);
        self.move_pc(size as i64);
    }


    fn handle_move_into_reg_from_addr_literal(&mut self) {
        assert_exists!(ByteCodes::MOVE_INTO_REG_FROM_ADDR_LITERAL);

        let size = self.get_next_byte();
        let dest_reg = Registers::from(self.get_next_byte());
        let src_address = self.get_next_address();

        self.move_bytes_into_register(src_address, dest_reg, size);
    }


    fn handle_move_into_addr_in_reg_from_reg(&mut self) {
        assert_exists!(ByteCodes::MOVE_INTO_ADDR_IN_REG_FROM_REG);

        let size = self.get_next_byte();
        let dest_address_reg = Registers::from(self.get_next_byte());
        let src_reg = Registers::from(self.get_next_byte());
        let dest_address = self.get_register(dest_address_reg) as Address;

        self.move_from_register_into_address(src_reg, dest_address, size);
    }


    fn handle_move_into_addr_in_reg_from_addr_in_reg(&mut self) {
        assert_exists!(ByteCodes::MOVE_INTO_ADDR_IN_REG_FROM_ADDR_IN_REG);

        let size = self.get_next_byte();
        let dest_address_reg = Registers::from(self.get_next_byte());
        let src_address_reg = Registers::from(self.get_next_byte());
        let dest_address = self.get_register(dest_address_reg) as Address;
        let src_address = self.get_register(src_address_reg) as Address;
        
        self.memory.memcpy(src_address, dest_address, size as Size);
    }


    fn handle_move_into_addr_in_reg_from_const(&mut self) {
        assert_exists!(ByteCodes::MOVE_INTO_ADDR_IN_REG_FROM_CONST);

        let size = self.get_next_byte();
        let dest_address_reg = Registers::from(self.get_next_byte());
        let dest_address = self.get_register(dest_address_reg) as Address;
        let src_address = self.get_pc();
        
        self.memory.memcpy(src_address, dest_address, size as Size);
        self.move_pc(size as i64);
    }


    fn handle_move_into_addr_in_reg_from_addr_literal(&mut self) {
        assert_exists!(ByteCodes::MOVE_INTO_ADDR_IN_REG_FROM_ADDR_LITERAL);

        let size = self.get_next_byte();
        let dest_address_reg = Registers::from(self.get_next_byte());
        let dest_address = self.get_register(dest_address_reg) as Address;
        let src_address = self.get_next_address();

        self.memory.memcpy(src_address, dest_address, size as Size);
    }


    fn handle_move_into_addr_literal_from_reg(&mut self) {
        assert_exists!(ByteCodes::MOVE_INTO_ADDR_LITERAL_FROM_REG);

        let size = self.get_next_byte();
        let dest_address = self.get_next_address();
        let src_reg = Registers::from(self.get_next_byte());

        self.move_from_register_into_address(src_reg, dest_address, size);
    }


    fn handle_move_into_addr_literal_from_addr_in_reg(&mut self) {
        assert_exists!(ByteCodes::MOVE_INTO_ADDR_LITERAL_FROM_ADDR_IN_REG);

        let size = self.get_next_byte();
        let dest_address = self.get_next_address();
        let src_address_reg = Registers::from(self.get_next_byte());
        let src_address = self.get_register(src_address_reg) as Address;

        self.memory.memcpy(src_address, dest_address, size as Size);
    }


    fn handle_move_into_addr_literal_from_const(&mut self) {
        assert_exists!(ByteCodes::MOVE_INTO_ADDR_LITERAL_FROM_CONST);

        let size = self.get_next_byte();
        let dest_address = self.get_next_address();
        let src_address = self.get_pc();

        self.memory.memcpy(src_address, dest_address, size as Size);
        self.move_pc(size as i64);
    }


    fn handle_move_into_addr_literal_from_addr_literal(&mut self) {
        assert_exists!(ByteCodes::MOVE_INTO_ADDR_LITERAL_FROM_ADDR_LITERAL);

        let size = self.get_next_byte();
        let dest_address = self.get_next_address();
        let src_address = self.get_next_address();

        self.memory.memcpy(src_address, dest_address, size as Size);
    }


    fn handle_push_from_reg(&mut self) {
        assert_exists!(ByteCodes::PUSH_FROM_REG);

        let src_reg = Registers::from(self.get_next_byte());

        self.push_stack(self.get_register(src_reg));
    }


    fn handle_push_from_addr_in_reg(&mut self) {
        assert_exists!(ByteCodes::PUSH_FROM_ADDR_IN_REG);

        let size = self.get_next_byte();

        let src_address_reg = Registers::from(self.get_next_byte());
        let src_address = self.get_register(src_address_reg) as Address;

        self.push_stack_from_address(src_address, size as Size);
    }


    fn handle_push_from_const(&mut self) {
        assert_exists!(ByteCodes::PUSH_FROM_CONST);

        let size = self.get_next_byte();

        // Hack to get around the borrow checker
        self.push_stack_from_address(self.get_pc(), size as Size);
        self.move_pc(size as i64);
    }


    fn handle_push_from_addr_literal(&mut self) {
        assert_exists!(ByteCodes::PUSH_FROM_ADDR_LITERAL);

        let size = self.get_next_byte();

        let src_address = self.get_next_address();

        self.push_stack_from_address(src_address, size as Size);
    }


    fn handle_pop_into_reg(&mut self) {
        assert_exists!(ByteCodes::POP_INTO_REG);

        let size = self.get_next_byte();

        let dest_reg = Registers::from(self.get_next_byte());
        let bytes = self.pop_stack_bytes(size as Size);
        let value = bytes_to_int(bytes, size);

        self.set_register(dest_reg, value);
    }


    fn handle_pop_into_addr_in_reg(&mut self) {
        assert_exists!(ByteCodes::POP_INTO_ADDR_IN_REG);

        let size = self.get_next_byte();

        let dest_address_reg = Registers::from(self.get_next_byte());
        let dest_address = self.get_register(dest_address_reg) as Address;

        let src_address = self.get_pc();

        self.memory.memcpy(src_address, dest_address, size as Size);
        self.move_pc(size as i64);
    }


    fn handle_pop_into_addr_literal(&mut self) {
        assert_exists!(ByteCodes::POP_INTO_ADDR_LITERAL);

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


    fn handle_jump(&mut self) {
        assert_exists!(ByteCodes::JUMP);

        let jump_address = self.get_next_address();

        self.jump_to(jump_address);
    }


    fn handle_jump_not_zero(&mut self) {
        assert_exists!(ByteCodes::JUMP_NOT_ZERO);

        let jump_address = self.get_next_address();

        if self.get_register(Registers::ZERO_FLAG) != 0 {
            self.jump_to(jump_address);
        }
    }


    fn handle_jump_zero(&mut self) {
        assert_exists!(ByteCodes::JUMP_ZERO);

        let jump_address = self.get_next_address();

        if self.get_register(Registers::ZERO_FLAG) == 0 {
            self.jump_to(jump_address);
        }
    }


    fn handle_jump_greater(&mut self) {
        assert_exists!(ByteCodes::JUMP_GREATER);

        let jump_address = self.get_next_address();

        if self.get_register(Registers::SIGN_FLAG) == self.get_register(Registers::OVERFLOW_FLAG)
            && self.get_register(Registers::ZERO_FLAG) == 0 {
            self.jump_to(jump_address);
        }
    }


    fn handle_jump_greater_or_equal(&mut self) {
        assert_exists!(ByteCodes::JUMP_GREATER_OR_EQUAL);

        let jump_address = self.get_next_address();

        if self.get_register(Registers::SIGN_FLAG) == self.get_register(Registers::OVERFLOW_FLAG) {
            self.jump_to(jump_address);
        }
    }


    fn handle_jump_less(&mut self) {
        assert_exists!(ByteCodes::JUMP_LESS);

        let jump_address = self.get_next_address();

        if self.get_register(Registers::SIGN_FLAG) != self.get_register(Registers::OVERFLOW_FLAG) {
            self.jump_to(jump_address);
        }
    }


    fn handle_jump_less_or_equal(&mut self) {
        assert_exists!(ByteCodes::JUMP_LESS_OR_EQUAL);

        let jump_address = self.get_next_address();

        if self.get_register(Registers::SIGN_FLAG) != self.get_register(Registers::OVERFLOW_FLAG)
            || self.get_register(Registers::ZERO_FLAG) == 1 {
            self.jump_to(jump_address);
        }
    }


    fn handle_jump_overflow(&mut self) {
        assert_exists!(ByteCodes::JUMP_OVERFLOW);

        let jump_address = self.get_next_address();

        if self.get_register(Registers::OVERFLOW_FLAG) == 1 {
            self.jump_to(jump_address);
        }
    }


    fn handle_jump_not_overflow(&mut self) {
        assert_exists!(ByteCodes::JUMP_NOT_OVERFLOW);

        let jump_address = self.get_next_address();

        if self.get_register(Registers::OVERFLOW_FLAG) == 0 {
            self.jump_to(jump_address);
        }
    }


    fn handle_jump_carry(&mut self) {
        assert_exists!(ByteCodes::JUMP_CARRY);

        let jump_address = self.get_next_address();

        if self.get_register(Registers::CARRY_FLAG) == 1 {
            self.jump_to(jump_address);
        }
    }


    fn handle_jump_not_carry(&mut self) {
        assert_exists!(ByteCodes::JUMP_NOT_CARRY);

        let jump_address = self.get_next_address();

        if self.get_register(Registers::CARRY_FLAG) == 0 {
            self.jump_to(jump_address);
        }
    }


    fn handle_jump_sign(&mut self) {
        assert_exists!(ByteCodes::JUMP_SIGN);

        let jump_address = self.get_next_address();

        if self.get_register(Registers::SIGN_FLAG) == 1 {
            self.jump_to(jump_address);
        }
    }


    fn handle_jump_not_sign(&mut self) {
        assert_exists!(ByteCodes::JUMP_NOT_SIGN);

        let jump_address = self.get_next_address();

        if self.get_register(Registers::SIGN_FLAG) == 0 {
            self.jump_to(jump_address);
        }
    }


    fn handle_call(&mut self) {
        assert_exists!(ByteCodes::CALL);

        let jump_address = self.get_next_address();

        // Push the return address onto the stack (return address is the current pc)
        self.push_stack(self.get_pc() as u64);

        // Jump to the subroutine
        self.jump_to(jump_address);
    }


    fn handle_return(&mut self) {
        assert_exists!(ByteCodes::RETURN);

        // Get the return address from the stack
        let return_address = bytes_as_address(
            self.pop_stack_bytes(ADDRESS_SIZE)
        );

        // Jump to the return address
        self.jump_to(return_address);
    }


    fn handle_compare_reg_reg(&mut self) {
        assert_exists!(ByteCodes::COMPARE_REG_REG);

        let left_reg = Registers::from(self.get_next_byte());
        let right_reg = Registers::from(self.get_next_byte());
    
        let result = self.get_register(left_reg) - self.get_register(right_reg);

        self.set_arithmetical_flags(
            result == 0,
            is_msb_set(result),
            0,
            false,
            false
        );
    }


    fn handle_compare_reg_addr_in_reg(&mut self) {
        assert_exists!(ByteCodes::COMPARE_REG_ADDR_IN_REG);

        let size = self.get_next_byte();

        let left_reg = Registers::from(self.get_next_byte());
        let left_value = self.get_register(left_reg);

        let right_address_reg = Registers::from(self.get_next_byte());
        let right_address = self.get_register(right_address_reg) as Address;
        let right_value = bytes_to_int(self.memory.get_bytes(right_address, size as usize), size);

        let result = left_value - right_value;

        self.set_arithmetical_flags(
            result == 0,
            is_msb_set(result),
            0,
            false,
            false
        );
    }


    fn handle_compare_reg_const(&mut self) {
        assert_exists!(ByteCodes::COMPARE_REG_CONST);

        let size = self.get_next_byte();

        let left_reg = Registers::from(self.get_next_byte());
        let left_value = self.get_register(left_reg);

        let right_value = bytes_to_int(self.get_next_bytes(size as usize), size);

        let result = left_value - right_value;

        self.set_arithmetical_flags(
            result == 0,
            is_msb_set(result),
            0,
            false,
            false
        );
    }


    fn handle_compare_reg_addr_literal(&mut self) {
        assert_exists!(ByteCodes::COMPARE_REG_ADDR_LITERAL);

        let size = self.get_next_byte();

        let left_reg = Registers::from(self.get_next_byte());
        let left_value = self.get_register(left_reg);

        let right_address = self.get_next_address();
        let right_value = bytes_to_int(self.memory.get_bytes(right_address, size as usize), size);

        let result = left_value - right_value;

        self.set_arithmetical_flags(
            result == 0,
            is_msb_set(result),
            0,
            false,
            false
        );
    }


    fn handle_compare_addr_in_reg_reg(&mut self) {
        assert_exists!(ByteCodes::COMPARE_ADDR_IN_REG_REG);

        let size = self.get_next_byte();

        let left_address_reg = Registers::from(self.get_next_byte());
        let left_address = self.get_register(left_address_reg) as Address;
        let left_value = bytes_to_int(self.memory.get_bytes(left_address, ADDRESS_SIZE), size);
        
        let right_reg = Registers::from(self.get_next_byte());
        let right_value = self.get_register(right_reg);

        let result = left_value - right_value;

        self.set_arithmetical_flags(
            result == 0,
            is_msb_set(result),
            0,
            false,
            false
        );
    }


    fn handle_compare_addr_in_reg_addr_in_reg(&mut self) {
        assert_exists!(ByteCodes::COMPARE_ADDR_IN_REG_ADDR_IN_REG);

        let size = self.get_next_byte();

        let left_address_reg = Registers::from(self.get_next_byte());
        let left_address = self.get_register(left_address_reg) as Address;
        let left_value = bytes_to_int(self.memory.get_bytes(left_address, ADDRESS_SIZE), size);
        
        let right_address_reg = Registers::from(self.get_next_byte());
        let right_address = self.get_register(right_address_reg) as Address;
        let right_value = bytes_to_int(self.memory.get_bytes(right_address, size as usize), size);

        let result = left_value - right_value;

        self.set_arithmetical_flags(
            result == 0,
            is_msb_set(result),
            0,
            false,
            false
        );
    }


    fn handle_compare_addr_in_reg_const(&mut self) {
        assert_exists!(ByteCodes::COMPARE_ADDR_IN_REG_CONST);

        let size = self.get_next_byte();

        let left_address_reg = Registers::from(self.get_next_byte());
        let left_address = self.get_register(left_address_reg) as Address;
        let left_value = bytes_to_int(self.memory.get_bytes(left_address, ADDRESS_SIZE), size);
       
        let right_value = bytes_to_int(self.get_next_bytes(size as usize), size);

        let result = left_value - right_value;

        self.set_arithmetical_flags(
            result == 0,
            is_msb_set(result),
            0,
            false,
            false
        );
    }


    fn handle_compare_addr_in_reg_addr_literal(&mut self) {
        assert_exists!(ByteCodes::COMPARE_ADDR_IN_REG_ADDR_LITERAL);

        let size = self.get_next_byte();

        let left_address_reg = Registers::from(self.get_next_byte());
        let left_address = self.get_register(left_address_reg) as Address;
        let left_value = bytes_to_int(self.memory.get_bytes(left_address, ADDRESS_SIZE), size);
       
        let right_address = self.get_next_address();
        let right_value = bytes_to_int(self.memory.get_bytes(right_address, size as usize), size);

        let result = left_value - right_value;

        self.set_arithmetical_flags(
            result == 0,
            is_msb_set(result),
            0,
            false,
            false
        );
    }


    fn handle_compare_const_reg(&mut self) {
        assert_exists!(ByteCodes::COMPARE_CONST_REG);

        let size = self.get_next_byte();

        let left_address = self.get_next_bytes(size as Size);
        let left_value = bytes_to_int(left_address, size);

        let right_reg = Registers::from(self.get_next_byte());
        let right_value = self.get_register(right_reg);
        
        let result = left_value - right_value;

        self.set_arithmetical_flags(
            result == 0,
            is_msb_set(result),
            0,
            false,
            false
        );
    }


    fn handle_compare_const_addr_in_reg(&mut self) {
        assert_exists!(ByteCodes::COMPARE_CONST_ADDR_IN_REG);

        let size = self.get_next_byte();

        let left_address = self.get_next_bytes(size as Size);
        let left_value = bytes_to_int(left_address, size);

        let right_address_reg = Registers::from(self.get_next_byte());
        let right_address = self.get_register(right_address_reg) as Address;
        let right_value = bytes_to_int(self.memory.get_bytes(right_address, size as usize), size);

        let result = left_value - right_value;

        self.set_arithmetical_flags(
            result == 0,
            is_msb_set(result),
            0,
            false,
            false
        );
    }


    fn handle_compare_const_const(&mut self) {
        assert_exists!(ByteCodes::COMPARE_CONST_CONST);

        let size = self.get_next_byte();

        let left_address = self.get_next_bytes(size as Size);
        let left_value = bytes_to_int(left_address, size);

        let right_address = self.get_next_bytes(size as Size);
        let right_value = bytes_to_int(right_address, size);
        
        let result = left_value - right_value;

        self.set_arithmetical_flags(
            result == 0,
            is_msb_set(result),
            0,
            false,
            false
        );
    }


    fn handle_compare_const_addr_literal(&mut self) {
        assert_exists!(ByteCodes::COMPARE_CONST_ADDR_LITERAL);

        let size = self.get_next_byte();

        let left_address = self.get_next_bytes(size as Size);
        let left_value = bytes_to_int(left_address, size);

        let right_address = self.get_next_address();
        let right_value = bytes_to_int(self.memory.get_bytes(right_address, size as usize), size);

        let result = left_value - right_value;

        self.set_arithmetical_flags(
            result == 0,
            is_msb_set(result), 
            0,
            false,
            false
        );
    }


    fn handle_compare_addr_literal_reg(&mut self) {
        assert_exists!(ByteCodes::COMPARE_ADDR_LITERAL_REG);

        let size = self.get_next_byte();

        let left_address = self.get_next_address();
        let left_value = bytes_to_int(self.memory.get_bytes(left_address, size as usize), size);

        let right_reg = Registers::from(self.get_next_byte());
        let right_value = self.get_register(right_reg);

        let result = left_value - right_value;

        self.set_arithmetical_flags(
            result == 0,
            is_msb_set(result), 
            0,
            false,
            false
        );
    }


    fn handle_compare_addr_literal_addr_in_reg(&mut self) {
        assert_exists!(ByteCodes::COMPARE_ADDR_LITERAL_ADDR_IN_REG);

        let size = self.get_next_byte();

        let left_address = self.get_next_address();
        let left_value = bytes_to_int(self.memory.get_bytes(left_address, size as usize), size);

        let right_address_reg = Registers::from(self.get_next_byte());
        let right_address = self.get_register(right_address_reg) as Address;
        let right_value = bytes_to_int(self.memory.get_bytes(right_address, size as usize), size);

        let result = left_value - right_value;

        self.set_arithmetical_flags(
            result == 0,
            is_msb_set(result), 
            0,
            false,
            false
        );
    }


    fn handle_compare_addr_literal_const(&mut self) {
        assert_exists!(ByteCodes::COMPARE_ADDR_LITERAL_CONST);

        let size = self.get_next_byte();

        let left_address = self.get_next_address();
        let left_value = bytes_to_int(self.memory.get_bytes(left_address, size as usize), size);

        let right_address = self.get_next_bytes(size as Size);
        let right_value = bytes_to_int(right_address, size);
        
        let result = left_value - right_value;

        self.set_arithmetical_flags(
            result == 0,
            is_msb_set(result), 
            0,
            false,
            false
        );
    }


    fn handle_compare_addr_literal_addr_literal(&mut self) {
        assert_exists!(ByteCodes::COMPARE_ADDR_LITERAL_ADDR_LITERAL);

        let size = self.get_next_byte();

        let left_address = self.get_next_address();
        let left_value = bytes_to_int(self.memory.get_bytes(left_address, size as usize), size);

        let right_address = self.get_next_address();
        let right_value = bytes_to_int(self.memory.get_bytes(right_address, size as usize), size);
        
        let result = left_value - right_value;

        self.set_arithmetical_flags(
            result == 0,
            is_msb_set(result), 
            0,
            false,
            false
        );
    }


    fn handle_print_signed(&mut self) {
        assert_exists!(ByteCodes::PRINT_SIGNED);

        let value = self.get_register(Registers::PRINT);
        print!("{}", value);
        std::io::stdout().flush().expect("Failed to flush stdout");
    }


    fn handle_print_unsigned(&mut self) {
        assert_exists!(ByteCodes::PRINT_UNSIGNED);

        let value = self.get_register(Registers::PRINT) as u64;
        print!("{}", value);
        io::stdout().flush().expect("Failed to flush stdout");
    }


    fn handle_print_char(&mut self) {
        assert_exists!(ByteCodes::PRINT_CHAR);

        let value = self.get_register(Registers::PRINT);
        print!("{}", value as u8 as char);
        io::stdout().flush().expect("Failed to flush stdout");
    }


    fn handle_print_string(&mut self) {
        assert_exists!(ByteCodes::PRINT_STRING);

        let string_address = self.get_register(Registers::PRINT) as Address;
        let length = self.strlen(string_address);
        let bytes = self.memory.get_bytes(string_address, length as usize);

        io::stdout().write(bytes).expect("Failed to write to stdout");
        io::stdout().flush().expect("Failed to flush stdout");
    }


    fn handle_print_bytes(&mut self) {
        assert_exists!(ByteCodes::PRINT_BYTES);

        let bytes_address = self.get_register(Registers::PRINT) as Address;
        let length = self.get_register(Registers::R1) as usize;
        let bytes = self.memory.get_bytes(bytes_address, length);

        io::stdout().write(bytes).expect("Failed to write to stdout");
    }


    fn handle_input_int(&mut self) {
        assert_exists!(ByteCodes::INPUT_INT);

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
                        self.set_register(Registers::INPUT, value as u64);
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
        assert_exists!(ByteCodes::INPUT_STRING);

        let mut input = String::new();
        match io::stdin().read_line(&mut input) {
            Ok(bytes_read) => {

                // Check for EOF errors
                if bytes_read == 0 {
                    self.set_error(ErrorCodes::EndOfFile);
                    return;
                }

                self.set_register(Registers::INPUT, input.len() as u64);
                self.push_stack_bytes(input.as_bytes());
                self.set_error(ErrorCodes::NoError); 
            },
            Err(_) => {
                self.set_error(ErrorCodes::GenericError);
            },
        }
    }


    fn handle_exit(&mut self) {
        assert_exists!(ByteCodes::EXIT);

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

        Self::handle_jump,
        Self::handle_jump_not_zero,
        Self::handle_jump_zero,
        Self::handle_jump_greater,
        Self::handle_jump_less,
        Self::handle_jump_greater_or_equal,
        Self::handle_jump_less_or_equal,
        Self::handle_jump_carry,
        Self::handle_jump_not_carry,
        Self::handle_jump_overflow,
        Self::handle_jump_not_overflow,
        Self::handle_jump_sign,
        Self::handle_jump_not_sign,

        Self::handle_call,
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
        Self::handle_print_bytes,
        
        Self::handle_input_int,
        Self::handle_input_string,

        Self::handle_exit,
    ];


}

