use std::fs;
use std::path::Path;
use std::io;
use rust_vm_lib::assembly::{AssemblyCode, ByteCode};


pub fn load_assembly(file_path: &str) -> io::Result<AssemblyCode> {

    let file_content = fs::read_to_string(file_path)?;
    
    Ok(
        file_content.lines().map(
            |line| line.to_string()
        ).collect()
    )
}


fn generate_output_name(input_name: &str) -> String {
    
    Path::new(input_name).with_extension("bc").to_str().unwrap().to_string()
}


pub fn save_byte_code(byte_code: ByteCode, input_file_path: &str) -> io::Result<String> {

    let output_name = generate_output_name(input_file_path);

    fs::write(&output_name, byte_code)?;
    
    Ok(output_name)
}

