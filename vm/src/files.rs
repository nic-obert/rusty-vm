use std::fs;
use std::path::Path;
use rust_vm_lib::assembly::ByteCode;


pub fn load_byte_code(file_path: &Path) -> ByteCode {
    fs::read(file_path).unwrap_or_else(
        |error| panic!("Failed to read file {}.\n{}", file_path.display(), error)
    )
}

