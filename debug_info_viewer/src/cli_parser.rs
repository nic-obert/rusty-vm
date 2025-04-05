use std::path::PathBuf;

use clap::Parser;


#[derive(Parser)]
#[clap(author, version, about)]
pub struct CliParser {

    /// The input binary file path to view debug information from
    #[clap(value_parser)]
    input_file: PathBuf,

    /// Just check if the binary file has debug information
    #[clap(short = 'c', long = "has-debug")]
    has_debug: bool,

    /// Check if the binary file has valid debug information
    #[clap(long = "check-valid")]
    check_valid: bool,

    /// Include the source files table in the output
    #[clap(short = 's', long = "sources", conflicts_with="has_debug")]
    include_source_files: bool,

    /// Include the label names table in the output
    #[clap(short = 'n', long = "names", conflicts_with="has_debug")]
    include_label_names: bool,

    /// Include the label table in the output
    #[clap(short = 'l', long = "labels", conflicts_with="has_debug")]
    include_labels: bool,

    /// Include the instructions table in the output
    #[clap(short = 'i', long = "instructions", conflicts_with="has_debug")]
    include_instructions: bool

}

impl CliParser {

    pub fn args(self) -> CliArgs {
        CliArgs {
            input_file: self.input_file,
            action: {
                if self.has_debug {
                    Action::JustCheckHasDebug
                } else if !self.check_valid && !self.include_source_files && !self.include_label_names && !self.include_labels && !self.include_instructions {
                    // No specified action means everything is included
                    Action::ViewInfo {
                        check_valid: true,
                        include_source_files: true,
                        include_label_names: true,
                        include_labels: true,
                        include_instructions: true
                    }
                } else {
                    Action::ViewInfo {
                        check_valid: self.check_valid,
                        include_source_files: self.include_source_files,
                        include_label_names: self.include_label_names,
                        include_labels: self.include_labels,
                        include_instructions: self.include_instructions
                    }
                }
            }
        }
    }

}


pub struct CliArgs {

    pub input_file: PathBuf,
    pub action: Action

}


pub enum Action {
    JustCheckHasDebug,
    ViewInfo {
        check_valid: bool,
        include_source_files: bool,
        include_label_names: bool,
        include_labels: bool,
        include_instructions: bool
    }
}
