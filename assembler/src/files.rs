use std::fs;
use std::path::Path;
use std::io;
use rusty_vm_lib::assembly::ByteCode;


pub fn save_byte_code(byte_code: ByteCode, output_name: &Path) -> io::Result<()> {

    fs::write(output_name, byte_code)
    
}

