use std::{fs, path::Path};
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


fn generate_output_name(input_name: &str) -> String {
    Path::new(input_name).with_extension("bc").to_str().unwrap().to_string()
}


pub fn save_byte_code(byte_code: ByteCode, input_file_path: &str) -> String {
    let output_name = generate_output_name(input_file_path);
    fs::write(&output_name, byte_code)
       .expect(format!("Could not write to file {}", &output_name).as_str());
    output_name
}

