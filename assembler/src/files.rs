use std::fs;
use std::path::Path;
use std::io;
use rusty_vm_lib::assembly::ByteCode;


pub fn load_assembly(file_path: &Path) -> io::Result<String> {

    let file_content = fs::read_to_string(file_path)?;
    
    Ok(
        file_content.lines().map(
            |line| line.to_string()
        ).collect()
    )
}


pub fn save_byte_code(byte_code: ByteCode, output_name: &Path) -> io::Result<()> {

    fs::write(output_name, byte_code)
    
}

