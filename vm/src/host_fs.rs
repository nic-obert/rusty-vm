use std::path::Path;

use rust_vm_lib::registers::Registers;
use rust_vm_lib::vm::{ErrorCodes, Address};

use crate::memory::Memory;
use crate::register::CPURegisters;


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


type CodeHanlder = fn(&mut CPURegisters, &mut Memory) -> ErrorCodes;

const CODE_HANDLERS: [CodeHanlder; 1] = [
    handle_exists,
];

