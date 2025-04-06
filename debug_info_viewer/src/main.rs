mod cli_parser;
mod file_utils;
mod backend;

use clap::Parser;
use cli_parser::CliParser;
use rusty_vm_lib::debugger::{self, SectionParseError};


fn main() {

    let args = CliParser::parse().args();

    match args.action {
        cli_parser::Action::JustCheckHasDebug => {

            let program = file_utils::read_file(args.input_file.as_path());

            let has_debug_info = backend::has_debug_info(&program);

            if has_debug_info {
                println!("Program has debug information");
            } else {
                println!("Program doesn't have debug information");
            }
        },

        cli_parser::Action::ViewInfo { check_valid, include_source_files, include_label_names, include_labels, include_instructions } => {

            let program = file_utils::read_file(args.input_file.as_path());

            let sections_table = backend::read_debug_sections_table(&program)
                .unwrap_or_else(|err| {
                    match err {
                        SectionParseError::NoDebugInfo => println!("Program doesn't have debug information"),
                        SectionParseError::InvalidLabelNamesRange => println!("Invalid sections table: invalid label names range"),
                        SectionParseError::InvalidSourceFilesRange => println!("Invalid sections table: invalid source files range"),
                        SectionParseError::InvalidLabelsRange => println!("Invalid sections table: invalid labels range"),
                        SectionParseError::InvalidInstructionsRange => println!("Invalid sections table: invalid instructions range"),
                    }
                    std::process::exit(1);
                });

            if check_valid {
                // TODO
            }

            if include_source_files {
                println!("\nSource files section:");
                let source_files = debugger::read_source_files(&sections_table, &program);
                for file in source_files {
                    match file {
                        Ok(file) => println!("{}", file.to_string_lossy()),
                        Err(err) => println!("Invalid source files section: {}", err),
                    }
                }
            }

            if include_labels {
                println!("\nLabel names section:");
                let label_names = debugger::read_label_names(&sections_table, &program);
                for label in label_names {
                    match label {
                        Ok(name) => println!("{}", name),
                        Err(err) => println!("Invalid label names section: {}", err),
                    }
                }
            }
        },
    }

}
