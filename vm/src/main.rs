
mod processor;
mod memory;
mod video;
mod errors;
mod files;
mod cli_parser;

use std::path::Path;

use clap::Parser;

use cli_parser::CliParser;


fn main() {
 
    let args = CliParser::parse();

    let main_path = Path::new(&args.input_file).canonicalize().unwrap_or_else(
        |err| panic!("Failed to canonicalize path \"{}\"\n\n{}", args.input_file.display(), err)
    );

    if let Some(extension) = main_path.extension() {
        if extension != "bc" {
            println!("Warning: The input file extension is not \".bc\".");
        }
    }

    let byte_code = files::load_byte_code(&args.input_file);

    let mut processor = processor::Processor::new(args.stack_size, args.video_size);

    processor.execute(&byte_code, args.mode);

}

