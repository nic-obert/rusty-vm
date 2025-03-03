mod cli_parser;
mod ui;
mod debugger;

use std::rc::Rc;

use clap::Parser;
use cli_parser::CliParser;
use debugger::Debugger;


fn main() -> Result<(), slint::PlatformError>  {

    let args = CliParser::parse();

    let debugger = Debugger::try_attach(args.shmem_id)
        .unwrap_or_else(|err| {
            eprintln!("Fatal error: {}", err);
            std::process::exit(1);
        });
    let debugger = Rc::new(debugger);

    ui::run_ui(debugger)

}
