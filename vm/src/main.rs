
mod processor;
mod memory;
mod video;
mod errors;
mod files;

use clap::Parser;
use std::path::PathBuf;


#[derive(Parser)]
#[clap(author, about, version)]
struct Cli {

    /// The input bytecode file to execute
    #[clap(value_parser)]
    pub input_file: PathBuf,

    /// Stack size in bytes
    #[clap(long, default_value = "1024")]
    pub stack_size: usize,

    /// Video memory size in pixels
    #[clap(long, default_value = "1024")]
    pub video_size: usize,

    /// Verbose mode
    #[clap(short, long, action)]
    pub verbose: bool,

}


fn main() {
 
    let args = Cli::parse();

    let byte_code = files::load_byte_code(&args.input_file);

    let mut processor = processor::Processor::new(args.stack_size, args.video_size);

    let error_code = processor.execute(byte_code, args.verbose);

    println!("Program exited with code {}", error_code);

}

