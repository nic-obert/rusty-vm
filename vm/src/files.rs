use std::fs;
use std::path::Path;
use rust_vm_lib::assembly::ByteCode;


pub fn load_byte_code(file_path: &Path) -> ByteCode {
    fs::read(file_path)
        .expect(format!("Failed to read file {}", file_path.display()).as_str())
}

