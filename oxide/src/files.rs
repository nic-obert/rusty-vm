use std::fs;
use std::path::Path;
use std::io;

// use rusty_vm_lib::assembly::ByteCode;
use rusty_vm_lib::ir::IRCode;


pub fn load_ir_code(file_path: &Path) -> io::Result<IRCode> {

    let file_content = fs::read_to_string(file_path)?;
    
    Ok(
        file_content.lines().map(
            |line| line.to_string()
        ).collect()
    )
}


// fn generate_output_name(input_name: &Path) -> String {
    
//     input_name.with_extension("bc").to_str().unwrap().to_string()
// }


// pub fn save_byte_code(byte_code: ByteCode, input_file: &Path) -> io::Result<String> {

//     let output_name = generate_output_name(input_file);

//     fs::write(&output_name, byte_code)?;
    
//     Ok(output_name)
// }

