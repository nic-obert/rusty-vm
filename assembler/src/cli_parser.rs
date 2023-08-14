use clap::Parser;


#[derive(Parser)]
#[clap(author, version, about)]
pub struct CliParser {

    /// The input file path to assemble
    #[clap(value_parser)]
    pub input_file: String,

    /// The output file path to write the byte code to
    #[clap(short = 'o')]
    pub output: Option<String>,

    /// Run the assembler in verbose mode
    #[clap(short = 'v', action)]
    pub verbose: bool,
    
}

