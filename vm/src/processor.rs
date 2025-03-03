use std::cmp::min;
use std::ffi::CString;
use std::io::Write;
use std::mem;
use std::io;
use std::ops::BitAnd;
use std::ops::Shl;
use std::os::fd::AsRawFd;
use std::path::PathBuf;
use std::ptr;
use std::ptr::NonNull;
use std::thread;
use std::time::{SystemTime, UNIX_EPOCH};
use num_traits::ops::overflowing::OverflowingAdd;
use num_traits::ops::overflowing::OverflowingSub;
use num_traits::Num;
use rand::Rng;
use libc::{poll, POLLIN, pollfd};

use rusty_vm_lib::assembly;
use rusty_vm_lib::debugger::CPU_REGISTERS_OFFSET;
use rusty_vm_lib::debugger::DEBUGGER_ATTACH_SLEEP;
use rusty_vm_lib::debugger::DEBUGGER_COMMAND_WAIT_SLEEP;
use rusty_vm_lib::debugger::DEBUGGER_PATH_ENV;
use rusty_vm_lib::debugger::RUNNING_FLAG_OFFSET;
use rusty_vm_lib::debugger::TERMINATE_COMMAND_OFFSET;
use rusty_vm_lib::debugger::VM_MEM_OFFSET;
use rusty_vm_lib::debugger::VM_UPDATED_COUNTER_OFFSET;
use rusty_vm_lib::registers::{Registers, CPURegisters};
use rusty_vm_lib::byte_code::ByteCodes;
use rusty_vm_lib::vm::{Address, ADDRESS_SIZE, ErrorCodes};
use rusty_vm_lib::interrupts::Interrupts;
use shared_memory::ShmemConf;

use crate::host_fs::HostFS;
use crate::memory::{Memory, Byte};
use crate::cli_parser::ExecutionMode;
use crate::error::{self, error};
use crate::modules::CPUModules;
use crate::storage::Storage;
use crate::terminal::Terminal;


fn libc_read_line() -> String {

    let fd = io::stdin().as_raw_fd();

    let mut line_ptr = ptr::null_mut();
    let mut line_size = 0;
    const FILE_OPEN_MODE: *const i8 = c"r".as_ptr();

    unsafe {

        let file = libc::fdopen(fd, FILE_OPEN_MODE);
        assert!(!file.is_null());

        let _ = libc::getline(&mut line_ptr, &mut line_size, file);

        CString::from_raw(line_ptr).into_string().unwrap()
    }
}


trait UnsignedInt: Num + OverflowingSub + OverflowingAdd + BitAnd<Self, Output = Self> + Shl<usize, Output = Self> + Copy {}
impl UnsignedInt for u8 {}
impl UnsignedInt for u16 {}
impl UnsignedInt for u32 {}
impl UnsignedInt for u64 {}

#[inline]
fn is_msb_set<T>(value: T) -> bool
where T: UnsignedInt // + BitAnd<T, Output = T> + Num + Shl<usize, Output = T>
{
    !(value & (T::one() << (mem::size_of::<T>()*8-1))).is_zero()
}


/// Converts a byte array to an integer
fn bytes_to_int(bytes: &[Byte], handled_size: Byte) -> u64 {
    // TODO: don't use try_into. use a more performant method that doesn't return an option. panic is ok here
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
#[inline]
unsafe fn bytes_as_address(bytes: &[Byte]) -> Address {
    Address::from_le_bytes(
        (bytes.as_ptr() as *const [Byte; ADDRESS_SIZE]).read()
    )
}


pub struct Processor {

    registers: CPURegisters,
    memory: Memory,
    start_time: SystemTime,
    modules: CPUModules,
    quiet_exit: bool,
    debug_mode_running: Option<*mut bool>,

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
            registers: CPURegisters::default(),
            memory: Memory::new(max_memory_size),
            // Initialize temporarily, will be reinitialized in `execute`
            start_time: SystemTime::now(),
            quiet_exit,
            modules: CPUModules::new(
                storage,
                Terminal::new(),
                HostFS::new()
            ),
            debug_mode_running: None,
        }
    }


    /// Execute the given bytecode
    pub fn execute(&mut self, byte_code: &[Byte], mode: ExecutionMode) {

        // Set the program counter to the start of the program

        if byte_code.len() < ADDRESS_SIZE {
            error::error(format!("Bytecode is too small to contain a start address: minimum required size is {} bytes, got {}", ADDRESS_SIZE, byte_code.len()).as_str());
        }

        let program_start: Address = unsafe { bytes_as_address(&byte_code[byte_code.len() - ADDRESS_SIZE..]) };
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
            ExecutionMode::Debug => {
                self.run_debug();
            }
        }
    }


    fn run_debug(&mut self) {

        let debugger_path = std::env::var(DEBUGGER_PATH_ENV).expect("Missing debugger");

        let shmem_size = VM_MEM_OFFSET + self.memory.size_bytes();

        let shmem = ShmemConf::new()
            .size(shmem_size)
            .create()
            .unwrap();

        unsafe {
            let new_vm_mem = NonNull::slice_from_raw_parts(
                NonNull::new(shmem.as_ptr().byte_add(VM_MEM_OFFSET)).expect("Should be valid"),
                self.memory.size_bytes()
            );

            self.memory.set_shared_buffer(new_vm_mem);
        };
        let running_flag = unsafe { shmem.as_ptr().byte_add(RUNNING_FLAG_OFFSET).cast::<bool>() };
        let terminate_command = unsafe { shmem.as_ptr().byte_add(TERMINATE_COMMAND_OFFSET).cast::<bool>() };
        let vm_updated_counter = unsafe { shmem.as_ptr().byte_add(VM_UPDATED_COUNTER_OFFSET) };
        let cpu_registers = unsafe { shmem.as_ptr().byte_add(CPU_REGISTERS_OFFSET).cast::<CPURegisters>() };

        self.debug_mode_running = Some(running_flag);

        unsafe {
            running_flag.write_volatile(false);
            terminate_command.write_volatile(false);
            vm_updated_counter.write_volatile(0);
            cpu_registers.write_volatile(self.registers.clone());
        }

        // Launch the debugger process
        let mut debugger_process = std::process::Command::new(debugger_path)
            .arg(shmem.get_os_id())
            .spawn()
            .expect("Failed to start debugger process");

        // Wait for the debugger to be ready
        println!("Waiting for debugger");
        while unsafe { !running_flag.read_volatile() } {
            thread::sleep(DEBUGGER_ATTACH_SLEEP);
        }
        println!("Debugger connected");
        while unsafe { !terminate_command.read_volatile() } {

            if unsafe { running_flag.read_volatile() } {

                let opcode = self.get_next_item();
                self.handle_instruction(opcode);

            } else {

                unsafe {
                    // When the VM is stopped, communicate the current registers state to the debugger
                    cpu_registers.write_volatile(self.registers.clone());
                    // Tell the debugger the VM has finished sending data
                    vm_updated_counter.write_volatile(vm_updated_counter.read_volatile().wrapping_add(1));
                }

                // Wait until execution is resumed by the debugger
                while unsafe { !running_flag.read_volatile() } {
                    thread::sleep(DEBUGGER_COMMAND_WAIT_SLEEP);
                }
            }

        }

        match debugger_process.wait() {
            Ok(status) => println!("Debugger exited with {}", status),
            Err(err) => println!("Error waiting debugger process: {}", err),
        }


    }


    #[must_use]
    fn subtract_set_flags<T>(&mut self, left: T, right: T) -> T
    where T: UnsignedInt
    {
        let (result, carry) = left.overflowing_sub(&right);

        let sign_left = is_msb_set(left);
        let sign_right = is_msb_set(right);
        let sign_result = is_msb_set(result);

        let overflow = {
            !sign_left && sign_right && sign_result
            || sign_left && !sign_right && !sign_result
        };

        self.set_arithmetical_flags(
            result.is_zero(),
            sign_result,
            0,
            carry,
            overflow
        );

        result
    }


    #[must_use]
    fn add_set_flags<T>(&mut self, left: T, right: T) -> T
    where T: UnsignedInt
    {
        let (result, carry) = left.overflowing_add(&right);

        let sign_left = is_msb_set(left);
        let sign_right = is_msb_set(right);
        let sign_result = is_msb_set(result);

        let overflow = {
            !sign_left && !sign_right && sign_result
            || sign_left && sign_right && !sign_result
        };

        self.set_arithmetical_flags(
            result.is_zero(),
            sign_result,
            0,
            carry,
            overflow
        );

        result
    }


    fn compare_set_flags(&mut self, left: u64, right: u64) {
        let _ = self.subtract_set_flags(left, right);
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

        let bytes = self.memory.get_bytes(address, size as usize);

        match size {

            1 => {
                let value = bytes[0];
                let res = self.add_set_flags(value, 1);
                self.memory.set_byte(address, res);
            },

            2 => {
                let value = u16::from_le_bytes(bytes.try_into().unwrap());
                let res = self.add_set_flags(value, 1);
                self.memory.set_bytes(address, &res.to_le_bytes());
            },

            4 => {
                let value = u32::from_le_bytes(bytes.try_into().unwrap());
                let res = self.add_set_flags(value, 1);
                self.memory.set_bytes(address, &res.to_le_bytes());
            },

            8 => {
                let value = u64::from_le_bytes(bytes.try_into().unwrap());
                let res = self.add_set_flags(value, 1);
                self.memory.set_bytes(address, &res.to_le_bytes());
            },

            _ => error::error(format!("Invalid size for incrementing bytes: {}.", size).as_str()),
        };
    }


    /// Decrement the `size`-sized value at the given address
    fn decrement_bytes(&mut self, address: Address, size: Byte) {

        let bytes = self.memory.get_bytes(address, size as usize);

        match size {

            1 => {
                let value = bytes[0];
                let res = self.subtract_set_flags(value, 1);
                self.memory.set_byte(address, res);
            },

            2 => {
                let value = u16::from_le_bytes(bytes.try_into().unwrap());
                let res = self.subtract_set_flags(value, 1);
                self.memory.set_bytes(address, &res.to_le_bytes());
            },

            4 => {
                let value = u32::from_le_bytes(bytes.try_into().unwrap());
                let res = self.subtract_set_flags(value, 1);
                self.memory.set_bytes(address, &res.to_le_bytes());
            },

            8 => {
                let value = u64::from_le_bytes(bytes.try_into().unwrap());
                let res = self.subtract_set_flags(value, 1);
                self.memory.set_bytes(address, &res.to_le_bytes());
            },

            _ => error::error(format!("Invalid size for decrementing bytes: {}.", size).as_str()),
        };
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


    /// Get the next item of type `T` in the bytecode and update the program counter
    fn get_next_item<T>(&mut self) -> T {
        let pc = self.registers.pc();
        self.registers.inc_pc(mem::size_of::<T>());
        self.memory.read::<T>(pc)
    }


    /// Get the next byte in the bytecode
    fn get_next_byte(&mut self) -> Byte {
        let pc = self.registers.pc();
        self.registers.inc_pc(1);
        self.memory.get_byte(pc)
    }


    fn run(&mut self) {
        loop {
            let opcode = self.get_next_item();
            self.handle_instruction(opcode);
        }
    }


    fn run_interactive(&mut self, byte_code_size: usize) {

        println!("Running VM in interactive mode");
        println!("Warning: interactive mode reads lines from stdin to advence the program counter.");
        println!("Byte code size is {} bytes", byte_code_size);
        println!("Start address is: {}", self.registers.pc());
        println!();

        loop {

            let opcode = self.get_next_item();

            println!();

            println!("PC: {}, opcode: {}", self.registers.pc(), opcode);

            let (handled_size, args) = assembly::parse_bytecode_args(opcode, &self.memory.get_raw()[self.registers.pc()..])
                .unwrap_or_else(|err| error(format!("Could not parse arguments for opcode {opcode}:\n{err}").as_str()));

            print!("Instruction args (handled size {handled_size}): ");
            for arg in args {
                print!("{} ", arg);
            }
            println!();

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
            let opcode = self.get_next_item();
            println!("PC: {}, opcode: {}", self.registers.pc(), opcode);
            self.handle_instruction(opcode);
        }
    }


    fn handle_instruction(&mut self, opcode: ByteCodes) {
        match opcode {

            ByteCodes::INTEGER_ADD => {
                let r1 = self.registers.get(Registers::R1);
                let r2 = self.registers.get(Registers::R2);

                let result = self.add_set_flags(r1, r2);
                self.registers.set(Registers::R1, result);
            },

            ByteCodes::INTEGER_SUB => {
                let r1 = self.registers.get(Registers::R1);
                let r2 = self.registers.get(Registers::R2);

                let result = self.subtract_set_flags(r1, r2);
                self.registers.set(Registers::R1, result);
            },

            ByteCodes::INTEGER_MUL => {
                let r1 = self.registers.get(Registers::R1);
                let r2 = self.registers.get(Registers::R2);

                let (result, carry) = r1.overflowing_mul(r2);

                let sign_left = is_msb_set(r1);
                let sign_right = is_msb_set(r2);
                let sign_result = is_msb_set(result);

                let overflow = {
                    (sign_left != sign_right) && !sign_result
                    || (sign_left == sign_right) && sign_result
                };

                self.registers.set(Registers::R1, result);

                self.set_arithmetical_flags(
                    result == 0,
                    sign_result,
                    0,
                    carry,
                    overflow
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

                let result = self.add_set_flags(value, 1);
                self.registers.set(dest_reg, result);
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

                let result = self.subtract_set_flags(value, 1);
                self.registers.set(dest_reg, result);
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

            ByteCodes::MEM_COPY_BLOCK_REG => {
                let dest_address = self.registers.get(Registers::R1) as Address;
                let src_address = self.registers.get(Registers::R2) as Address;
                let size_reg = Registers::from(self.get_next_byte());
                let copy_size = self.registers.get(size_reg) as usize;

                self.memory.memcpy(src_address, dest_address, copy_size);
            },

            ByteCodes::MEM_COPY_BLOCK_REG_SIZED => {
                let size = self.get_next_byte();
                let dest_address = self.registers.get(Registers::R1) as Address;
                let src_address = self.registers.get(Registers::R2) as Address;
                let size_reg = Registers::from(self.get_next_byte());
                let copy_size = self.registers.get_masked(size_reg, size) as usize;

                self.memory.memcpy(src_address, dest_address, copy_size);
            },

            ByteCodes::MEM_COPY_BLOCK_ADDR_IN_REG => {
                let size = self.get_next_byte();
                let dest_address = self.registers.get(Registers::R1) as Address;
                let src_address = self.registers.get(Registers::R2) as Address;
                let size_reg = Registers::from(self.get_next_byte());
                let size_address = self.registers.get(size_reg) as Address;
                let copy_size = bytes_to_int(
                    self.memory.get_bytes(size_address, size as usize),
                    size
                ) as usize;

                self.memory.memcpy(src_address, dest_address, copy_size);
            },

            ByteCodes::MEM_COPY_BLOCK_CONST => {
                let size = self.get_next_byte();
                let dest_address = self.registers.get(Registers::R1) as Address;
                let src_address = self.registers.get(Registers::R2) as Address;
                let copy_size =  bytes_to_int(
                    self.get_next_bytes(size as usize),
                    size
                ) as usize;

                self.memory.memcpy(src_address, dest_address, copy_size);
            },

            ByteCodes::MEM_COPY_BLOCK_ADDR_LITERAL => {
                let size = self.get_next_byte();
                let dest_address = self.registers.get(Registers::R1) as Address;
                let src_address = self.registers.get(Registers::R2) as Address;
                let size_address = self.get_next_address();
                let copy_size = bytes_to_int(
                    self.memory.get_bytes(size_address, size as usize),
                    size
                ) as usize;

                self.memory.memcpy(src_address, dest_address, copy_size);
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

                if self.registers.get(Registers::SIGN_FLAG) == 0
                    && self.registers.get(Registers::ZERO_FLAG) == 0
                {
                    self.jump_to(jump_address);
                }
            },

            ByteCodes::JUMP_LESS => {
                let jump_address = self.get_next_address();

                if self.registers.get(Registers::SIGN_FLAG) == 1 {
                    self.jump_to(jump_address);
                }
            },

            ByteCodes::JUMP_GREATER_OR_EQUAL => {
                let jump_address = self.get_next_address();

                if self.registers.get(Registers::SIGN_FLAG) == 0 {
                    self.jump_to(jump_address);
                }
            },

            ByteCodes::JUMP_LESS_OR_EQUAL => {
                let jump_address = self.get_next_address();

                if self.registers.get(Registers::SIGN_FLAG) == 1
                    || self.registers.get(Registers::ZERO_FLAG) == 1
                {
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

            ByteCodes::CALL_CONST => {
                let jump_address = self.get_next_address();

                // Push the return address onto the stack (return address is the current pc)
                self.push_stack(self.registers.pc() as u64);

                // Jump to the subroutine
                self.jump_to(jump_address);
            },

            ByteCodes::CALL_REG => {
                let jump_reg = Registers::from(self.get_next_byte());

                let jump_address = self.registers.get(jump_reg) as Address;

                self.push_stack(self.registers.pc() as u64);

                self.jump_to(jump_address);
            }

            ByteCodes::RETURN => {
                // Get the return address from the stack
                let return_address = unsafe { bytes_as_address(
                    self.pop_stack_bytes(ADDRESS_SIZE)
                ) };

                // Jump to the return address
                self.jump_to(return_address);
            },

            ByteCodes::COMPARE_REG_REG => {
                let left_reg = Registers::from(self.get_next_byte());
                let right_reg = Registers::from(self.get_next_byte());
                let left = self.registers.get(left_reg);
                let right = self.registers.get(right_reg);

                self.compare_set_flags(left, right);
            },

            ByteCodes::COMPARE_REG_REG_SIZED => {
                let size = self.get_next_byte();
                let left_reg = Registers::from(self.get_next_byte());
                let right_reg = Registers::from(self.get_next_byte());
                let left = self.registers.get_masked(left_reg, size);
                let right = self.registers.get_masked(right_reg, size);

                self.compare_set_flags(left, right);
            }

            ByteCodes::COMPARE_REG_ADDR_IN_REG => {
                let size = self.get_next_byte();

                let left_reg = Registers::from(self.get_next_byte());
                let left = self.registers.get(left_reg);

                let right_address_reg = Registers::from(self.get_next_byte());
                let right_address = self.registers.get(right_address_reg) as Address;
                let right = bytes_to_int(self.memory.get_bytes(right_address, size as usize), size);

                self.compare_set_flags(left, right);
            },

            ByteCodes::COMPARE_REG_CONST => {
                let size = self.get_next_byte();

                let left_reg = Registers::from(self.get_next_byte());
                let left = self.registers.get(left_reg);

                let right = bytes_to_int(self.get_next_bytes(size as usize), size);

                self.compare_set_flags(left, right);
            },

            ByteCodes::COMPARE_REG_ADDR_LITERAL => {
                let size = self.get_next_byte();

                let left_reg = Registers::from(self.get_next_byte());
                let left = self.registers.get(left_reg);

                let right_address = self.get_next_address();
                let right = bytes_to_int(self.memory.get_bytes(right_address, size as usize), size);

                self.compare_set_flags(left, right);
            },

            ByteCodes::COMPARE_ADDR_IN_REG_REG => {
                let size = self.get_next_byte();

                let left_address_reg = Registers::from(self.get_next_byte());
                let left_address = self.registers.get(left_address_reg) as Address;
                let left = bytes_to_int(self.memory.get_bytes(left_address, size as usize), size);

                let right_reg = Registers::from(self.get_next_byte());
                let right = self.registers.get(right_reg);

                self.compare_set_flags(left, right);
            },

            ByteCodes::COMPARE_ADDR_IN_REG_ADDR_IN_REG => {
                let size = self.get_next_byte();

                let left_address_reg = Registers::from(self.get_next_byte());
                let left_address = self.registers.get(left_address_reg) as Address;
                let left = bytes_to_int(self.memory.get_bytes(left_address, size as usize), size);

                let right_address_reg = Registers::from(self.get_next_byte());
                let right_address = self.registers.get(right_address_reg) as Address;
                let right = bytes_to_int(self.memory.get_bytes(right_address, size as usize), size);

                self.compare_set_flags(left, right);
            },

            ByteCodes::COMPARE_ADDR_IN_REG_CONST => {
                let size = self.get_next_byte();

                let left_address_reg = Registers::from(self.get_next_byte());
                let left_address = self.registers.get(left_address_reg) as Address;
                let left = bytes_to_int(self.memory.get_bytes(left_address, size as usize), size);

                let right = bytes_to_int(self.get_next_bytes(size as usize), size);

                self.compare_set_flags(left, right);
            },

            ByteCodes::COMPARE_ADDR_IN_REG_ADDR_LITERAL => {
                let size = self.get_next_byte();

                let left_address_reg = Registers::from(self.get_next_byte());
                let left_address = self.registers.get(left_address_reg) as Address;
                let left = bytes_to_int(self.memory.get_bytes(left_address, size as usize), size);

                let right_address = self.get_next_address();
                let right = bytes_to_int(self.memory.get_bytes(right_address, size as usize), size);

                self.compare_set_flags(left, right);
            },

            ByteCodes::COMPARE_CONST_REG => {
                let size = self.get_next_byte();

                let left_address = self.get_next_bytes(size as usize);
                let left = bytes_to_int(left_address, size);

                let right_reg = Registers::from(self.get_next_byte());
                let right = self.registers.get(right_reg);

                self.compare_set_flags(left, right);
            },

            ByteCodes::COMPARE_CONST_ADDR_IN_REG => {
                let size = self.get_next_byte();

                let left_address = self.get_next_bytes(size as usize);
                let left = bytes_to_int(left_address, size);

                let right_address_reg = Registers::from(self.get_next_byte());
                let right_address = self.registers.get(right_address_reg) as Address;
                let right = bytes_to_int(self.memory.get_bytes(right_address, size as usize), size);

                self.compare_set_flags(left, right);
            },

            ByteCodes::COMPARE_CONST_CONST => {
                let size = self.get_next_byte();

                let left_address = self.get_next_bytes(size as usize);
                let left = bytes_to_int(left_address, size);

                let right_address = self.get_next_bytes(size as usize);
                let right = bytes_to_int(right_address, size);

                self.compare_set_flags(left, right);
            },

            ByteCodes::COMPARE_CONST_ADDR_LITERAL => {
                let size = self.get_next_byte();

                let left_address = self.get_next_bytes(size as usize);
                let left = bytes_to_int(left_address, size);

                let right_address = self.get_next_address();
                let right = bytes_to_int(self.memory.get_bytes(right_address, size as usize), size);

                self.compare_set_flags(left, right);
            },

            ByteCodes::COMPARE_ADDR_LITERAL_REG => {
                let size = self.get_next_byte();

                let left_address = self.get_next_address();
                let left = bytes_to_int(self.memory.get_bytes(left_address, size as usize), size);

                let right_reg = Registers::from(self.get_next_byte());
                let right = self.registers.get(right_reg);

                self.compare_set_flags(left, right);
            },

            ByteCodes::COMPARE_ADDR_LITERAL_ADDR_IN_REG => {
                let size = self.get_next_byte();

                let left_address = self.get_next_address();
                let left = bytes_to_int(self.memory.get_bytes(left_address, size as usize), size);

                let right_address_reg = Registers::from(self.get_next_byte());
                let right_address = self.registers.get(right_address_reg) as Address;
                let right = bytes_to_int(self.memory.get_bytes(right_address, size as usize), size);

                self.compare_set_flags(left, right);
            },

            ByteCodes::COMPARE_ADDR_LITERAL_CONST => {
                let size = self.get_next_byte();

                let left_address = self.get_next_address();
                let left = bytes_to_int(self.memory.get_bytes(left_address, size as usize), size);

                let right_address = self.get_next_bytes(size as usize);
                let right = bytes_to_int(right_address, size);

                self.compare_set_flags(left, right);
            },

            ByteCodes::COMPARE_ADDR_LITERAL_ADDR_LITERAL => {
                let size = self.get_next_byte();

                let left_address = self.get_next_address();
                let left = bytes_to_int(self.memory.get_bytes(left_address, size as usize), size);

                let right_address = self.get_next_address();
                let right = bytes_to_int(self.memory.get_bytes(right_address, size as usize), size);

                self.compare_set_flags(left, right);
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

            ByteCodes::SWAP_BYTES_ENDIANNESS => {
                let value = self.registers.get(Registers::R1);

                self.registers.set(Registers::R1, value.swap_bytes());
            },

            ByteCodes::INTERRUPT => {
                let intr_code = self.registers.get(Registers::INTERRUPT) as u8;

                self.handle_interrupt(intr_code);
            },

            ByteCodes::BREAKPOINT => {
                if let Some(debug_running) = self.debug_mode_running {
                    unsafe {
                        debug_running.write_volatile(false);
                    }
                } else {
                    println!("Breakpoint debug instruction encountered in non-debug execution mode.");
                    self.registers.set_error(ErrorCodes::PermissionDenied);
                    self.handle_instruction(ByteCodes::EXIT);
                }
            },

            ByteCodes::EXIT => {
                let exit_code_n = self.registers.get(Registers::ERROR) as u8;
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
        // TODO: group the flags into a single immutable register.
        // This may increase performance since operations on flags remain in registers instead of
        // going to memory every time.
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

                let line  = libc_read_line();

                // Check for EOF errors
                if line.is_empty() {
                    self.registers.set_error(ErrorCodes::EndOfFile);
                    return;
                }

                match line.as_str().trim().parse::<i64>() {
                    Ok(value) => {
                        self.registers.set(Registers::INPUT, value as u64);
                        self.registers.set_error(ErrorCodes::NoError);
                    },
                    Err(_) => {
                        self.registers.set_error(ErrorCodes::InvalidInput);
                    }
                }

            },

            Interrupts::InputUnsignedInt => {

                let line  = libc_read_line();

                // Check for EOF errors
                if line.is_empty() {
                    self.registers.set_error(ErrorCodes::EndOfFile);
                    return;
                }

                if line.is_empty() {
                    self.registers.set_error(ErrorCodes::EndOfFile);
                    return;
                }

                match line.as_str().trim().parse::<u64>() {
                    Ok(value) => {
                        self.registers.set(Registers::INPUT, value);
                        self.registers.set_error(ErrorCodes::NoError);
                    },
                    Err(_) => {
                        self.registers.set_error(ErrorCodes::InvalidInput);
                    }
                }
            },

            Interrupts::InputByte => {
                // Use libc because Rust's io functions alter the stdin events
                const BUF_SIZE: usize = 1;
                let mut buf: [u8; BUF_SIZE] = [0];
                let fd = io::stdin().as_raw_fd();
                let bytes_read = unsafe {
                    libc::read(fd, buf.as_mut_ptr() as _, BUF_SIZE)
                };
                if bytes_read == BUF_SIZE as isize {
                    self.registers.set_error(ErrorCodes::NoError);
                    self.registers.set(Registers::INPUT, buf[0] as u64);
                } else {
                    self.registers.set_error(ErrorCodes::EndOfFile);
                }
            },

            Interrupts::InputString => {

                let buf_addr = self.registers.get(Registers::R1) as Address;
                let size = self.registers.get(Registers::R2) as usize;

                let buf = self.memory.get_bytes_mut(buf_addr, size);

                let fd = io::stdin().as_raw_fd();
                let bytes_read = unsafe {
                    libc::read(fd, buf.as_mut_ptr() as _, size)
                };

                assert!(bytes_read >= -1);
                if bytes_read == -1 {
                    // Unexpected error occurred
                    let errno = unsafe {
                        libc::__errno_location().read()
                    };
                    panic!("Failed to read from stdin. Errno: {}", errno);
                } else if (bytes_read as usize) < size || bytes_read == 0 {
                    self.registers.set_error(ErrorCodes::EndOfFile);
                } else {
                    self.registers.set_error(ErrorCodes::NoError);
                }

                self.registers.set(Registers::INPUT, bytes_read as u64);
            },

            Interrupts::StdinHasData => {
                let fd = io::stdin().as_raw_fd();
                const PFD_STRUCTURE_COUNT: u64 = 1;
                const POLL_TIMEOUT: i32 = 0;
                let mut pfd = pollfd {
                    fd,
                    events: POLLIN,
                    revents: 0
                };
                let _ = unsafe {
                    poll(&mut pfd, PFD_STRUCTURE_COUNT, POLL_TIMEOUT)
                };
                let stdin_is_empty = (pfd.revents & POLLIN) != 0;
                self.registers.set(Registers::R1, stdin_is_empty as u64);
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
