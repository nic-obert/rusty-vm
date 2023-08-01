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

    let assembly = files::load_assembly(&args.input_file);

    let byte_code = assembler::assemble(assembly, args.verbose);
    
    if let Some(output) = &args.output {
        files::save_byte_code(byte_code, &output);
        
        if args.verbose {
            println!("\n\nAssembly code saved to {}", output);
        }

    } else {
        let output_file = files::save_byte_code(byte_code, &args.input_file);

        if args.verbose {
            println!("\n\nAssembly code saved to {}", output_file);
        }

    };
    
}

