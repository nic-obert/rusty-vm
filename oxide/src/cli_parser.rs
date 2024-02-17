use std::path::PathBuf;

use clap::{Parser, Subcommand};
use indoc::printdoc;


#[derive(Parser)]
#[command(author, version, about)]
pub struct CliParser {    

    #[command(subcommand)]
    pub top_level_command: Option<TopLevelCommand>,

    /// The input file path
    pub input_file: Option<PathBuf>,

    /// The output file path to write the byte code to
    #[arg(short = 'o', requires("input_file"), )]
    pub output: Option<String>,

    /// Run the compiler in verbose mode
    #[arg(short = 'v', requires("input_file"))]
    pub verbose: bool,

    /// Check the source code without emitting code
    #[arg(short = 'c', requires("input_file"), conflicts_with("output"))]
    pub check: bool,

    /// Turn on these optimizations.
    /// Use -Oall to turn on all optimizations
    #[arg(short = 'O', requires("input_file"), group = "optimizations", conflicts_with("check"))]
    optimizations_on: Vec<String>,
    /// Turn off these optimizations.
    /// Use -Xall to turn off all optimizations
    #[arg(short = 'X', requires("input_file"), group = "optimizations", conflicts_with("check"))]
    optimizations_off: Vec<String>,
    
}


#[derive(Subcommand)]
pub enum TopLevelCommand {

    /// List the available optimizations
    ListOptimizations,

}


impl CliParser {
    
    pub fn optimization(&self) -> OptimizationFlags {
        let mut flags = OptimizationFlags::default();

        for opt in &self.optimizations_on {
            match opt.as_str() {
                "all" => {
                    if self.optimizations_on.len() > 1 {
                        println!("Turning on single optimizations is redundant because -Oall turns on all optimizations.")
                    }
                    flags = OptimizationFlags::all()
                },
                "evaluate_constants" => flags.evaluate_constants = true,
                _ => {
                    println!("Unknown optimization flag: {}", opt);
                    std::process::exit(1);
                }
            }
        }

        for opt in &self.optimizations_off {
            match opt.as_str() {
                "all" => {
                    if self.optimizations_off.len() > 1 {
                        println!("Turning off single optimizations is redundant because -Xall turns off all optimizations.")
                    }
                    flags = OptimizationFlags::none()
                },
                "evaluate_constants" => flags.evaluate_constants = false,
                _ => {
                    println!("Unknown optimization flag: {}", opt);
                    std::process::exit(1);
                }
            }
        }

        flags
    }

}


#[derive(Clone)]
pub struct OptimizationFlags {
    pub evaluate_constants: bool,
}

impl OptimizationFlags {

    pub fn all() -> Self {
        OptimizationFlags {
            evaluate_constants: true,
        }
    }

    pub fn none() -> Self {
        OptimizationFlags {
            evaluate_constants: false,
        }
    }

    pub fn list_optimizations() {
        printdoc! {"
            Available optimizations    Default
            - evaluate_constants       true
        "}
    }

}

impl Default for OptimizationFlags {
    fn default() -> Self {
        OptimizationFlags {
            evaluate_constants: true,
        }
    }
}

