use std::fs;
use crate::assembler::{AssemblyCode, ByteCode};


pub fn load_assembly(file_path: &str) -> AssemblyCode {
    let file_content = fs::read_to_string(file_path)
        .expect(format!("Could not read file {}", file_path).as_str());
    
    let mut lines = file_content.lines();
    let mut assembly_code: Vec<String> = Vec::new();
    
    while let Some(line) = lines.next() {
        assembly_code.push(String::from(line));
    }

    assembly_code
}


pub fn load_byte_code(file_path: &str) -> ByteCode {
    match fs::read(file_path) {
        Ok(bytes) => bytes,
        Err(e) => panic!("Could not read file {}: {}", file_path, e)
    }
}


pub fn save_byte_code(byte_code: ByteCode, file_path: &str) {
    match fs::write(file_path, byte_code) {
        Ok(_) => (),
        Err(e) => panic!("Could not write file {}: {}", file_path, e)
    }
}

