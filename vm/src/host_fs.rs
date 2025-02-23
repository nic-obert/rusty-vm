use std::fs;
use std::path::Path;

use rusty_vm_lib::registers::{Registers, CPURegisters};
use rusty_vm_lib::vm::{ErrorCodes, Address};

use crate::memory::Memory;


pub struct HostFS;

impl HostFS {

    pub fn new() -> Self {
        Self {}
    }


    pub fn handle_code(&self, code: usize, registers: &mut CPURegisters, memory: &mut Memory) -> ErrorCodes {
        CODE_HANDLERS[code](registers, memory)
    }

}


fn read_until_null(data: &[u8]) -> Option<&[u8]> {
    let end = data.iter().position(|&b| b == 0)?;
    Some(&data[..end])
}


fn handle_exists(registers: &mut CPURegisters, memory: &mut Memory) -> ErrorCodes {

    let path_address = registers.get(Registers::R1) as Address;

    let file_path = if let Some(raw_bytes) = read_until_null(&memory.get_raw()[path_address..]) {
        match std::str::from_utf8(raw_bytes) {
            Ok(s) => Path::new(s),
            Err(_) => return ErrorCodes::InvalidInput
        }
    } else {
        return ErrorCodes::InvalidInput;
    };

    registers.set(Registers::R1, file_path.exists() as u64);

    ErrorCodes::NoError
}


fn handle_read_all(registers: &mut CPURegisters, memory: &mut Memory) -> ErrorCodes {

    let path_address = registers.get(Registers::R1) as Address;
    let buffer_address = registers.get(Registers::R2) as Address;

    let file_path = if let Some(raw_bytes) = read_until_null(&memory.get_raw()[path_address..]) {
        match std::str::from_utf8(raw_bytes) {
            Ok(s) => Path::new(s),
            Err(_) => return ErrorCodes::InvalidInput
        }
    } else {
        return ErrorCodes::InvalidInput;
    };

    match fs::read(file_path) {
        Ok(bytes) => {
            memory.set_bytes(buffer_address, &bytes);
            ErrorCodes::NoError
        },
        Err(e) => ErrorCodes::from(e)
    }
}


fn handle_write_all(registers: &mut CPURegisters, memory: &mut Memory) -> ErrorCodes {

    let path_address = registers.get(Registers::R1) as Address;
    let buffer_address = registers.get(Registers::R2) as Address;
    let buffer_size = registers.get(Registers::R3) as usize;

    let file_path = if let Some(raw_bytes) = read_until_null(&memory.get_raw()[path_address..]) {
        match std::str::from_utf8(raw_bytes) {
            Ok(s) => Path::new(s),
            Err(_) => return ErrorCodes::InvalidInput
        }
    } else {
        return ErrorCodes::InvalidInput;
    };

    fs::write(file_path, memory.get_bytes(buffer_address, buffer_size)).into()
}


fn handle_create_file(registers: &mut CPURegisters, memory: &mut Memory) -> ErrorCodes {

    let path_address = registers.get(Registers::R1) as Address;

    let file_path = if let Some(raw_bytes) = read_until_null(&memory.get_raw()[path_address..]) {
        match std::str::from_utf8(raw_bytes) {
            Ok(s) => Path::new(s),
            Err(_) => return ErrorCodes::InvalidInput
        }
    } else {
        return ErrorCodes::InvalidInput;
    };

    fs::File::create(file_path).into()

}


fn handle_create_dir(registers: &mut CPURegisters, memory: &mut Memory) -> ErrorCodes {

    let path_address = registers.get(Registers::R1) as Address;

    let dir_path = if let Some(raw_bytes) = read_until_null(&memory.get_raw()[path_address..]) {
        match std::str::from_utf8(raw_bytes) {
            Ok(s) => Path::new(s),
            Err(_) => return ErrorCodes::InvalidInput
        }
    } else {
        return ErrorCodes::InvalidInput;
    };

    fs::create_dir_all(dir_path).into()

}


type CodeHanlder = fn(&mut CPURegisters, &mut Memory) -> ErrorCodes;

const CODE_HANDLERS: [CodeHanlder; 5] = [
    handle_exists, // 0
    handle_read_all, // 1
    handle_write_all, // 2
    handle_create_file, //3
    handle_create_dir, // 4
];
