mod assembler;
mod files;
mod token_to_byte_code;
mod tokenizer;
mod argmuments_table;
mod error;
mod data_types;
mod encoding;
use clap::Parser;


#[derive(Parser)]
#[clap(author, version, about)]
struct Cli {

    /// The input file path to assemble
    #[clap(value_parser)]
    pub input_file: String,

    /// The output file path to write the byte code to
    #[clap(short, long)]
    pub output: Option<String>,

    /// Run the assembler in verbose mode
    #[clap(short, long, action)]
    pub verbose: bool,
}


fn main() {

    let args = Cli::parse();

    let assembly = match files::load_assembly(&args.input_file) {

        Ok(assembly) => assembly,

        Err(error) => {
            error::io_error("_start_", &error, format!("Failed to load assembly file \"{}\"", &args.input_file).as_str());
        }

    };

    let byte_code = assembler::assemble(assembly, args.verbose, &args.input_file);
    
    if let Some(output) = &args.output {
        match files::save_byte_code(byte_code, &output) {

            Ok(_) => {}

            Err(error) => {
                error::io_error("_start_", &error, format!("Failed to save byte code to \"{}\"", &output).as_str());
            }

        };
        
        if args.verbose {
            println!("\n\nAssembly code saved to {}", output);
        }

    } else {
        let output_file = match files::save_byte_code(byte_code, &args.input_file) {

            Ok(output_file) => output_file,

            Err(error) => {
                error::io_error("_start_", &error, format!("Failed to save byte code to \"{}\"", &args.input_file).as_str());
            }

        };

        if args.verbose {
            println!("\n\nAssembly code saved to {}", output_file);
        }

    };
    
}

