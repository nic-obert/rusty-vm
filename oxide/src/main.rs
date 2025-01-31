#![feature(os_str_display)]
#![feature(str_from_raw_parts)]
#![feature(gen_blocks)]

mod cli_parser;
mod targets;
mod module_manager;
mod symbol_table;
mod tokenizer;
mod lang;
mod statics_stable;
mod compiler;


use clap::Parser;
use cli_parser::{CliParser, OptimizationFlags, TopLevelCommand};
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

    let mut module_manager = ModuleManager::new(args.include_paths.unwrap_or_default());

    let modules = compiler::prepare_modules(&mut module_manager, input_file);

    // Don't emit compiled binary
    if args.check {
        std::process::exit(0);
    }

}
