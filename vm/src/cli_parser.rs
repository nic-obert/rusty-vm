use std::path::PathBuf;

use clap::{ValueEnum, Parser};


#[derive(Parser)]
#[clap(author, about, version)]
pub struct CliParser {

    /// The input bytecode file to execute
    #[clap(value_parser)]
    pub input_file: PathBuf,

    /// Stack size in bytes
    #[clap(long, default_value = "1024")]
    pub stack_size: usize,

    /// Execution mode
    #[arg(value_enum)]
    #[clap(short, long, default_value="n")]
    pub mode: ExecutionMode,

}


/// How the VM should execute the bytecode
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum ExecutionMode {
    Normal,
    Verbose,
    Interactive,
}


impl ValueEnum for ExecutionMode {

    fn from_str(input: &str, _ignore_case: bool) -> Result<Self, String> {
        match input {
            "n" => Ok(ExecutionMode::Normal),

            "v" => Ok(ExecutionMode::Verbose),

            "i" => Ok(ExecutionMode::Interactive),

            _ => Err(format!("Invalid execution mode: {}", input)),
        }
    }


    fn value_variants<'a>() -> &'a [Self] {
        &[
            ExecutionMode::Normal,
            ExecutionMode::Verbose,
            ExecutionMode::Interactive,
        ]
    }

    
    fn to_possible_value(&self) -> Option<clap::builder::PossibleValue> {
        match self {
            ExecutionMode::Normal => Some(clap::builder::PossibleValue::new("n")),
            ExecutionMode::Verbose => Some(clap::builder::PossibleValue::new("v")),
            ExecutionMode::Interactive => Some(clap::builder::PossibleValue::new("i")),
        }
    }
    
}

