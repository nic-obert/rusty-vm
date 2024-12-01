#![feature(os_str_display)]

mod cli_parser;
mod targets;
mod module_manager;
mod symbol_table;
mod tokenizer;
mod lang;

use std::fs;

use clap::Parser;
use cli_parser::{CliParser, OptimizationFlags, TopLevelCommand};
use lang::errors::io_error;
use targets::Targets;
use module_manager::ModuleManager;



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

    let optimization_flags = args.optimization_flags();

    let module_manager = ModuleManager::new(args.include_paths.unwrap_or_default());

    let parent_dir = std::env::current_dir()
        .unwrap_or_else(|err| io_error(err, "Could not get current directory"))
        .parent();

    let module = module_manager.load_module(parent_dir, module_name)
        .unwrap_or_else(|err| io_error(err, "Could not load source file"));

    // Don't emit compiled binary
    if args.check {
        std::process::exit(0);
    }

}
