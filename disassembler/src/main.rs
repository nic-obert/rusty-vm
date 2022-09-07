mod files;
mod disassembler;
mod disassembly_table;
use clap::Parser;
use std::path::Path;


#[derive(Parser)]
#[clap(author, version, about)]
struct Cli {

    /// The input file path to disassemble
    #[clap(value_parser)]
    pub input_file: String,

    /// The output file path to write the disassembly to
    #[clap(short, long)]
    pub output: Option<String>,

    /// Run the disassembler in verbose mode
    #[clap(short, long, action)]
    pub verbose: bool,
}


fn generate_output_name(input_name: &str) -> String {
    Path::new(input_name).with_extension("dis").to_str().unwrap().to_string()
}


fn main() {

    let args = Cli::parse();

    let byte_code = files::load_byte_code(&args.input_file);
    let assembly = disassembler::disassemble(byte_code, args.verbose);

    if let Some(output) = &args.output {
        files::save_assembly_code(&output, &assembly);
    } else {
        let output = generate_output_name(&args.input_file);
        files::save_assembly_code(&output, &assembly);
    };

}

