mod cli_parser;
mod targets;
mod module_manager;
mod symbol_table;
mod tokenizer;
mod lang;

use std::fs;

use clap::Parser;
use cli_parser::{CliParser, OptimizationFlags, TopLevelCommand};
use targets::Targets;



fn main() {

    let args = CliParser::parse();

    if let Some(command) = args.top_level_command {
        match command {

            TopLevelCommand::ListOptimizations => {
                OptimizationFlags::list_optimizations();
                std::process::exit(0);
            },

            TopLevelCommand::ListTargets => {
                Targets::list_targets();
                std::process::exit(0);
            }

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

    let source = match fs::read_to_string(input_file) {
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


    // Don't emit compiled binary
    if args.check {
        std::process::exit(0);
    }

}
