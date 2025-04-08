#![feature(new_range_api)]
#![feature(array_chunks)]
#![feature(iter_next_chunk)]
#![feature(slice_as_chunks)]
#![feature(test)]
#![feature(map_try_insert)]
mod backend;
mod ui;

use std::sync::{Arc, RwLock};

use clap::Parser;
use backend::cli_parser::CliParser;
use backend::debugger::Debugger;


fn main() -> Result<(), slint::PlatformError>  {

    let args = CliParser::parse();

    let debugger = Debugger::try_attach(args.shmem_id, args.debug_mode)
        .unwrap_or_else(|err| {
            eprintln!("Fatal error: {}", err);
            std::process::exit(1);
        });
    let debugger = Arc::new(RwLock::new(debugger));

    ui::debugger_ui::run_ui(debugger)

}
