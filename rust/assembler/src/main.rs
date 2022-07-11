use std::env;
mod shared;
use shared::files;
mod assembler;



fn main() {

    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        println!("Usage: {} <input_file>", args[0]);
        return;
    }

    let input_file = &args[1];
    let assembly = files::load_assembly(input_file);

    


}