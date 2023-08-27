use std::fs;
use std::path::Path;

use rust_vm_lib::assembly::ByteCode;

use crate::error;


pub fn load_byte_code(file_path: &Path) -> ByteCode {
    fs::read(file_path).unwrap_or_else(
        |err| error::io_error(file_path, &err, format!("Failed to read file {}", file_path.display()).as_str())
    )
}

