#![allow(clippy::no_effect)]


use std::cmp::min;
use std::io::{Read, Write};
use std::io;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};
use rand::Rng;

use rusty_vm_lib::registers::Registers;
use rusty_vm_lib::byte_code::ByteCodes;
use rusty_vm_lib::vm::{Address, ADDRESS_SIZE, ErrorCodes};
use rusty_vm_lib::interrupts::Interrupts;

use crate::host_fs::HostFS;
use crate::memory::{Memory, Byte};
use crate::cli_parser::ExecutionMode;
use crate::error;
use crate::modules::CPUModules;
use crate::register::CPURegisters;
use crate::storage::Storage;
use crate::terminal::Terminal;


/// Return whether the most significant bit of the given value is set
#[inline]
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


// TODO: make this function faster (and unsafe)
/// Interprets the given bytes as an address
/// 
/// The byte array must be 8 bytes long
#[inline]
fn bytes_as_address(bytes: &[Byte]) -> Address {
    Address::from_le_bytes(bytes.try_into().unwrap())
}


pub struct Processor {

    registers: CPURegisters,
    pub memory: Memory,
    start_time: SystemTime,
    quiet_exit: bool,
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


    pub fn new(max_memory_size: usize, quiet_exit: bool, storage: Option<StorageOptions>) -> Self {

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

        // Initialize the stack pointer to the end of the memory. The stack grows downwards
        self.registers.set(Registers::STACK_TOP_POINTER, self.memory.get_stack_base() as u64);

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
    /// Decrement the stack top pointer
    #[inline]
    fn push_stack_pointer(&mut self, offset: usize) {
        self.registers.set(Registers::STACK_TOP_POINTER, self.registers.stack_top() as u64 - offset as u64);

        self.check_stack_overflow();
    }


    /// If the stack has overflowed, set the error register and terminate the program (stack overflow is unrecoverable)
    fn check_stack_overflow(&mut self) {
        if (self.registers.get(Registers::STACK_TOP_POINTER) as usize) > self.memory.get_stack_base() {
            self.registers.set_error(ErrorCodes::StackOverflow);
            self.handle_instruction(ByteCodes::EXIT);
        }
    }


    /// Pop the stack pointer backwards
    /// Increment the stack top pointer
    fn pop_stack_pointer(&mut self, offset: usize) {
        self.registers.set(Registers::STACK_TOP_POINTER, self.registers.stack_top() as u64 + offset as u64);

        self.check_stack_overflow();
    }


    /// Get the next address in the bytecode
    #[inline]
    fn get_next_address(&mut self) -> Address {
        let pc = self.registers.pc();
        self.registers.inc_pc(ADDRESS_SIZE);
        self.memory.read::<Address>(pc)
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
            self.registers.stack_top(),
            bytes,
        );
    }


    /// Copy the bytes at the given address onto the stack
    fn push_stack_from_address(&mut self, src_address: Address, size: usize) {

        // Push the stack pointer first so that it points to where the bytes will be inserted
        self.push_stack_pointer(size);

        // Copy the bytes onto the stack
        self.memory.memcpy(src_address, self.registers.stack_top(), size);
    }


    /// Push an 8-byte value onto the stack
    fn push_stack(&mut self, value: u64) {
        self.push_stack_bytes(&value.to_le_bytes());
    }


    /// Pop `size` bytes from the stack
    fn pop_stack_bytes(&mut self, size: usize) -> &[Byte] {

        self.pop_stack_pointer(size);

        // Subtract the size from the stack top pointer to get the address of the previous top of the stack
        self.memory.get_bytes(
            self.registers.stack_top() - size,
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
        println!("Start address is: {}", self.registers.pc());
        println!();

        let mut last_instruction_pc: Address = self.registers.pc();

        loop {

            let previous_args = self.memory.get_bytes(
                last_instruction_pc,
                self.registers.pc().saturating_sub(last_instruction_pc)
            );
            println!("Previous args: {:?}", previous_args);

            let opcode = ByteCodes::from(self.get_next_byte());

            last_instruction_pc = self.registers.pc();

            println!();

            println!("PC: {}, opcode: {}", self.registers.pc(), opcode);
            println!("Registers: {}", self.display_registers());

            const MAX_STACK_VIEW_RANGE: usize = 32;
            // let stack_top = self.memory.get_stack_base() - self.registers.get(Registers::STACK_TOP_POINTER) as Address;
            // let top_bound = stack_top.saturating_sub(MAX_STACK_VIEW_RANGE);
            // let base_bound = stack_top;
            
            let upper_bound = min(self.registers.stack_top() + MAX_STACK_VIEW_RANGE, self.memory.get_stack_base());

            println!(
                "Stack: {} {:?} {}",
                self.registers.stack_top(), &self.memory.get_raw()[self.registers.stack_top() .. upper_bound], upper_bound
            );

            io::stdin().read_line(&mut String::new()).unwrap();

            self.handle_instruction(opcode);

        }
    }


    fn display_registers(&self) -> String {
        self.registers.iter().enumerate().fold(String::new(), |mut output, (i, reg)| {
            output.push_str(format!("{}: {}, ", Registers::from(i as u8), reg).as_str());
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

    
    fn handle_instruction(&mut self, opcode: ByteCodes) {
        match opcode {

            ByteCodes::INTEGER_ADD => {
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
            },

            ByteCodes::INTEGER_SUB => {
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
            },

            ByteCodes::INTEGER_MUL => {
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
            },

            ByteCodes::INTEGER_DIV => {
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
            },

            ByteCodes::INTEGER_MOD => {
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
            },

            ByteCodes::FLOAT_ADD => {
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
            },

            ByteCodes::FLOAT_SUB => {
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
            },

            ByteCodes::FLOAT_MUL => {
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
            },

            ByteCodes::FLOAT_DIV => {
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
            },

            ByteCodes::FLOAT_MOD => {
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
            },

            ByteCodes::INC_REG => {
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
            },

            ByteCodes::INC_ADDR_IN_REG => {
                let size = self.get_next_byte();
                let address_reg = Registers::from(self.get_next_byte());
                let address: Address = self.registers.get(address_reg) as Address;
                
                self.increment_bytes(address, size);
            },

            ByteCodes::INC_ADDR_LITERAL => {
                let size = self.get_next_byte();
                let dest_address = self.get_next_address();
                
                self.increment_bytes(dest_address, size);
            },

            ByteCodes::DEC_REG => {
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
            },

            ByteCodes::DEC_ADDR_IN_REG => {
                let size = self.get_next_byte();
                let address_reg = Registers::from(self.get_next_byte());
                let address: Address = self.registers.get(address_reg) as Address;
                
                self.decrement_bytes(address, size);
            },

            ByteCodes::DEC_ADDR_LITERAL => {
                let size = self.get_next_byte();
                let dest_address = self.get_next_address();
            
                self.decrement_bytes(dest_address, size);
            },

            ByteCodes::NO_OPERATION => {
                // No operation
            },

            ByteCodes::MOVE_INTO_REG_FROM_REG => {
                let dest_reg = Registers::from(self.get_next_byte());
                let source_reg = Registers::from(self.get_next_byte());
                self.registers.set(dest_reg, self.registers.get(source_reg));
            },

            ByteCodes::MOVE_INTO_REG_FROM_REG_SIZED => {
                let size = self.get_next_byte();
                let dest_reg = Registers::from(self.get_next_byte());
                let source_reg = Registers::from(self.get_next_byte());

                let value = self.registers.get_masked(source_reg, size);

                self.registers.set(dest_reg, value);
            },

            ByteCodes::MOVE_INTO_REG_FROM_ADDR_IN_REG => {
                let size = self.get_next_byte();
                let dest_reg = Registers::from(self.get_next_byte());
                let address_reg = Registers::from(self.get_next_byte());
                let src_address = self.registers.get(address_reg) as Address;

                self.move_bytes_into_register(src_address, dest_reg, size);
            },

            ByteCodes::MOVE_INTO_REG_FROM_CONST => {
                let size = self.get_next_byte();
                let dest_reg = Registers::from(self.get_next_byte());
        
                // Hack the borrow checker
                let src_address = self.registers.pc();
        
                self.move_bytes_into_register(src_address, dest_reg, size);
                self.registers.inc_pc(size as usize);
            },
            
            ByteCodes::MOVE_INTO_REG_FROM_ADDR_LITERAL => {
                let size = self.get_next_byte();
                let dest_reg = Registers::from(self.get_next_byte());
                let src_address = self.get_next_address();

                self.move_bytes_into_register(src_address, dest_reg, size);
            },
            
            ByteCodes::MOVE_INTO_ADDR_IN_REG_FROM_REG => {
                let size = self.get_next_byte();
                let dest_address_reg = Registers::from(self.get_next_byte());
                let src_reg = Registers::from(self.get_next_byte());
                let dest_address = self.registers.get(dest_address_reg) as Address;

                self.move_from_register_into_address(src_reg, dest_address, size);
            },
            
            ByteCodes::MOVE_INTO_ADDR_IN_REG_FROM_ADDR_IN_REG => {
                let size = self.get_next_byte();
                let dest_address_reg = Registers::from(self.get_next_byte());
                let src_address_reg = Registers::from(self.get_next_byte());
                let dest_address = self.registers.get(dest_address_reg) as Address;
                let src_address = self.registers.get(src_address_reg) as Address;
                
                self.memory.memcpy(src_address, dest_address, size as usize);
            },
            
            ByteCodes::MOVE_INTO_ADDR_IN_REG_FROM_CONST => {
                let size = self.get_next_byte();
                let dest_address_reg = Registers::from(self.get_next_byte());
                let dest_address = self.registers.get(dest_address_reg) as Address;
                let src_address = self.registers.pc();
                
                self.memory.memcpy(src_address, dest_address, size as usize);
                self.registers.inc_pc(size as usize);
            },
            
            ByteCodes::MOVE_INTO_ADDR_IN_REG_FROM_ADDR_LITERAL => {
                let size = self.get_next_byte();
                let dest_address_reg = Registers::from(self.get_next_byte());
                let dest_address = self.registers.get(dest_address_reg) as Address;
                let src_address = self.get_next_address();
        
                self.memory.memcpy(src_address, dest_address, size as usize);
            },
            
            ByteCodes::MOVE_INTO_ADDR_LITERAL_FROM_REG => {
                let size = self.get_next_byte();
                let dest_address = self.get_next_address();
                let src_reg = Registers::from(self.get_next_byte());
        
                self.move_from_register_into_address(src_reg, dest_address, size);
            },
            
            ByteCodes::MOVE_INTO_ADDR_LITERAL_FROM_ADDR_IN_REG => {
                let size = self.get_next_byte();
                let dest_address = self.get_next_address();
                let src_address_reg = Registers::from(self.get_next_byte());
                let src_address = self.registers.get(src_address_reg) as Address;
        
                self.memory.memcpy(src_address, dest_address, size as usize);  
            },
            
            ByteCodes::MOVE_INTO_ADDR_LITERAL_FROM_CONST => {
                let size = self.get_next_byte();
                let dest_address = self.get_next_address();
                let src_address = self.registers.pc();
        
                self.memory.memcpy(src_address, dest_address, size as usize);
                self.registers.inc_pc(size as usize);
            },
            
            ByteCodes::MOVE_INTO_ADDR_LITERAL_FROM_ADDR_LITERAL => {
                let size = self.get_next_byte();
                let dest_address = self.get_next_address();
                let src_address = self.get_next_address();

                self.memory.memcpy(src_address, dest_address, size as usize);
            },
            
            ByteCodes::PUSH_FROM_REG => {
                let src_reg = Registers::from(self.get_next_byte());

                self.push_stack(self.registers.get(src_reg));
            },

            ByteCodes::PUSH_FROM_REG_SIZED => {
                let size = self.get_next_byte();
                let src_reg = Registers::from(self.get_next_byte());

                self.push_stack(
                    self.registers.get_masked(src_reg, size)
                );
            }
            
            ByteCodes::PUSH_FROM_ADDR_IN_REG => {
                let size = self.get_next_byte();

                let src_address_reg = Registers::from(self.get_next_byte());
                let src_address = self.registers.get(src_address_reg) as Address;
        
                self.push_stack_from_address(src_address, size as usize);  
            },
            
            ByteCodes::PUSH_FROM_CONST => {
                let size = self.get_next_byte();

                // Hack to get around the borrow checker
                self.push_stack_from_address(self.registers.pc(), size as usize);
                self.registers.inc_pc(size as usize);
            },
            
            ByteCodes::PUSH_FROM_ADDR_LITERAL => {
                let size = self.get_next_byte();

                let src_address = self.get_next_address();

                self.push_stack_from_address(src_address, size as usize);
            },
            
            ByteCodes::PUSH_STACK_POINTER_REG => {
                let reg = Registers::from(self.get_next_byte());
                let offset = self.registers.get(reg);
        
                self.push_stack_pointer(offset as usize);
            },

            ByteCodes::PUSH_STACK_POINTER_REG_SIZED => {
                let size = self.get_next_byte();
                let reg = Registers::from(self.get_next_byte());

                self.push_stack_pointer(
                    self.registers.get_masked(reg, size) as usize
                );
            },
            
            ByteCodes::PUSH_STACK_POINTER_ADDR_IN_REG => {
                let size = self.get_next_byte();

                let address_reg = Registers::from(self.get_next_byte());
                let address = self.registers.get(address_reg) as Address;
        
                let offset = bytes_to_int(self.memory.get_bytes(address, size as usize), size);
        
                self.push_stack_pointer(offset as usize);
            },
            
            ByteCodes::PUSH_STACK_POINTER_CONST => {
                let size = self.get_next_byte();

                let offset = bytes_to_int(self.get_next_bytes(size as usize), size);
        
                self.push_stack_pointer(offset as usize);  
            },
            
            ByteCodes::PUSH_STACK_POINTER_ADDR_LITERAL => {
                let size = self.get_next_byte();

                let address = self.get_next_address();
        
                let offset = bytes_to_int(self.memory.get_bytes(address, size as usize), size);
        
                self.push_stack_pointer(offset as usize);
            },
            
            ByteCodes::POP_INTO_REG => {
                let size = self.get_next_byte();

                let dest_reg = Registers::from(self.get_next_byte());
                let bytes = self.pop_stack_bytes(size as usize);
                let value = bytes_to_int(bytes, size);
        
                self.registers.set(dest_reg, value);
            },
            
            ByteCodes::POP_INTO_ADDR_IN_REG => {
                let size = self.get_next_byte();

                let dest_address_reg = Registers::from(self.get_next_byte());
                let dest_address = self.registers.get(dest_address_reg) as Address;
        
                self.memory.memcpy(self.registers.stack_top(), dest_address, size as usize);
        
                self.pop_stack_pointer(size as usize);
            },
            
            ByteCodes::POP_INTO_ADDR_LITERAL => {
                let size = self.get_next_byte();

                let dest_address = self.get_next_address();
        
                self.memory.memcpy(self.registers.stack_top(), dest_address, size as usize);
        
                self.pop_stack_pointer(size as usize);
            },
            
            ByteCodes::POP_STACK_POINTER_REG => {
                let reg = Registers::from(self.get_next_byte());
                let offset = self.registers.get(reg);

                self.pop_stack_pointer(offset as usize);
            },
            
            ByteCodes::POP_STACK_POINTER_REG_SIZED => {
                let size = self.get_next_byte();
                let reg = Registers::from(self.get_next_byte());
                let offset = self.registers.get_masked(reg, size);

                self.pop_stack_pointer(offset as usize);
            }

            ByteCodes::POP_STACK_POINTER_ADDR_IN_REG => {
                let size = self.get_next_byte();

                let address_reg = Registers::from(self.get_next_byte());
                let address = self.registers.get(address_reg) as Address;
        
                let offset = bytes_to_int(self.memory.get_bytes(address, size as usize), size);
        
                self.pop_stack_pointer(offset as usize);
            },
            
            ByteCodes::POP_STACK_POINTER_CONST => {
                let size = self.get_next_byte();

                let offset = bytes_to_int(self.get_next_bytes(size as usize), size);
        
                self.pop_stack_pointer(offset as usize);
            },
            
            ByteCodes::POP_STACK_POINTER_ADDR_LITERAL => {
                let size = self.get_next_byte();

                let address = self.get_next_address();
        
                let offset = bytes_to_int(self.memory.get_bytes(address, size as usize), size);
        
                self.pop_stack_pointer(offset as usize);
            },
            
            ByteCodes::LABEL => {
                unreachable!() // TODO: maybe this should be removed then
            },
            
            ByteCodes::JUMP => {
                let addr = self.get_next_address();
                self.jump_to(addr);
            },
            
            ByteCodes::JUMP_NOT_ZERO => {
                let jump_address = self.get_next_address();

                if self.registers.get(Registers::ZERO_FLAG) == 0 {
                    self.jump_to(jump_address);
                }
            },
            
            ByteCodes::JUMP_ZERO => {
                let jump_address = self.get_next_address();

                if self.registers.get(Registers::ZERO_FLAG) == 1 {
                    self.jump_to(jump_address);
                }
            },
            
            ByteCodes::JUMP_GREATER => {
                let jump_address = self.get_next_address();

                if self.registers.get(Registers::SIGN_FLAG) == self.registers.get(Registers::OVERFLOW_FLAG)
                    && self.registers.get(Registers::ZERO_FLAG) == 0 {
                    self.jump_to(jump_address);
                }
            },
            
            ByteCodes::JUMP_LESS => {
                let jump_address = self.get_next_address();

                if self.registers.get(Registers::SIGN_FLAG) != self.registers.get(Registers::OVERFLOW_FLAG) {
                    self.jump_to(jump_address);
                }
            },
            
            ByteCodes::JUMP_GREATER_OR_EQUAL => {
                let jump_address = self.get_next_address();

                if self.registers.get(Registers::SIGN_FLAG) == self.registers.get(Registers::OVERFLOW_FLAG) {
                    self.jump_to(jump_address);
                }
            },
            
            ByteCodes::JUMP_LESS_OR_EQUAL => {
                let jump_address = self.get_next_address();

                if self.registers.get(Registers::SIGN_FLAG) != self.registers.get(Registers::OVERFLOW_FLAG)
                    || self.registers.get(Registers::ZERO_FLAG) == 1 {
                    self.jump_to(jump_address);
                }
            },
            
            ByteCodes::JUMP_CARRY => {
                let jump_address = self.get_next_address();

                if self.registers.get(Registers::CARRY_FLAG) == 1 {
                    self.jump_to(jump_address);
                }
            },
            
            ByteCodes::JUMP_NOT_CARRY => {
                let jump_address = self.get_next_address();

                if self.registers.get(Registers::CARRY_FLAG) == 0 {
                    self.jump_to(jump_address);
                }
            },
            
            ByteCodes::JUMP_OVERFLOW => {
                let jump_address = self.get_next_address();

                if self.registers.get(Registers::OVERFLOW_FLAG) == 1 {
                    self.jump_to(jump_address);
                }
            },
            
            ByteCodes::JUMP_NOT_OVERFLOW => {
                let jump_address = self.get_next_address();

                if self.registers.get(Registers::OVERFLOW_FLAG) == 0 {
                    self.jump_to(jump_address);
                }
            },
            
            ByteCodes::JUMP_SIGN => {
                let jump_address = self.get_next_address();

                if self.registers.get(Registers::SIGN_FLAG) == 1 {
                    self.jump_to(jump_address);
                }  
            },
            
            ByteCodes::JUMP_NOT_SIGN => {
                let jump_address = self.get_next_address();

                if self.registers.get(Registers::SIGN_FLAG) == 0 {
                    self.jump_to(jump_address);
                }
            },
            
            ByteCodes::CALL => {
                let jump_address = self.get_next_address();

                // Push the return address onto the stack (return address is the current pc)
                self.push_stack(self.registers.pc() as u64);
        
                // Jump to the subroutine
                self.jump_to(jump_address);
            },
            
            ByteCodes::RETURN => {
                // Get the return address from the stack
                let return_address = bytes_as_address(
                    self.pop_stack_bytes(ADDRESS_SIZE)
                );

                // Jump to the return address
                self.jump_to(return_address);
            },
            
            ByteCodes::COMPARE_REG_REG => {
                let left_reg = Registers::from(self.get_next_byte());
                let right_reg = Registers::from(self.get_next_byte());
            
                let result = self.registers.get(left_reg) as i64
                    - self.registers.get(right_reg) as i64;
        
                self.set_arithmetical_flags(
                    result == 0,
                    is_msb_set(result as u64),
                    0,
                    false,
                    false
                );
            },

            ByteCodes::COMPARE_REG_REG_SIZED => {
                let size = self.get_next_byte();
                let left_reg = Registers::from(self.get_next_byte());
                let right_reg = Registers::from(self.get_next_byte());

                let result = self.registers.get_masked(left_reg, size) as i64
                    - self.registers.get_masked(right_reg, size) as i64;
            
                self.set_arithmetical_flags(
                    result == 0,
                    is_msb_set(result as u64),
                    0,
                    false,
                    false
                );
            }
            
            ByteCodes::COMPARE_REG_ADDR_IN_REG => {
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
            },
            
            ByteCodes::COMPARE_REG_CONST => {
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
            },
            
            ByteCodes::COMPARE_REG_ADDR_LITERAL => {
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
            },
            
            ByteCodes::COMPARE_ADDR_IN_REG_REG => {
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
            },
            
            ByteCodes::COMPARE_ADDR_IN_REG_ADDR_IN_REG => {
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
            },
            
            ByteCodes::COMPARE_ADDR_IN_REG_CONST => {
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
            },
            
            ByteCodes::COMPARE_ADDR_IN_REG_ADDR_LITERAL => {
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
            },
            
            ByteCodes::COMPARE_CONST_REG => {
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
            },
            
            ByteCodes::COMPARE_CONST_ADDR_IN_REG => {
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
            },
            
            ByteCodes::COMPARE_CONST_CONST => {
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
            },
            
            ByteCodes::COMPARE_CONST_ADDR_LITERAL => {
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
            },
            
            ByteCodes::COMPARE_ADDR_LITERAL_REG => {
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
            },
            
            ByteCodes::COMPARE_ADDR_LITERAL_ADDR_IN_REG => {
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
            },
            
            ByteCodes::COMPARE_ADDR_LITERAL_CONST => {
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
            },
            
            ByteCodes::COMPARE_ADDR_LITERAL_ADDR_LITERAL => {
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
            },
            
            ByteCodes::AND => {
                let result = self.registers.get(Registers::R1) & self.registers.get(Registers::R2);

                self.registers.set(Registers::R1, result);
        
                self.set_arithmetical_flags(
                    result == 0,
                    is_msb_set(result),
                    0,
                    false,
                    false
                );
            },
            
            ByteCodes::OR => {
                let result = self.registers.get(Registers::R1) | self.registers.get(Registers::R2);

                self.registers.set(Registers::R1, result);
        
                self.set_arithmetical_flags(
                    result == 0,
                    is_msb_set(result),
                    0,
                    false,
                    false
                );
            },
            
            ByteCodes::XOR => {
                let result = self.registers.get(Registers::R1) ^ self.registers.get(Registers::R2);

                self.registers.set(Registers::R1, result);
        
                self.set_arithmetical_flags(
                    result == 0,
                    is_msb_set(result),
                    0,
                    false,
                    true
                );
            },
            
            ByteCodes::NOT => {
                let result = !self.registers.get(Registers::R1);

                self.registers.set(Registers::R1, result);
        
                self.set_arithmetical_flags(
                    result == 0,
                    is_msb_set(result),
                    0,
                    false,
                    true
                );
            },
            
            ByteCodes::SHIFT_LEFT => {
                let value = self.registers.get(Registers::R1);
                let shift_amount = self.registers.get(Registers::R2);
        
                let result = value.overflowing_shl(shift_amount as u32).0;
        
                self.registers.set(Registers::R1, result);
            },
            
            ByteCodes::SHIFT_RIGHT => {
                let value = self.registers.get(Registers::R1);
                let shift_amount = self.registers.get(Registers::R2);
        
                let result = value.overflowing_shr(shift_amount as u32).0;
        
                self.registers.set(Registers::R1, result);
            },
            
            ByteCodes::INTERRUPT => {
                let intr_code = self.registers.get(Registers::INTERRUPT) as u8;
        
                self.handle_interrupt(intr_code);
            },
            
            ByteCodes::EXIT => {
                let exit_code_n = self.registers.get(Registers::EXIT) as u8;
                let exit_code = ErrorCodes::from(exit_code_n);
        
                if !self.quiet_exit {
                    println!("Program exited with code {} ({})", exit_code_n, exit_code);
                }
                
                std::process::exit(exit_code as i32);
            },
        }
    }


    /// Set the arithmetical flags
    #[inline]
    fn set_arithmetical_flags(&mut self, zf: bool, sf: bool, rf: u64, cf: bool, of: bool) {
        self.registers.set(Registers::ZERO_FLAG, zf as u64);
        self.registers.set(Registers::SIGN_FLAG, sf as u64);
        self.registers.set(Registers::REMAINDER_FLAG, rf);
        self.registers.set(Registers::CARRY_FLAG, cf as u64);
        self.registers.set(Registers::OVERFLOW_FLAG, of as u64);
    }


    fn handle_interrupt(&mut self, intr_code: u8) {

        match Interrupts::from(intr_code) {

            Interrupts::PrintSigned => {
                let value = self.registers.get(Registers::PRINT);
                print!("{}", value as i64);
                io::stdout().flush().expect("Failed to flush stdout");
            },

            Interrupts::PrintUnsigned => {
                let value = self.registers.get(Registers::PRINT);
                print!("{}", value);
                io::stdout().flush().expect("Failed to flush stdout");
            },

            Interrupts::PrintFloat => {
                let value = self.registers.get(Registers::PRINT);
                print!("{}", value as f64);
                io::stdout().flush().expect("Failed to flush stdout");
            }

            Interrupts::PrintChar => {
                let value = self.registers.get(Registers::PRINT);
                io::stdout().write_all(&[value as u8]).expect("Failed to write to stdout");
                io::stdout().flush().expect("Failed to flush stdout");
            },

            Interrupts::PrintString => {
                let string_address = self.registers.get(Registers::PRINT) as Address;
                let length = self.strlen(string_address);
                let bytes = self.memory.get_bytes(string_address, length);

                io::stdout().write_all(bytes).expect("Failed to write to stdout");
                io::stdout().flush().expect("Failed to flush stdout");
            },

            Interrupts::PrintBytes => {
                let bytes_address = self.registers.get(Registers::PRINT) as Address;
                let length = self.registers.get(Registers::R1) as usize;
                let bytes = self.memory.get_bytes(bytes_address, length);

                io::stdout().write_all(bytes).expect("Failed to write to stdout");
                io::stdout().flush().expect("Failed to flush stdout");
            },

            Interrupts::InputSignedInt => {
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
            },

            Interrupts::InputUnsignedInt => {
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
            },

            Interrupts::InputString => {
                
                let buf_addr = self.registers.get(Registers::R1) as Address;
                let size = self.registers.get(Registers::R2) as usize;
                    
                let buf = self.memory.get_bytes_mut(buf_addr, size);

                match io::stdin().read(buf) {
                    Ok(bytes_read) => {

                        // Check for EOF errors
                        if bytes_read == 0 {
                            self.registers.set_error(ErrorCodes::EndOfFile);
                            return;
                        }

                        self.registers.set(Registers::INPUT, bytes_read as u64);
                        self.registers.set_error(ErrorCodes::NoError); 
                    },

                    Err(_) => {
                        self.registers.set_error(ErrorCodes::GenericError);
                    },
                }
            },

            Interrupts::Random => {
                let mut rng = rand::thread_rng();
                let random_number = rng.gen_range(u64::MIN..u64::MAX);

                self.registers.set(Registers::R1, random_number);
            },

            Interrupts::HostTimeNanos => {
                // Casting to u64 will be ok until around 2500
                let time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos() as u64;

                self.registers.set(Registers::R1, time);
            },

            Interrupts::ElapsedTimeNanos => {
                // Casting to u64 will be ok until around 2500
                let time = SystemTime::now().duration_since(self.start_time).unwrap().as_nanos() as u64;

                self.registers.set(Registers::R1, time);
            },

            Interrupts::DiskRead => {
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
            },

            Interrupts::DiskWrite => {
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
            },

            Interrupts::Terminal => {
                let term_code = self.registers.get(Registers::PRINT); 

                let err = self.modules.terminal.handle_code(term_code as usize, &mut self.registers, &mut self.memory);
                self.registers.set_error(err);
            },

            Interrupts::SetTimerNanos => {
                let time = self.registers.get(Registers::R1);
                let duration = std::time::Duration::from_nanos(time);
        
                std::thread::sleep(duration);
            },

            Interrupts::FlushStdout => {
                io::stdout().flush().expect("Failed to flush stdout");
            },

            Interrupts::HostFs => {
                let fs_code = self.registers.get(Registers::PRINT);

                let err = self.modules.host_fs.handle_code(fs_code as usize, &mut self.registers, &mut self.memory);
                self.registers.set_error(err);
            },

        }
    }

}

