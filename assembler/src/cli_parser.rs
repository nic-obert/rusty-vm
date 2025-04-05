use std::path::PathBuf;

use clap::Parser;


#[derive(Parser)]
#[clap(author, version, about)]
pub struct CliParser {

    /// The input file path to assemble
    #[clap(value_parser)]
    pub input_file: PathBuf,

    /// The output file path to write the byte code to
    #[clap(short = 'o')]
    pub output: Option<PathBuf>,

    /// Run the assembler in verbose mode
    #[clap(short = 'v', action)]
    pub verbose: bool,

    /// Just check the assembly without writing the byte code to a file
    #[clap(short = 'c', long = "check", action)]
    pub check: bool,

    /// List of paths to search for included libraries
    #[clap(short='L', value_delimiter=',')]
    pub include_paths: Vec<PathBuf>,

    /// Include debugging information in the assembled binary
    #[clap(short='d')]
    pub include_debug_info: bool,

}
