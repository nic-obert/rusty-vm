#![feature(io_error_more)]
#![feature(os_str_display)]
#![feature(cell_leak)]

#![deny(unused_must_use)]

mod assembler;
mod tokenizer;
mod error;
mod cli_parser;
mod module_manager;
mod symbol_table;
mod lang;
mod parser;
mod generator;

use std::path::Path;
use std::fs;
use std::env;

use clap::Parser;

use crate::cli_parser::CliParser;


fn main() {

    let args = CliParser::parse();

    let main_path = Path::new(&args.input_file).canonicalize().unwrap_or_else(
        |err| error::io_error(err, None, format!("Failed to canonicalize path \"{}\"", args.input_file.display()).as_str())
    );

    if let Some(extension) = main_path.extension() {
        if extension != "asm" {
            error::warn("The input file extension is not \".asm\".");
        }
    }

    let cwd = env::current_dir()
        .unwrap_or_else( |err| error::io_error(err, None, "Failed to resolve current directory path."));

    let byte_code = assembler::assemble_all(&cwd, &args.input_file, args.include_paths);
 
    let output_name = args.output
        .unwrap_or_else(|| args.input_file.with_extension("out") );

    if let Err(err) = fs::write(output_name, byte_code) {
        error::io_error(err, None, "Could not save byte code file.");
    }

}

