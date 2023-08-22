use std::path::PathBuf;

use clap::{ValueEnum, Parser};


#[derive(Parser)]
#[clap(author, about, version)]
pub struct CliParser {

    /// The input bytecode file to execute
    #[clap(value_parser)]
    pub input_file: PathBuf,

    /// Maximum memory size in bytes. Set to 0 for unlimited memory (not recommended).
    #[clap(long = "max-mem", default_value="1000000")]
    pub max_memory_size: usize,

    /// Execution mode. n = normal, v = verbose, i = interactive
    #[arg(value_enum)]
    #[clap(short = 'm', long, default_value="n")]
    pub mode: ExecutionMode,

    /// Don't print any message when exiting
    #[clap(short = 'q', long, action)]
    pub quiet: bool,

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

