mod files;
mod disassembler;
mod disassembly_table;
use std::env;
use std::path::Path;


fn generate_output_name(input_name: &str) -> String {
    Path::new(input_name).file_stem().unwrap().to_str().unwrap().to_owned() + ".asm"
}


fn main() {

    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        println!("Usage: {} <input file>", args[0]);
        return;
    }

    let input_name = &args[1];
    let byte_code = files::load_byte_code(input_name);
    let assembly = disassembler::disassemble(byte_code);

    let output_name = generate_output_name(input_name);
    files::save_assembly_code(&output_name, &assembly);

}

