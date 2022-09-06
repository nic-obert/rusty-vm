use std::fs;
use rust_vm_lib::assembly::{AssemblyCode, ByteCode};


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


pub fn save_byte_code(byte_code: ByteCode, file_path: &str) {
    fs::write(file_path, byte_code)
       .expect(format!("Could not write to file {}", file_path).as_str());
}

