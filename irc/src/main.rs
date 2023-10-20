mod operations;
mod token;
mod data_types;
mod tokenizer;
mod cli_parser;
mod files;
mod error;
mod ast;

use std::path::Path;

use clap::Parser;

use crate::cli_parser::CliParser;


fn main() {
    
    let args = CliParser::parse();

    let input_file = Path::new(&args.input_file);

    let source = match files::load_ir_code(input_file) {
        Ok(source) => source,
        Err(e) => {
            println!("Could not open file {}\n{}", input_file.display(), e);
            std::process::exit(1);
        }
    };

    let mut tokens = tokenizer::tokenize(&source, input_file);

    for token in tokens.iter_items() {
        println!("{}", token);
    }

    // let syntax = ast::parse_syntax_tree(&mut tokens, &source);

}

