mod assembler;
mod files;
mod token_to_byte_code;
mod tokenizer;
mod argmuments_table;
mod error;
mod data_types;
mod encoding;
mod configs;

use std::path::Path;
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

    let phantom_path = Path::new("_start_");
    let main_path = Path::new(&args.input_file).canonicalize().unwrap_or_else(
        |err| error::io_error(phantom_path, &err, format!("Failed to canonicalize path \"{}\"", &args.input_file).as_str())
    );

    let assembly = match files::load_assembly(&main_path) {

        Ok(assembly) => assembly,

        Err(error) => {
            error::io_error(phantom_path, &error, format!("Failed to load assembly file \"{}\"", &args.input_file).as_str());
        }

    };

    let byte_code = assembler::assemble(assembly, args.verbose, &main_path);
    
    if let Some(output_raw) = &args.output {

        let output_path = Path::new(output_raw);

        match files::save_byte_code(byte_code, output_path) {

            Ok(_) => {}

            Err(error) => {
                error::io_error(phantom_path, &error, format!("Failed to save byte code to \"{}\"", output_path.display()).as_str());
            }

        };
        
        if args.verbose {
            println!("\n\nAssembly code saved to {}", output_path.display());
        }

    } else {
        let output_file = match files::save_byte_code(byte_code, &main_path) {

            Ok(output_file) => output_file,

            Err(error) => {
                error::io_error(phantom_path, &error, format!("Failed to save byte code to \"{}\"", &args.input_file).as_str());
            }

        };

        if args.verbose {
            println!("\n\nAssembly code saved to {}", output_file);
        }

    };
    
}

