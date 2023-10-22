#![allow(clippy::no_effect)]


use std::io::Write;
use std::io;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};
use rand::Rng;

use assert_exists::assert_exists;

use rust_vm_lib::registers::{self, Registers};
use rust_vm_lib::byte_code::{ByteCodes, BYTE_CODE_COUNT};
use rust_vm_lib::vm::{Address, ADDRESS_SIZE, ErrorCodes};

use crate::host_fs::HostFS;
use crate::memory::{Memory, Byte};
use crate::cli_parser::ExecutionMode;
use crate::error;
use crate::modules::CPUModules;
use crate::register::CPURegisters;
use crate::storage::Storage;
use crate::terminal::Terminal;


/// Return whether the most significant bit of the given value is set
#[inline(always)]
fn is_msb_set(value: u64) -> bool {
    value & (1 << 63) != 0
}


/// Converts a byte array to an integer
fn bytes_to_int(bytes: &[Byte], handled_size: Byte) -> u64 {
    match handled_size {
        1 => bytes[0] as u64,
        2 => u16::from_le_bytes(bytes.try_into().unwrap()) as u64,
        4 => u32::from_le_bytes(bytes.try_into().unwrap()) as u64,
        8 => u64::from_le_bytes(bytes.try_into().unwrap()),
        _ => error::error(format!("Invalid number size: {}", handled_size).as_str()),
    }
}


/// Interprets the given bytes as an address
/// 
/// The byte array must be 8 bytes long
#[inline(always)]
fn bytes_as_address(bytes: &[Byte]) -> Address {
    Address::from_le_bytes(bytes.try_into().unwrap())
}


pub struct Processor {

    registers: CPURegisters,
    pub memory: Memory,
    start_time: SystemTime,
    quiet_exit: bool,
    /// The program counter of the last instruction executed in interactive mode.
    /// 
    /// This is only used in interactive mode
    interactive_last_instruction_pc: Address,
    modules: CPUModules,

}


pub struct StorageOptions {

    pub file_path: PathBuf,
    pub max_size: Option<usize>,

}


impl StorageOptions {

    pub fn new(file_path: PathBuf, max_size: Option<usize>) -> Self {
        Self {
            file_path,
            max_size,
        }
    }

}


impl Processor {

    const STATIC_PROGRAM_ADDRESS: Address = 0;


    pub fn new(max_memory_size: Option<usize>, quiet_exit: bool, storage: Option<StorageOptions>) -> Self {

        let storage = if let Some(storage) = storage {
            Some(Storage::new(storage.file_path, storage.max_size))
        } else {
            None
        };

        Self {
            registers: CPURegisters::new(),
            memory: Memory::new(max_memory_size),
            // Initialize temporarily, will be reinitialized in `execute`
            start_time: SystemTime::now(),
            quiet_exit,
            interactive_last_instruction_pc: 0,
            modules: CPUModules::new(
                storage,
                Terminal::new(),
                HostFS::new()
            ),
        }
    }


    /// Execute the given bytecode
    pub fn execute(&mut self, byte_code: &[Byte], mode: ExecutionMode) {

        // Set the program counter to the start of the program

        if byte_code.len() < ADDRESS_SIZE {
            error::error(format!("Bytecode is too small to contain a start address: minimum required size is {} bytes, got {}", ADDRESS_SIZE, byte_code.len()).as_str());
        }

        let program_start: Address = bytes_as_address(&byte_code[byte_code.len() - ADDRESS_SIZE..]);
        self.registers.set(Registers::PROGRAM_COUNTER, program_start as u64);

        // Set the heap start to after the static program section
        self.memory.init_layout(byte_code.len() as Address);

        // Initialize the stack pointer
        self.registers.set(Registers::STACK_BASE_POINTER, self.memory.get_stack_start() as u64);

        // Load the program into memory
        self.memory.set_bytes(Self::STATIC_PROGRAM_ADDRESS, byte_code);

        self.start_time = SystemTime::now();

        // Execute the program
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


    /// Push the stack pointer forward
    /// Increment the stack top pointer
    /// Decrement the stack base pointer
    #[inline(always)]
    fn push_stack_pointer(&mut self, offset: usize) {
        self.registers.set(Registers::STACK_TOP_POINTER, self.registers.stack_top() as u64 + offset as u64);
        self.registers.set(Registers::STACK_BASE_POINTER, self.registers.stack_base() as u64 - offset as u64);

        self.check_stack_overflow();
    }


    /// Checks if the stack has overflowed into the heap
    /// If the stack has overflowed, set the error register and terminate the program (stack overflow is unrecoverable)
    fn check_stack_overflow(&mut self) {
        if (self.registers.get(Registers::STACK_BASE_POINTER) as usize) < self.memory.get_heap_end() {
            self.registers.set_error(ErrorCodes::StackOverflow);
            self.handle_exit();
        }
    }


    /// Pop the stack pointer backwards
    /// Decrement the stack top pointer
    /// Increment the stack base pointer
    fn pop_stack_pointer(&mut self, offset: usize) {
        self.registers.set(Registers::STACK_TOP_POINTER, self.registers.stack_top() as u64 - offset as u64);
        self.registers.set(Registers::STACK_BASE_POINTER, self.registers.stack_base() as u64 + offset as u64);
    }


    /// Get the next address in the bytecode
    #[inline(always)]
    fn get_next_address(&mut self) -> Address {
        let pc = self.registers.pc();
        self.registers.inc_pc(ADDRESS_SIZE);
        bytes_as_address(self.memory.get_bytes(pc, ADDRESS_SIZE))
    }


    /// Increment the `size`-sized value at the given address
    fn increment_bytes(&mut self, address: Address, size: Byte) {
        let bytes = self.memory.get_bytes_mut(address, size as usize);

        let (result, carry) = match size {

            1 => {
                let value = bytes[0];
                let (res, carry) = {
                    if let Some(res) = value.checked_add(1) {
                        (res, false)
                    } else {
                        (value.wrapping_add(1), true)
                    }
                };
                bytes[0] = res;

                (res as u64, carry)
            },

            2 => {
                let value = u16::from_le_bytes(bytes.try_into().unwrap());
                let (res, carry) = {
                    if let Some(res) = value.checked_add(1) {
                        (res, false)
                    } else {
                        (value.wrapping_add(1), true)
                    }
                };
                bytes.copy_from_slice(&res.to_le_bytes());

                (res as u64, carry)
            },

            4 => {
                let value = u32::from_le_bytes(bytes.try_into().unwrap());
                let (res, carry) = {
                    if let Some(res) = value.checked_add(1) {
                        (res, false)
                    } else {
                        (value.wrapping_add(1), true)
                    }
                };
                bytes.copy_from_slice(&res.to_le_bytes());

                (res as u64, carry)
            },

            8 => {
                let value = u64::from_le_bytes(bytes.try_into().unwrap());
                let (res, carry) = {
                    if let Some(res) = value.checked_add(1) {
                        (res, false)
                    } else {
                        (value.wrapping_add(1), true)
                    }
                };
                bytes.copy_from_slice(&res.to_le_bytes());

                (res, carry)
            },

            _ => error::error(format!("Invalid size for incrementing bytes: {}.", size).as_str()),
        };

        self.set_arithmetical_flags(
            result == 0,
            is_msb_set(result),
            0,
            carry,
            carry ^ is_msb_set(result)
        );
    }
    
    
    /// Decrement the `size`-sized value at the given address
    fn decrement_bytes(&mut self, address: Address, size: Byte) {
        let bytes = self.memory.get_bytes_mut(address, size as usize);

        let (result, carry) = match size {

            1 => {
                let value = bytes[0];
                let (res, carry) = {
                    if let Some(res) = value.checked_sub(1) {
                        (res, false)
                    } else {
                        (value.wrapping_sub(1), true)
                    }
                };
                bytes[0] = res;

                (res as u64, carry)
            },

            2 => {
                let value = u16::from_le_bytes(bytes.try_into().unwrap());
                let (res, carry) = {
                    if let Some(res) = value.checked_sub(1) {
                        (res, false)
                    } else {
                        (value.wrapping_sub(1), true)
                    }
                };
                bytes.copy_from_slice(&res.to_le_bytes());

                (res as u64, carry)
            },

            4 => {
                let value = u32::from_le_bytes(bytes.try_into().unwrap());
                let (res, carry) = {
                    if let Some(res) = value.checked_sub(1) {
                        (res, false)
                    } else {
                        (value.wrapping_sub(1), true)
                    }
                };
                bytes.copy_from_slice(&res.to_le_bytes());

                (res as u64, carry)
            },

            8 => {
                let value = u64::from_le_bytes(bytes.try_into().unwrap());
                let (res, carry) = {
                    if let Some(res) = value.checked_sub(1) {
                        (res, false)
                    } else {
                        (value.wrapping_sub(1), true)
                    }
                };
                bytes.copy_from_slice(&res.to_le_bytes());

                (res, carry)
            },

            _ => error::error(format!("Invalid size for decrementing bytes: {}.", size).as_str()),
        };

        self.set_arithmetical_flags(
            result == 0,
            is_msb_set(result),
            0,
            carry,
            carry ^ is_msb_set(result)
        );
    }


    /// Mobe a number of bytes from the given address into the given register
    fn move_bytes_into_register(&mut self, src_address: Address, dest_reg: Registers, handled_size: Byte) {
        let bytes = self.memory.get_bytes(src_address, handled_size as usize);
        self.registers.set(
            dest_reg,
            bytes_to_int(bytes, handled_size)
        );
    }


    /// Jump (set the pc) to the given address
    #[inline(always)]
    fn jump_to(&mut self, address: Address) {
        self.registers.set(Registers::PROGRAM_COUNTER, address as u64);
    }


    /// Copy the bytes onto the stack
    fn push_stack_bytes(&mut self, bytes: &[Byte]) {

        // Push the stack pointer first so that it points to where the bytes will be inserted
        self.push_stack_pointer(bytes.len());

        // Copy the bytes onto the stack
        self.memory.set_bytes(
            self.registers.stack_base(),
            bytes,
        );
    }


    /// Copy the bytes at the given address onto the stack
    fn push_stack_from_address(&mut self, src_address: Address, size: usize) {

        // Push the stack pointer first so that it points to where the bytes will be inserted
        self.push_stack_pointer(size);

        // Copy the bytes onto the stack
        self.memory.memcpy(src_address, self.registers.stack_base(), size);
    }


    /// Push an 8-byte value onto the stack
    fn push_stack(&mut self, value: u64) {
        self.push_stack_bytes(&value.to_le_bytes());
    }


    /// Pop `size` bytes from the stack
    fn pop_stack_bytes(&mut self, size: usize) -> &[Byte] {

        self.pop_stack_pointer(size);

        // Subtract the size from the stack base pointer to get the address of the previous top of the stack
        self.memory.get_bytes(
            self.registers.stack_base() - size,
            size,
        )
    }


    /// Move a number of bytes from the given register into the given address
    fn move_from_register_into_address(&mut self, src_reg: Registers, dest_address: Address, handled_size: Byte) {
        let value = self.registers.get(src_reg);

        match handled_size {
            1 => self.memory.set_bytes(dest_address, &(value as u8).to_le_bytes()),
            2 => self.memory.set_bytes(dest_address, &(value as u16).to_le_bytes()),
            4 => self.memory.set_bytes(dest_address, &(value as u32).to_le_bytes()),
            8 => self.memory.set_bytes(dest_address, &value.to_le_bytes()),
            _ => error::error(format!("Invalid size for move instruction {}.", handled_size).as_str()),
        }
    }
        
    
    /// Get the next `size` bytes in the bytecode
    fn get_next_bytes(&mut self, size: usize) -> &[Byte] {
        let pc = self.registers.pc();
        self.registers.inc_pc(size);
        self.memory.get_bytes(pc, size)
    }


    /// Get the next byte in the bytecode
    fn get_next_byte(&mut self) -> Byte {
        let pc = self.registers.pc();
        self.registers.inc_pc(1);
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
        println!("Start address is: {:#X}", self.registers.pc());
        println!();

        loop {

            let previous_args = self.memory.get_bytes(
                self.interactive_last_instruction_pc,
                self.registers.pc().saturating_sub(self.interactive_last_instruction_pc)
            );
            println!("Previous args: {:?}", previous_args);

            let opcode = ByteCodes::from(self.get_next_byte());

            self.interactive_last_instruction_pc = self.registers.pc();

            println!();

            println!("PC: {}, opcode: {}", self.registers.pc(), opcode);
            println!("Registers: {}", self.display_registers());

            const MAX_STACK_VIEW_RANGE: usize = 32;
            let top_bound = self.registers.stack_top().saturating_sub(MAX_STACK_VIEW_RANGE);
            let base_bound = self.registers.stack_top();

            println!(
                "Stack: {:#X} {:?} {:#X}",
                top_bound, &self.memory.get_raw()[top_bound .. base_bound], self.registers.stack_top()
            );

            io::stdin().read_line(&mut String::new()).unwrap();

            self.handle_instruction(opcode);

        }
    }


    fn display_registers(&self) -> String {
        self.registers.iter().enumerate().fold(String::new(), |mut output, (i, reg)| {
            output.push_str(format!("{}: {}, ", registers::REGISTER_NAMES[i], reg).as_str());
            output
        })
    }


    fn run_verbose(&mut self) {
        loop {
            let opcode = ByteCodes::from(self.get_next_byte());
            println!("PC: {}, opcode: {}", self.registers.pc(), opcode);
            self.handle_instruction(opcode);
        }
    }

    
    #[inline(always)]
    fn handle_instruction(&mut self, opcode: ByteCodes) {
        Self::INSTRUCTION_HANDLER_TABLE[opcode as usize](self);
    }


    /// Set the arithmetical flags
    fn set_arithmetical_flags(&mut self, zf: bool, sf: bool, rf: u64, cf: bool, of: bool) {
        self.registers.set(Registers::ZERO_FLAG, zf as u64);
        self.registers.set(Registers::SIGN_FLAG, sf as u64);
        self.registers.set(Registers::REMAINDER_FLAG, rf);
        self.registers.set(Registers::CARRY_FLAG, cf as u64);
        self.registers.set(Registers::OVERFLOW_FLAG, of as u64);
    }


    // Instruction handlers


    fn handle_integer_add(&mut self) {
        assert_exists!(ByteCodes::INTEGER_ADD);

        let r1 = self.registers.get(Registers::R1);
        let r2 = self.registers.get(Registers::R2);
        
        let (result, carry) = match r1.checked_add(r2) {
            Some(result) => (result, false),
            None => (r1.wrapping_add(r2), true)
        };

        self.registers.set(Registers::R1, result);

        self.set_arithmetical_flags(
            result == 0,
            is_msb_set(result),
            0,
            carry,
            carry ^ is_msb_set(result)
        );
    }


    fn handle_integer_sub(&mut self) {
        assert_exists!(ByteCodes::INTEGER_SUB);
        
        let r1 = self.registers.get(Registers::R1);
        let r2 = self.registers.get(Registers::R2);

        let (result, carry) = match r1.checked_sub(r2) {
            Some(result) => (result, false),
            None => (r1.wrapping_sub(r2), true)
        };

        self.registers.set(Registers::R1, result);

        self.set_arithmetical_flags(
            result == 0,
            is_msb_set(result),
            0,
            carry,
            carry ^ is_msb_set(result)
        );
    }


    fn handle_integer_mul(&mut self) {
        assert_exists!(ByteCodes::INTEGER_MUL);
        
        let r1 = self.registers.get(Registers::R1);
        let r2 = self.registers.get(Registers::R2);

        let (result, carry) = match r1.checked_mul(r2) {
            Some(result) => (result, false),
            None => (r1.wrapping_mul(r2), true)
        };

        self.registers.set(Registers::R1, result);

        self.set_arithmetical_flags(
            result == 0,
            is_msb_set(result),
            0,
            carry,
            carry ^ is_msb_set(result)
        );
    }


    fn handle_integer_div(&mut self) {
        assert_exists!(ByteCodes::INTEGER_DIV);

        let r1 = self.registers.get(Registers::R1);
        let r2 = self.registers.get(Registers::R2);

        if r2 == 0 {
            self.registers.set_error(ErrorCodes::ZeroDivision);
            return;
        }

        // Assume no carry or overflow
        let result = r1 / r2;
        
        self.registers.set(Registers::R1, result);

        self.set_arithmetical_flags(
            result == 0,
            is_msb_set(result),
            r1 % r2,
            false,
            false
        );
    }


    fn handle_integer_mod(&mut self) {
        assert_exists!(ByteCodes::INTEGER_MOD);
        
        let r1 = self.registers.get(Registers::R1);
        let r2 = self.registers.get(Registers::R2);

        if r2 == 0 {
            self.registers.set_error(ErrorCodes::ZeroDivision);
            return;
        }

        let result = r1 % r2;

        self.registers.set(Registers::R1, result);

        self.set_arithmetical_flags(
            result == 0,
            is_msb_set(result),
            0,
            false,
            false
        );
    }


    fn handle_float_add(&mut self) {
        assert_exists!(ByteCodes::FLOAT_ADD);

        let r1 = self.registers.get(Registers::R1) as f64;
        let r2 = self.registers.get(Registers::R2) as f64;
        
        let result = r1 + r2;

        self.registers.set(Registers::R1, result as u64);

        self.set_arithmetical_flags(
            result == 0.0,
            result.is_sign_negative(),
            // Use unused integer flags as float flags
            result.is_nan() as u64,
            result.is_infinite() && result.is_sign_positive(),
            result.is_infinite() && result.is_sign_negative()
        );
    }


    fn handle_float_sub(&mut self) {
        assert_exists!(ByteCodes::FLOAT_SUB);

        let r1 = self.registers.get(Registers::R1) as f64;
        let r2 = self.registers.get(Registers::R2) as f64;
        
        let result = r1 - r2;

        self.registers.set(Registers::R1, result as u64);

        self.set_arithmetical_flags(
            result == 0.0,
            result.is_sign_negative(),
            // Use unused integer flags as float flags
            result.is_nan() as u64,
            result.is_infinite() && result.is_sign_positive(),
            result.is_infinite() && result.is_sign_negative()
        );
    }


    fn handle_float_mul(&mut self) {
        assert_exists!(ByteCodes::FLOAT_MUL);

        let r1 = self.registers.get(Registers::R1) as f64;
        let r2 = self.registers.get(Registers::R2) as f64;
        
        let result = r1 * r2;

        self.registers.set(Registers::R1, result as u64);

        self.set_arithmetical_flags(
            result == 0.0,
            result.is_sign_negative(),
            // Use unused integer flags as float flags
            result.is_nan() as u64,
            result.is_infinite() && result.is_sign_positive(),
            result.is_infinite() && result.is_sign_negative()
        );
    }


    fn handle_float_div(&mut self) {
        assert_exists!(ByteCodes::FLOAT_DIV);

        let r1 = self.registers.get(Registers::R1) as f64;
        let r2 = self.registers.get(Registers::R2) as f64;
        
        let result = r1 / r2;

        self.registers.set(Registers::R1, result as u64);

        self.set_arithmetical_flags(
            result == 0.0,
            result.is_sign_negative(),
            // Use unused integer flags as float flags
            result.is_nan() as u64,
            result.is_infinite() && result.is_sign_positive(),
            result.is_infinite() && result.is_sign_negative()
        );
    }


    fn handle_float_mod(&mut self) {
        assert_exists!(ByteCodes::FLOAT_MOD);

        let r1 = self.registers.get(Registers::R1) as f64;
        let r2 = self.registers.get(Registers::R2) as f64;
        
        let result = r1 % r2;

        self.registers.set(Registers::R1, result as u64);

        self.set_arithmetical_flags(
            result == 0.0,
            result.is_sign_negative(),
            // Use unused integer flags as float flags
            result.is_nan() as u64,
            result.is_infinite() && result.is_sign_positive(),
            result.is_infinite() && result.is_sign_negative()
        );
    }


    fn handle_inc_reg(&mut self) {
        assert_exists!(ByteCodes::INC_REG);

        let dest_reg = Registers::from(self.get_next_byte());
        let value = self.registers.get(dest_reg);

        let (result, carry) = match value.checked_add(1) {
            Some(result) => (result, false),
            None => (value.saturating_add(1), true)
        };

        self.registers.set(dest_reg, result);

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
        let address: Address = self.registers.get(address_reg) as Address;
        
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
        let value = self.registers.get(dest_reg);

        let (result, carry) = match value.checked_sub(1) {
            Some(result) => (result, false),
            None => (value.wrapping_sub(1), true)
        };

        self.registers.set(dest_reg, result);

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
        let address: Address = self.registers.get(address_reg) as Address;
        
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
        self.registers.set(dest_reg, self.registers.get(source_reg));
    }


    fn handle_move_into_reg_from_addr_in_reg(&mut self) {
        assert_exists!(ByteCodes::MOVE_INTO_REG_FROM_ADDR_IN_REG);

        let size = self.get_next_byte();
        let dest_reg = Registers::from(self.get_next_byte());
        let address_reg = Registers::from(self.get_next_byte());
        let src_address = self.registers.get(address_reg) as Address;

        self.move_bytes_into_register(src_address, dest_reg, size);
    }


    fn handle_move_into_reg_from_const(&mut self) {
        assert_exists!(ByteCodes::MOVE_INTO_REG_FROM_CONST);

        let size = self.get_next_byte();
        let dest_reg = Registers::from(self.get_next_byte());

        // Hack the borrow checker
        let src_address = self.registers.pc();

        self.move_bytes_into_register(src_address, dest_reg, size);
        self.registers.inc_pc(size as usize);
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
        let dest_address = self.registers.get(dest_address_reg) as Address;

        self.move_from_register_into_address(src_reg, dest_address, size);
    }


    fn handle_move_into_addr_in_reg_from_addr_in_reg(&mut self) {
        assert_exists!(ByteCodes::MOVE_INTO_ADDR_IN_REG_FROM_ADDR_IN_REG);

        let size = self.get_next_byte();
        let dest_address_reg = Registers::from(self.get_next_byte());
        let src_address_reg = Registers::from(self.get_next_byte());
        let dest_address = self.registers.get(dest_address_reg) as Address;
        let src_address = self.registers.get(src_address_reg) as Address;
        
        self.memory.memcpy(src_address, dest_address, size as usize);
    }


    fn handle_move_into_addr_in_reg_from_const(&mut self) {
        assert_exists!(ByteCodes::MOVE_INTO_ADDR_IN_REG_FROM_CONST);

        let size = self.get_next_byte();
        let dest_address_reg = Registers::from(self.get_next_byte());
        let dest_address = self.registers.get(dest_address_reg) as Address;
        let src_address = self.registers.pc();
        
        self.memory.memcpy(src_address, dest_address, size as usize);
        self.registers.inc_pc(size as usize);
    }


    fn handle_move_into_addr_in_reg_from_addr_literal(&mut self) {
        assert_exists!(ByteCodes::MOVE_INTO_ADDR_IN_REG_FROM_ADDR_LITERAL);

        let size = self.get_next_byte();
        let dest_address_reg = Registers::from(self.get_next_byte());
        let dest_address = self.registers.get(dest_address_reg) as Address;
        let src_address = self.get_next_address();

        self.memory.memcpy(src_address, dest_address, size as usize);
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
        let src_address = self.registers.get(src_address_reg) as Address;

        self.memory.memcpy(src_address, dest_address, size as usize);
    }


    fn handle_move_into_addr_literal_from_const(&mut self) {
        assert_exists!(ByteCodes::MOVE_INTO_ADDR_LITERAL_FROM_CONST);

        let size = self.get_next_byte();
        let dest_address = self.get_next_address();
        let src_address = self.registers.pc();

        self.memory.memcpy(src_address, dest_address, size as usize);
        self.registers.inc_pc(size as usize);
    }


    fn handle_move_into_addr_literal_from_addr_literal(&mut self) {
        assert_exists!(ByteCodes::MOVE_INTO_ADDR_LITERAL_FROM_ADDR_LITERAL);

        let size = self.get_next_byte();
        let dest_address = self.get_next_address();
        let src_address = self.get_next_address();

        self.memory.memcpy(src_address, dest_address, size as usize);
    }


    fn handle_push_from_reg(&mut self) {
        assert_exists!(ByteCodes::PUSH_FROM_REG);

        let src_reg = Registers::from(self.get_next_byte());

        self.push_stack(self.registers.get(src_reg));
    }


    fn handle_push_from_addr_in_reg(&mut self) {
        assert_exists!(ByteCodes::PUSH_FROM_ADDR_IN_REG);

        let size = self.get_next_byte();

        let src_address_reg = Registers::from(self.get_next_byte());
        let src_address = self.registers.get(src_address_reg) as Address;

        self.push_stack_from_address(src_address, size as usize);
    }


    fn handle_push_from_const(&mut self) {
        assert_exists!(ByteCodes::PUSH_FROM_CONST);

        let size = self.get_next_byte();

        // Hack to get around the borrow checker
        self.push_stack_from_address(self.registers.pc(), size as usize);
        self.registers.inc_pc(size as usize);
    }


    fn handle_push_from_addr_literal(&mut self) {
        assert_exists!(ByteCodes::PUSH_FROM_ADDR_LITERAL);

        let size = self.get_next_byte();

        let src_address = self.get_next_address();

        self.push_stack_from_address(src_address, size as usize);
    }


    fn handle_push_stack_pointer_reg(&mut self) {
        assert_exists!(ByteCodes::PUSH_STACK_POINTER_REG);

        let reg = Registers::from(self.get_next_byte());
        let offset = self.registers.get(reg);

        self.push_stack_pointer(offset as usize);
    }


    fn handle_push_stack_pointer_addr_in_reg(&mut self) {
        assert_exists!(ByteCodes::PUSH_STACK_POINTER_ADDR_IN_REG);

        let size = self.get_next_byte();

        let address_reg = Registers::from(self.get_next_byte());
        let address = self.registers.get(address_reg) as Address;

        let offset = bytes_to_int(self.memory.get_bytes(address, size as usize), size);

        self.push_stack_pointer(offset as usize);
    }


    fn handle_push_stack_pointer_const(&mut self) {
        assert_exists!(ByteCodes::PUSH_STACK_POINTER_CONST);

        let size = self.get_next_byte();

        let offset = bytes_to_int(self.get_next_bytes(size as usize), size);

        self.push_stack_pointer(offset as usize);
    }


    fn handle_push_stack_pointer_addr_literal(&mut self) {
        assert_exists!(ByteCodes::PUSH_STACK_POINTER_ADDR_LITERAL);

        let size = self.get_next_byte();

        let address = self.get_next_address();

        let offset = bytes_to_int(self.memory.get_bytes(address, size as usize), size);

        self.push_stack_pointer(offset as usize);
    }


    fn handle_pop_into_reg(&mut self) {
        assert_exists!(ByteCodes::POP_INTO_REG);

        let size = self.get_next_byte();

        let dest_reg = Registers::from(self.get_next_byte());
        let bytes = self.pop_stack_bytes(size as usize);
        let value = bytes_to_int(bytes, size);

        self.registers.set(dest_reg, value);
    }


    fn handle_pop_into_addr_in_reg(&mut self) {
        assert_exists!(ByteCodes::POP_INTO_ADDR_IN_REG);

        let size = self.get_next_byte();

        let dest_address_reg = Registers::from(self.get_next_byte());
        let dest_address = self.registers.get(dest_address_reg) as Address;

        self.memory.memcpy(self.registers.stack_base(), dest_address, size as usize);

        self.pop_stack_pointer(size as usize);
    }


    fn handle_pop_into_addr_literal(&mut self) {
        assert_exists!(ByteCodes::POP_INTO_ADDR_LITERAL);

        let size = self.get_next_byte();

        let dest_address = self.get_next_address();

        self.memory.memcpy(self.registers.stack_base(), dest_address, size as usize);

        self.pop_stack_pointer(size as usize);
    }


    fn handle_pop_stack_pointer_reg(&mut self) {
        assert_exists!(ByteCodes::POP_STACK_POINTER_REG);

        let reg = Registers::from(self.get_next_byte());
        let offset = self.registers.get(reg);

        self.pop_stack_pointer(offset as usize);
    }


    fn handle_pop_stack_pointer_addr_in_reg(&mut self) {
        assert_exists!(ByteCodes::POP_STACK_POINTER_ADDR_IN_REG);

        let size = self.get_next_byte();

        let address_reg = Registers::from(self.get_next_byte());
        let address = self.registers.get(address_reg) as Address;

        let offset = bytes_to_int(self.memory.get_bytes(address, size as usize), size);

        self.pop_stack_pointer(offset as usize);
    }


    fn handle_pop_stack_pointer_const(&mut self) {
        assert_exists!(ByteCodes::POP_STACK_POINTER_CONST);

        let size = self.get_next_byte();

        let offset = bytes_to_int(self.get_next_bytes(size as usize), size);

        self.pop_stack_pointer(offset as usize);
    }


    fn handle_pop_stack_pointer_addr_literal(&mut self) {
        assert_exists!(ByteCodes::POP_STACK_POINTER_ADDR_LITERAL);

        let size = self.get_next_byte();

        let address = self.get_next_address();

        let offset = bytes_to_int(self.memory.get_bytes(address, size as usize), size);

        self.pop_stack_pointer(offset as usize);
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

        if self.registers.get(Registers::ZERO_FLAG) == 0 {
            self.jump_to(jump_address);
        }
    }


    fn handle_jump_zero(&mut self) {
        assert_exists!(ByteCodes::JUMP_ZERO);

        let jump_address = self.get_next_address();

        if self.registers.get(Registers::ZERO_FLAG) == 1 {
            self.jump_to(jump_address);
        }
    }


    fn handle_jump_greater(&mut self) {
        assert_exists!(ByteCodes::JUMP_GREATER);

        let jump_address = self.get_next_address();

        if self.registers.get(Registers::SIGN_FLAG) == self.registers.get(Registers::OVERFLOW_FLAG)
            && self.registers.get(Registers::ZERO_FLAG) == 0 {
            self.jump_to(jump_address);
        }
    }


    fn handle_jump_greater_or_equal(&mut self) {
        assert_exists!(ByteCodes::JUMP_GREATER_OR_EQUAL);

        let jump_address = self.get_next_address();

        if self.registers.get(Registers::SIGN_FLAG) == self.registers.get(Registers::OVERFLOW_FLAG) {
            self.jump_to(jump_address);
        }
    }


    fn handle_jump_less(&mut self) {
        assert_exists!(ByteCodes::JUMP_LESS);

        let jump_address = self.get_next_address();

        if self.registers.get(Registers::SIGN_FLAG) != self.registers.get(Registers::OVERFLOW_FLAG) {
            self.jump_to(jump_address);
        }
    }


    fn handle_jump_less_or_equal(&mut self) {
        assert_exists!(ByteCodes::JUMP_LESS_OR_EQUAL);

        let jump_address = self.get_next_address();

        if self.registers.get(Registers::SIGN_FLAG) != self.registers.get(Registers::OVERFLOW_FLAG)
            || self.registers.get(Registers::ZERO_FLAG) == 1 {
            self.jump_to(jump_address);
        }
    }


    fn handle_jump_overflow(&mut self) {
        assert_exists!(ByteCodes::JUMP_OVERFLOW);

        let jump_address = self.get_next_address();

        if self.registers.get(Registers::OVERFLOW_FLAG) == 1 {
            self.jump_to(jump_address);
        }
    }


    fn handle_jump_not_overflow(&mut self) {
        assert_exists!(ByteCodes::JUMP_NOT_OVERFLOW);

        let jump_address = self.get_next_address();

        if self.registers.get(Registers::OVERFLOW_FLAG) == 0 {
            self.jump_to(jump_address);
        }
    }


    fn handle_jump_carry(&mut self) {
        assert_exists!(ByteCodes::JUMP_CARRY);

        let jump_address = self.get_next_address();

        if self.registers.get(Registers::CARRY_FLAG) == 1 {
            self.jump_to(jump_address);
        }
    }


    fn handle_jump_not_carry(&mut self) {
        assert_exists!(ByteCodes::JUMP_NOT_CARRY);

        let jump_address = self.get_next_address();

        if self.registers.get(Registers::CARRY_FLAG) == 0 {
            self.jump_to(jump_address);
        }
    }


    fn handle_jump_sign(&mut self) {
        assert_exists!(ByteCodes::JUMP_SIGN);

        let jump_address = self.get_next_address();

        if self.registers.get(Registers::SIGN_FLAG) == 1 {
            self.jump_to(jump_address);
        }
    }


    fn handle_jump_not_sign(&mut self) {
        assert_exists!(ByteCodes::JUMP_NOT_SIGN);

        let jump_address = self.get_next_address();

        if self.registers.get(Registers::SIGN_FLAG) == 0 {
            self.jump_to(jump_address);
        }
    }


    fn handle_call(&mut self) {
        assert_exists!(ByteCodes::CALL);

        let jump_address = self.get_next_address();

        // Push the return address onto the stack (return address is the current pc)
        self.push_stack(self.registers.pc() as u64);

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
    
        let result = self.registers.get(left_reg) as i64 - self.registers.get(right_reg) as i64;

        self.set_arithmetical_flags(
            result == 0,
            is_msb_set(result as u64),
            0,
            false,
            false
        );
    }


    fn handle_compare_reg_addr_in_reg(&mut self) {
        assert_exists!(ByteCodes::COMPARE_REG_ADDR_IN_REG);

        let size = self.get_next_byte();

        let left_reg = Registers::from(self.get_next_byte());
        let left_value = self.registers.get(left_reg);

        let right_address_reg = Registers::from(self.get_next_byte());
        let right_address = self.registers.get(right_address_reg) as Address;
        let right_value = bytes_to_int(self.memory.get_bytes(right_address, size as usize), size);

        let result = left_value as i64 - right_value as i64;

        self.set_arithmetical_flags(
            result == 0,
            is_msb_set(result as u64),
            0,
            false,
            false
        );
    }


    fn handle_compare_reg_const(&mut self) {
        assert_exists!(ByteCodes::COMPARE_REG_CONST);

        let size = self.get_next_byte();

        let left_reg = Registers::from(self.get_next_byte());
        let left_value = self.registers.get(left_reg);

        let right_value = bytes_to_int(self.get_next_bytes(size as usize), size);

        let result = left_value as i64 - right_value as i64;

        self.set_arithmetical_flags(
            result == 0,
            is_msb_set(result as u64),
            0,
            false,
            false
        );
    }


    fn handle_compare_reg_addr_literal(&mut self) {
        assert_exists!(ByteCodes::COMPARE_REG_ADDR_LITERAL);

        let size = self.get_next_byte();

        let left_reg = Registers::from(self.get_next_byte());
        let left_value = self.registers.get(left_reg);

        let right_address = self.get_next_address();
        let right_value = bytes_to_int(self.memory.get_bytes(right_address, size as usize), size);

        let result = left_value as i64 - right_value as i64;

        self.set_arithmetical_flags(
            result == 0,
            is_msb_set(result as u64),
            0,
            false,
            false
        );
    }


    fn handle_compare_addr_in_reg_reg(&mut self) {
        assert_exists!(ByteCodes::COMPARE_ADDR_IN_REG_REG);

        let size = self.get_next_byte();

        let left_address_reg = Registers::from(self.get_next_byte());
        let left_address = self.registers.get(left_address_reg) as Address;
        let left_value = bytes_to_int(self.memory.get_bytes(left_address, size as usize), size);
        
        let right_reg = Registers::from(self.get_next_byte());
        let right_value = self.registers.get(right_reg);

        let result = left_value as i64 - right_value as i64;

        self.set_arithmetical_flags(
            result == 0,
            is_msb_set(result as u64),
            0,
            false,
            false
        );
    }


    fn handle_compare_addr_in_reg_addr_in_reg(&mut self) {
        assert_exists!(ByteCodes::COMPARE_ADDR_IN_REG_ADDR_IN_REG);

        let size = self.get_next_byte();

        let left_address_reg = Registers::from(self.get_next_byte());
        let left_address = self.registers.get(left_address_reg) as Address;
        let left_value = bytes_to_int(self.memory.get_bytes(left_address, size as usize), size);
        
        let right_address_reg = Registers::from(self.get_next_byte());
        let right_address = self.registers.get(right_address_reg) as Address;
        let right_value = bytes_to_int(self.memory.get_bytes(right_address, size as usize), size);

        let result = left_value as i64 - right_value as i64;

        self.set_arithmetical_flags(
            result == 0,
            is_msb_set(result as u64),
            0,
            false,
            false
        );
    }


    fn handle_compare_addr_in_reg_const(&mut self) {
        assert_exists!(ByteCodes::COMPARE_ADDR_IN_REG_CONST);

        let size = self.get_next_byte();

        let left_address_reg = Registers::from(self.get_next_byte());
        let left_address = self.registers.get(left_address_reg) as Address;
        let left_value = bytes_to_int(self.memory.get_bytes(left_address, size as usize), size);
       
        let right_value = bytes_to_int(self.get_next_bytes(size as usize), size);

        let result = left_value as i64 - right_value as i64;

        self.set_arithmetical_flags(
            result == 0,
            is_msb_set(result as u64),
            0,
            false,
            false
        );
    }


    fn handle_compare_addr_in_reg_addr_literal(&mut self) {
        assert_exists!(ByteCodes::COMPARE_ADDR_IN_REG_ADDR_LITERAL);

        let size = self.get_next_byte();

        let left_address_reg = Registers::from(self.get_next_byte());
        let left_address = self.registers.get(left_address_reg) as Address;
        let left_value = bytes_to_int(self.memory.get_bytes(left_address, size as usize), size);
       
        let right_address = self.get_next_address();
        let right_value = bytes_to_int(self.memory.get_bytes(right_address, size as usize), size);

        let result = left_value as i64 - right_value as i64;

        self.set_arithmetical_flags(
            result == 0,
            is_msb_set(result as u64),
            0,
            false,
            false
        );
    }


    fn handle_compare_const_reg(&mut self) {
        assert_exists!(ByteCodes::COMPARE_CONST_REG);

        let size = self.get_next_byte();

        let left_address = self.get_next_bytes(size as usize);
        let left_value = bytes_to_int(left_address, size);

        let right_reg = Registers::from(self.get_next_byte());
        let right_value = self.registers.get(right_reg);
        
        let result = left_value as i64 - right_value as i64;

        self.set_arithmetical_flags(
            result == 0,
            is_msb_set(result as u64),
            0,
            false,
            false
        );
    }


    fn handle_compare_const_addr_in_reg(&mut self) {
        assert_exists!(ByteCodes::COMPARE_CONST_ADDR_IN_REG);

        let size = self.get_next_byte();

        let left_address = self.get_next_bytes(size as usize);
        let left_value = bytes_to_int(left_address, size);

        let right_address_reg = Registers::from(self.get_next_byte());
        let right_address = self.registers.get(right_address_reg) as Address;
        let right_value = bytes_to_int(self.memory.get_bytes(right_address, size as usize), size);

        let result = left_value as i64 - right_value as i64;

        self.set_arithmetical_flags(
            result == 0,
            is_msb_set(result as u64),
            0,
            false,
            false
        );
    }


    fn handle_compare_const_const(&mut self) {
        assert_exists!(ByteCodes::COMPARE_CONST_CONST);

        let size = self.get_next_byte();

        let left_address = self.get_next_bytes(size as usize);
        let left_value = bytes_to_int(left_address, size);

        let right_address = self.get_next_bytes(size as usize);
        let right_value = bytes_to_int(right_address, size);
        
        let result = left_value as i64 - right_value as i64;

        self.set_arithmetical_flags(
            result == 0,
            is_msb_set(result as u64),
            0,
            false,
            false
        );
    }


    fn handle_compare_const_addr_literal(&mut self) {
        assert_exists!(ByteCodes::COMPARE_CONST_ADDR_LITERAL);

        let size = self.get_next_byte();

        let left_address = self.get_next_bytes(size as usize);
        let left_value = bytes_to_int(left_address, size);

        let right_address = self.get_next_address();
        let right_value = bytes_to_int(self.memory.get_bytes(right_address, size as usize), size);

        let result = left_value as i64 - right_value as i64;

        self.set_arithmetical_flags(
            result == 0,
            is_msb_set(result as u64), 
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
        let right_value = self.registers.get(right_reg);

        let result = left_value as i64 - right_value as i64;

        self.set_arithmetical_flags(
            result == 0,
            is_msb_set(result as u64), 
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
        let right_address = self.registers.get(right_address_reg) as Address;
        let right_value = bytes_to_int(self.memory.get_bytes(right_address, size as usize), size);

        let result = left_value as i64 - right_value as i64;

        self.set_arithmetical_flags(
            result == 0,
            is_msb_set(result as u64), 
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

        let right_address = self.get_next_bytes(size as usize);
        let right_value = bytes_to_int(right_address, size);
        
        let result = left_value as i64 - right_value as i64;

        self.set_arithmetical_flags(
            result == 0,
            is_msb_set(result as u64), 
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
        
        let result = left_value as i64 - right_value as i64;

        self.set_arithmetical_flags(
            result == 0,
            is_msb_set(result as u64), 
            0,
            false,
            false
        );
    }


    fn handle_and(&mut self) {
        assert_exists!(ByteCodes::AND);

        let result = self.registers.get(Registers::R1) & self.registers.get(Registers::R2);

        self.registers.set(Registers::R1, result);

        self.set_arithmetical_flags(
            result == 0,
            is_msb_set(result),
            0,
            false,
            false
        );
    }


    fn handle_or(&mut self) {
        assert_exists!(ByteCodes::OR);

        let result = self.registers.get(Registers::R1) | self.registers.get(Registers::R2);

        self.registers.set(Registers::R1, result);

        self.set_arithmetical_flags(
            result == 0,
            is_msb_set(result),
            0,
            false,
            false
        );
    }


    fn handle_xor(&mut self) {
        assert_exists!(ByteCodes::XOR);

        let result = self.registers.get(Registers::R1) ^ self.registers.get(Registers::R2);

        self.registers.set(Registers::R1, result);

        self.set_arithmetical_flags(
            result == 0,
            is_msb_set(result),
            0,
            false,
            true
        );
    }


    fn handle_not(&mut self) {
        assert_exists!(ByteCodes::NOT);

        let result = !self.registers.get(Registers::R1);

        self.registers.set(Registers::R1, result);

        self.set_arithmetical_flags(
            result == 0,
            is_msb_set(result),
            0,
            false,
            true
        );
    }


    fn handle_shift_left(&mut self) {
        assert_exists!(ByteCodes::SHIFT_LEFT);

        let value = self.registers.get(Registers::R1);
        let shift_amount = self.registers.get(Registers::R2);

        let result = value.overflowing_shl(shift_amount as u32).0;

        self.registers.set(Registers::R1, result);
    }


    fn handle_shift_right(&mut self) {
        assert_exists!(ByteCodes::SHIFT_RIGHT);

        let value = self.registers.get(Registers::R1);
        let shift_amount = self.registers.get(Registers::R2);

        let result = value.overflowing_shr(shift_amount as u32).0;

        self.registers.set(Registers::R1, result);
    }


    fn handle_interrupt_reg(&mut self) {
        assert_exists!(ByteCodes::INTERRUPT_REG);

        let reg = Registers::from(self.get_next_byte());
        let intr_code = self.registers.get(reg) as u8;

        self.handle_interrupt(intr_code);
    }


    #[inline(always)]
    fn handle_interrupt(&mut self, intr_code: u8) {
        Self::INTERRUPT_HANDLER_TABLE[intr_code as usize](self);
    }


    fn handle_exit(&mut self) {
        assert_exists!(ByteCodes::EXIT);

        let exit_code_n = self.registers.get(Registers::EXIT) as u8;
        let exit_code = ErrorCodes::from(exit_code_n);

        if !self.quiet_exit {
            println!("Program exited with code {} ({})", exit_code_n, exit_code);
        }
        
        std::process::exit(exit_code as i32);
    }


    const INSTRUCTION_HANDLER_TABLE: [ fn(&mut Self); BYTE_CODE_COUNT ] = [
        Self::handle_integer_add,
        Self::handle_integer_sub,
        Self::handle_integer_mul,
        Self::handle_integer_div,
        Self::handle_integer_mod,

        Self::handle_float_add,
        Self::handle_float_sub,
        Self::handle_float_mul,
        Self::handle_float_div,
        Self::handle_float_mod,

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

        Self::handle_push_stack_pointer_reg,
        Self::handle_push_stack_pointer_addr_in_reg,
        Self::handle_push_stack_pointer_const,
        Self::handle_push_stack_pointer_addr_literal,

        Self::handle_pop_into_reg,
        Self::handle_pop_into_addr_in_reg,
        Self::handle_pop_into_addr_literal,

        Self::handle_pop_stack_pointer_reg,
        Self::handle_pop_stack_pointer_addr_in_reg,
        Self::handle_pop_stack_pointer_const,
        Self::handle_pop_stack_pointer_addr_literal,

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

        Self::handle_and,
        Self::handle_or,
        Self::handle_xor,
        Self::handle_not,
        Self::handle_shift_left,
        Self::handle_shift_right,

        Self::handle_interrupt_reg,
        Self::handle_interrupt_addr_in_reg,
        Self::handle_interrupt_const,
        Self::handle_interrupt_addr_literal,

        Self::handle_exit,
    ];


    fn handle_interrupt_addr_in_reg(&mut self) {
        assert_exists!(ByteCodes::INTERRUPT_ADDR_IN_REG);

        let address_reg = Registers::from(self.get_next_byte());
        let address = self.registers.get(address_reg) as Address;
        let intr_code = bytes_to_int(self.memory.get_bytes(address, 1), 1) as u8;

        self.handle_interrupt(intr_code);
    }


    fn handle_interrupt_const(&mut self) {
        assert_exists!(ByteCodes::INTERRUPT_CONST);

        let intr_code = self.get_next_byte();

        self.handle_interrupt(intr_code);
    }


    fn handle_interrupt_addr_literal(&mut self) {
        assert_exists!(ByteCodes::INTERRUPT_ADDR_LITERAL);

        let address = self.get_next_address();
        let intr_code = bytes_to_int(self.memory.get_bytes(address, 1), 1) as u8;

        self.handle_interrupt(intr_code);
    }


    fn handle_print_signed(&mut self) {
        
        let value = self.registers.get(Registers::PRINT);
        print!("{}", value as i64);
        io::stdout().flush().expect("Failed to flush stdout");
    }


    fn handle_print_unsigned(&mut self) {

        let value = self.registers.get(Registers::PRINT);
        print!("{}", value);
        io::stdout().flush().expect("Failed to flush stdout");
    }


    fn handle_print_char(&mut self) {

        let value = self.registers.get(Registers::PRINT);
        io::stdout().write_all(&[value as u8]).expect("Failed to write to stdout");
        io::stdout().flush().expect("Failed to flush stdout");
    }


    fn handle_print_string(&mut self) {

        let string_address = self.registers.get(Registers::PRINT) as Address;
        let length = self.strlen(string_address);
        let bytes = self.memory.get_bytes(string_address, length);

        io::stdout().write_all(bytes).expect("Failed to write to stdout");
        io::stdout().flush().expect("Failed to flush stdout");
    }


    fn handle_print_bytes(&mut self) {

        let bytes_address = self.registers.get(Registers::PRINT) as Address;
        let length = self.registers.get(Registers::R1) as usize;
        let bytes = self.memory.get_bytes(bytes_address, length);

        io::stdout().write_all(bytes).expect("Failed to write to stdout");
        io::stdout().flush().expect("Failed to flush stdout");
    }


    fn handle_input_signed_int(&mut self) {

        let mut input = String::new();

        match io::stdin().read_line(&mut input) {
            Ok(bytes_read) => {

                // Check for EOF errors
                if bytes_read == 0 {
                    self.registers.set_error(ErrorCodes::EndOfFile);
                    return;
                }

                match input.trim().parse::<i64>() {
                    Ok(value) => {
                        self.registers.set(Registers::INPUT, value as u64);
                        self.registers.set_error(ErrorCodes::NoError);
                    },
                    Err(_) => {
                        self.registers.set_error(ErrorCodes::InvalidInput);
                    }
                }
            },
            Err(_) => {
                self.registers.set_error(ErrorCodes::GenericError);
            },
        }
    }


    fn handle_input_unsigned_int(&mut self) {
        let mut input = String::new();
        match io::stdin().read_line(&mut input) {
            Ok(bytes_read) => {

                // Check for EOF errors
                if bytes_read == 0 {
                    self.registers.set_error(ErrorCodes::EndOfFile);
                    return;
                }

                match input.trim().parse::<u64>() {
                    Ok(value) => {
                        self.registers.set(Registers::INPUT, value);
                        self.registers.set_error(ErrorCodes::NoError);
                    },
                    Err(_) => {
                        self.registers.set_error(ErrorCodes::InvalidInput);
                    }
                }
            },
            Err(_) => {
                self.registers.set_error(ErrorCodes::GenericError);
            },
        }
    }


    fn handle_input_string(&mut self) {

        let mut input = String::new();

        match io::stdin().read_line(&mut input) {
            Ok(bytes_read) => {

                // Check for EOF errors
                if bytes_read == 0 {
                    self.registers.set_error(ErrorCodes::EndOfFile);
                    return;
                }

                // Allocate the user input on the heap
                let address = match self.memory.allocate(input.len()) {
                    Ok(address) => address,
                    Err(e) => {
                        self.registers.set_error(e);
                        return;
                    }
                };
                self.memory.set_bytes(address, input.as_bytes());

                self.registers.set(Registers::INPUT, address as u64);
                self.registers.set(Registers::R1, input.len() as u64);

                self.registers.set_error(ErrorCodes::NoError); 
            },

            Err(_) => {
                self.registers.set_error(ErrorCodes::GenericError);
            },
        }
    }


    fn handle_malloc(&mut self) {

        let size = self.registers.get(Registers::R1) as usize;

        match self.memory.allocate(size) {

            Ok(address) => {
                self.registers.set(Registers::R1, address as u64);
                self.registers.set_error(ErrorCodes::NoError);
            },

            Err(e) => {
                self.registers.set_error(e);
            }

        }
    }


    fn handle_free(&mut self) {

        let address = self.registers.get(Registers::R1) as Address;

        match self.memory.free(address) {

            Ok(_) => {
                self.registers.set_error(ErrorCodes::NoError);
            },

            Err(e) => {
                self.registers.set_error(e);
            }

        }
    }


    fn handle_random(&mut self) {

        let mut rng = rand::thread_rng();
        let random_number = rng.gen_range(u64::MIN..u64::MAX);

        self.registers.set(Registers::R1, random_number);
    }


    fn handle_host_time_nanos(&mut self) {

        // Casting to u64 will be ok until around 2500
        let time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos() as u64;

        self.registers.set(Registers::R1, time);
    }


    fn handle_elapsed_time_nanos(&mut self) {

        // Casting to u64 will be ok until around 2500
        let time = SystemTime::now().duration_since(self.start_time).unwrap().as_nanos() as u64;

        self.registers.set(Registers::R1, time);
    }


    fn handle_disk_read(&mut self) {

        let disk_address = self.registers.get(Registers::R1) as Address;
        let buffer_address = self.registers.get(Registers::R2) as Address;
        let size = self.registers.get(Registers::R3) as usize;

        let err = if let Some(storage) = &self.modules.storage {
            match storage.read(disk_address, size) {
                Ok(bytes) => {
                    self.memory.set_bytes(buffer_address, &bytes);
                    ErrorCodes::NoError
                },
                Err(e) => e
            }
        } else {
            ErrorCodes::ModuleUnavailable
        };

        self.registers.set_error(err);
    }


    fn handle_disk_write(&mut self) {

        let disk_address = self.registers.get(Registers::R1) as Address;
        let data_address = self.registers.get(Registers::R2) as Address;
        let size = self.registers.get(Registers::R3) as usize;

        let buffer = self.memory.get_bytes(data_address, size);

        self.registers.set_error(
            if let Some(storage) = &self.modules.storage {
                match storage.write(disk_address, buffer) {
                    Ok(_) => ErrorCodes::NoError,
                    Err(e) => e
                }
            } else {
                ErrorCodes::ModuleUnavailable
            }
        );
    }


    fn handle_terminal(&mut self) {

        let term_code = self.registers.get(Registers::PRINT); 

        let err = self.modules.terminal.handle_code(term_code as usize, &mut self.registers, &mut self.memory);
        self.registers.set_error(err);
    }


    fn handle_set_timer_nanos(&mut self) {

        let time = self.registers.get(Registers::R1);
        let duration = std::time::Duration::from_nanos(time);

        std::thread::sleep(duration);
    }


    fn handle_flush_stdout(&mut self) {
        io::stdout().flush().expect("Failed to flush stdout");
    }


    fn handle_host_fs(&mut self) {
        
        let fs_code = self.registers.get(Registers::PRINT);

        let err = self.modules.host_fs.handle_code(fs_code as usize, &mut self.registers, &mut self.memory);
        self.registers.set_error(err);
    }


    const INTERRUPT_HANDLER_TABLE: [ fn(&mut Self); 19 ] = [
        Self::handle_print_signed, // 0
        Self::handle_print_unsigned, // 1
        Self::handle_print_char, // 2
        Self::handle_print_string, // 3
        Self::handle_print_bytes, // 4
        Self::handle_input_signed_int, // 5
        Self::handle_input_unsigned_int, // 6
        Self::handle_input_string, // 7
        Self::handle_malloc, // 8
        Self::handle_free, // 9
        Self::handle_random, // 10
        Self::handle_host_time_nanos, // 11
        Self::handle_elapsed_time_nanos, // 12
        Self::handle_disk_read, // 13
        Self::handle_disk_write, // 14
        Self::handle_terminal, // 15
        Self::handle_set_timer_nanos, // 16
        Self::handle_flush_stdout, // 17
        Self::handle_host_fs, // 18
    ];


}

