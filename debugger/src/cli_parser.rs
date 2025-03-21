use clap::Parser;


#[derive(Parser)]
#[clap(author, about, version)]
pub struct CliParser {

    /// ID of the shared memory region containing the loaded program to debug and controls
    #[clap(required = true)]
    pub shmem_id: String,

    /// Launch the application in debug mode
    #[clap(short='d', long="debug")]
    pub debug_mode: bool,

}
