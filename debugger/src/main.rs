#![feature(new_range_api)]
#![feature(array_chunks)]
#![feature(iter_next_chunk)]
#![feature(slice_as_chunks)]
#![feature(test)]
#![feature(map_try_insert)]
mod cli_parser;
mod ui;
mod debugger;
mod queue_model;

use std::sync::{Arc, RwLock};

use clap::Parser;
use cli_parser::CliParser;
use debugger::Debugger;


fn main() -> Result<(), slint::PlatformError>  {

    let args = CliParser::parse();

    let debugger = Debugger::try_attach(args.shmem_id, args.debug_mode)
        .unwrap_or_else(|err| {
            eprintln!("Fatal error: {}", err);
            std::process::exit(1);
        });
    let debugger = Arc::new(RwLock::new(debugger));

    ui::run_ui(debugger)

}
