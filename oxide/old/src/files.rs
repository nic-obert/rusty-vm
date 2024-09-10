use std::fs;
use std::path::Path;
use std::io;

use rusty_vm_lib::ir::SourceCode;

use crate::targets::CompiledBinary;


pub fn load_ir_code(file_path: &Path) -> io::Result<SourceCode> {

    let file_content = fs::read_to_string(file_path)?;
    
    Ok(
        file_content.lines().map(
            |line| line.to_string()
        ).collect()
    )
}


fn generate_output_name(input_name: &Path) -> String {
    
    input_name.with_extension("bc").to_str().unwrap().to_string()
}


pub fn save_byte_code(byte_code: &impl CompiledBinary, input_file_name: &Path) -> io::Result<String> {

    let output_name = generate_output_name(input_file_name);

    fs::write(&output_name, byte_code.as_bytes())?;
    
    Ok(output_name)
}

