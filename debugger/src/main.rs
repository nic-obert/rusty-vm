mod cli_parser;
mod ui;
mod debugger;

use clap::Parser;
use cli_parser::CliParser;
use debugger::Debugger;


fn main() -> Result<(), slint::PlatformError>  {

    let args = CliParser::parse();

    if !args.debug_mode {
        let debugger = Debugger::try_attach(args.shmem_id)
            .unwrap_or_else(|err| {
                println!("Fatal error: {}", err);
                std::process::exit(1);
            });
    }


    ui::run_ui()

}
