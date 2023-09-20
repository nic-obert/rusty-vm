use clap::Parser;


#[derive(Parser)]
#[command(author, version, about)]
pub struct CliParser {

    /// The input file path to compile
    pub input_file: String,

    /// The output file path to write the byte code to
    #[arg(short = 'o')]
    pub output: Option<String>,

    /// Run the compiler in verbose mode
    #[arg(short = 'v', action)]
    pub verbose: bool,

    /// Just check the ir code without writing the byte code to a file
    #[arg(short = 'c', long = "check", action)]
    pub check: bool,
    
}

