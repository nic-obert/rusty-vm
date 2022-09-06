mod assembler;
mod files;
mod token_to_byte_code;
mod token;
mod tokenizer;
mod argmuments_table;
use std::env;
use std::path::Path;


fn generate_output_name(input_name: &str) -> String {
    Path::new(input_name).file_stem().unwrap().to_str().unwrap().to_owned() + ".bc"
}


fn main() {

    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        println!("Usage: {} <input_file>", args[0]);
        return;
    }

    let input_file = &args[1];
    let assembly = files::load_assembly(input_file);

    let byte_code = assembler::assemble(assembly);

    let output_name = generate_output_name(input_file);
    files::save_byte_code(byte_code, &output_name);
    
}

