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
    #[arg(short = 'O', requires("input_file"), conflicts_with("check"))]
    optimizations_on: Vec<String>,
    /// Turn off these optimizations.
    /// Use -Xall to turn off all optimizations
    #[arg(short = 'X', requires("input_file"), conflicts_with("check"))]
    optimizations_off: Vec<String>,
    
}


#[derive(Subcommand)]
pub enum TopLevelCommand {

    /// List the available optimizations
    ListOptimizations,

}


impl CliParser {

    fn parse_optimizations(string_flags: &[String], on: bool, flags: &mut OptimizationFlags) {

        for opt in string_flags {
            match opt.as_str() {
                "all" => {
                    if string_flags.len() > 1 {
                        if on {
                            println!("Turning on single optimizations is redundant because -Oall turns on all optimizations.");
                            *flags = OptimizationFlags::all();
                        } else {
                            println!("Turning off single optimizations is redundant because -Xall turns off all optimizations.");
                            *flags = OptimizationFlags::none();
                        }
                    }
                },
                "evaluate_constants" => flags.evaluate_constants = on,
                "remove_useless_code" => flags.remove_useless_code = on,
                _ => {
                    println!("Unknown optimization flag: {}", opt);
                    std::process::exit(1);
                }
            }
        }

    }
    
    pub fn optimization(&self) -> OptimizationFlags {
        let mut flags = OptimizationFlags::default();

        Self::parse_optimizations(&self.optimizations_on, true, &mut flags);
        Self::parse_optimizations(&self.optimizations_off, false, &mut flags);

        flags
    }

}


macro_rules! declare_optimizations {
    (
        $(
            $opt_name:ident = $default:literal
        ),*
    ) => {

        #[derive(Clone)]
        pub struct OptimizationFlags {
            $(pub $opt_name: bool),*
        }

        impl OptimizationFlags {

            pub fn all() -> Self {
                OptimizationFlags {
                    $($opt_name: true),*
                }
            }

            pub fn none() -> Self {
                OptimizationFlags {
                    $($opt_name: false),*
                }
            }

            pub fn list_optimizations() {
                printdoc!("
                    Available optimizations:
                    {}
                    ",
                    concat!(
                        $("- ", stringify!($opt_name), " = ", $default, "\n"),*
                    )
                );
            }

        }

        impl Default for OptimizationFlags {
            fn default() -> Self {
                OptimizationFlags {
                    $($opt_name: $default),*
                }
            }
        }

    };
}


declare_optimizations!(
    evaluate_constants = true,
    remove_useless_code = false
);

