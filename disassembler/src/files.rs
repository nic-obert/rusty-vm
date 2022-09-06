use std::fs;
use rust_vm_lib::assembly::{AssemblyCode, ByteCode};


pub fn load_byte_code(file_path: &str) -> ByteCode {
    fs::read(file_path)
        .expect(format!("Failed to read file {}", file_path).as_str())
}


pub fn save_assembly_code(file_path: &str, assembly_code: &AssemblyCode) {
    fs::write(file_path, assembly_code.join("\n"))
        .expect(format!("Failed to write file {}", file_path).as_str())

}

