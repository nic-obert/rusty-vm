#![feature(non_null_from_ref)]
#![feature(slice_ptr_get)]

mod processor;
mod memory;
mod files;
mod cli_parser;
mod error;
mod storage;
mod terminal;
mod modules;
mod host_fs;

use clap::Parser;

use cli_parser::CliParser;
use processor::StorageOptions;


fn main() {

    let args = CliParser::parse();

    let byte_code = files::load_byte_code(&args.input_file);

    let mut processor = processor::Processor::new(
        args.max_memory_size,
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
