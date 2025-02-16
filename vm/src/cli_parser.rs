use std::path::PathBuf;

use clap::{ValueEnum, Parser};


#[derive(Parser)]
#[clap(author, about, version)]
pub struct CliParser {

    /// The input bytecode file to execute
    #[clap(value_parser, required = true)]
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

    /// Attach a storage file to the VM
    #[clap(short = 's', long = "storage-file", action)]
    pub storage_file: Option<PathBuf>,

    /// Maximum storage size in bytes. Set to 0 for unlimited storage (not recommended). Only works if a storage file is attached.
    #[clap(long = "max-storage", default_value="1000000", requires = "storage_file")]
    pub max_storage_size: usize,

}


/// How the VM should execute the bytecode
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum ExecutionMode {
    Normal,
    Verbose,
    Interactive,
    Debug,
}


impl ValueEnum for ExecutionMode {

    fn from_str(input: &str, _ignore_case: bool) -> Result<Self, String> {
        match input {
            "n" => Ok(ExecutionMode::Normal),

            "v" => Ok(ExecutionMode::Verbose),

            "i" => Ok(ExecutionMode::Interactive),

            "d" => Ok(ExecutionMode::Debug),

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
            ExecutionMode::Debug => Some(clap::builder::PossibleValue::new("d")),
        }
    }

}
