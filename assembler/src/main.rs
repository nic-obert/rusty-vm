mod assembler;
mod files;
mod token_to_byte_code;
mod token;
mod registers;
mod byte_code;
mod tokenizer;
use std::env;


fn main() {

    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        println!("Usage: {} <input_file>", args[0]);
        return;
    }

    let input_file = &args[1];
    let assembly = files::load_assembly(input_file);
    let byte_code = assembler::assemble(assembly);
    
}

