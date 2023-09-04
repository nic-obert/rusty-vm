mod allocator;
mod processor;
mod memory;
mod files;
mod cli_parser;
mod error;
mod storage;
mod terminal;

use std::path::Path;

use clap::Parser;

use cli_parser::CliParser;
use processor::StorageOptions;


fn main() {
 
    let args = CliParser::parse();

    let main_path = Path::new(&args.input_file).canonicalize().unwrap_or_else(
        |err| error::io_error(&args.input_file, &err, format!("Failed to canonicalize path \"{}\"", args.input_file.display()).as_str())
    );

    if let Some(extension) = main_path.extension() {
        if extension != "bc" {
            error::warn("The input file extension is not \".bc\".");
        }
    }

    let byte_code = files::load_byte_code(&args.input_file);

    let mut processor = processor::Processor::new(
        if args.max_memory_size == 0 { None } else { Some(args.max_memory_size) },
        args.quiet,
        if let Some(storage_file) = args.storage_file {
            Some(StorageOptions::new(
                storage_file,
                if args.max_storage_size == 0 { None } else { Some(args.max_storage_size) }
            ))
        } else {
            None
        }
    );

    processor.execute(&byte_code, args.mode);

}

