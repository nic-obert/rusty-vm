mod cli_parser;
mod file_utils;
mod backend;

use clap::Parser;
use cli_parser::CliParser;


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
            todo!()
        },
    }

}
