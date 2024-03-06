mod lang;
mod tokenizer;
mod cli_parser;
mod files;
mod ast;
mod symbol_table;
mod utils;
mod irc;
mod function_parser;
mod flow_analyzer;
mod open_linked_list;

use clap::Parser;
use cli_parser::{OptimizationFlags, TopLevelCommand};

use crate::cli_parser::CliParser;


fn main() {
    
    let args = CliParser::parse();

    match args.top_level_command {
        Some(TopLevelCommand::ListOptimizations) => {
            OptimizationFlags::list_optimizations();
            std::process::exit(0);
        },
        None => {
            // No command, so we are compiling by default
        }
    }

    let input_file = match &args.input_file {
        Some(file) => file,
        None => {
            println!("No input file specified");
            std::process::exit(1);
        }
    };

    let optimization_flags = args.optimization();

    let source = match files::load_ir_code(input_file) {
        Ok(source) => source,
        Err(e) => {
            println!("Could not open file {}\n{}", input_file.display(), e);
            std::process::exit(1);
        }
    };

    if args.verbose {
        println!("Read source code from {}", input_file.display());
        println!("Optimization flags: {}", optimization_flags);
    }

    let mut symbol_table = symbol_table::SymbolTable::new();

    let tokens = tokenizer::tokenize(&source, input_file, &mut symbol_table);

    let ast = ast::build_ast(tokens, &source, &mut symbol_table, args.verbose);

    let functions = function_parser::parse_functions(ast, &optimization_flags, &mut symbol_table, &source, args.verbose);

    let ir_code = irc::generate(functions, &mut symbol_table, &optimization_flags, args.verbose, &source);

    flow_analyzer::flow_graph(ir_code);

}

