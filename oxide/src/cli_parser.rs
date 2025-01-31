use std::path::{PathBuf, Path};
use std::fmt::Display;
use std::fmt;

use clap::{Parser, Subcommand};

use crate::targets::Targets;


#[derive(Parser)]
#[command(author, version, about)]
pub struct CliParser {

    #[command(subcommand)]
    pub top_level_command: Option<TopLevelCommand>,

    /// The input file path
    pub input_file: Option<PathBuf>,

    /// The output file path to write the byte code to
    #[arg(short = 'o', requires("input_file"), )]
    pub output: Option<PathBuf>,

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

    /// The target to compile to
    #[arg(short = 't', requires("input_file"), conflicts_with("check"))]
    target: Option<String>,

    /// Additional include paths to search when looking for modules to import
    #[arg(short = 'I', requires("input_file"))]
    pub include_paths: Option<Vec<Box<Path>>>,

}


#[derive(Subcommand)]
pub enum TopLevelCommand {

    /// List the available optimizations
    ListOptimizations,
    /// List the available targets
    ListTargets

}

const TURN_ON: bool = true;
const TURN_OFF: bool = false;

impl CliParser {

    pub fn optimization_flags(&self) -> OptimizationFlags {
        let mut flags = OptimizationFlags::default();

        Self::parse_optimizations(&self.optimizations_on, TURN_ON, &mut flags);
        Self::parse_optimizations(&self.optimizations_off, TURN_OFF, &mut flags);

        flags
    }


    pub fn target(&self) -> Targets {
        self.target.as_ref().map(|target|
            Targets::from_string(target)
                .unwrap_or_else(|| {
                    println!("Unknown target: {}", target);
                    std::process::exit(1);
                })
        ).unwrap_or_default()
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
                println!("Available optimizations:\n{}",
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

        impl Display for OptimizationFlags {
            #[allow(unused_assignments)] // The compiler cannot see that first is used when the macro is expanded
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                let mut first = true;
                write!(f, "[")?;
                $(
                    if self.$opt_name {
                        if first {
                            first = false;
                        } else {
                            write!(f, ", ")?;
                        }
                        write!(f, "{}", stringify!($opt_name))?;
                    }
                )*
                write!(f, "]")
            }
        }


        impl CliParser {

            fn parse_optimizations(string_flags: &[String], on_off: bool, flags: &mut OptimizationFlags) {

                for opt in string_flags {
                    match opt.as_str() {
                        "all" => {
                            match on_off {
                                TURN_ON => *flags = OptimizationFlags::all(),
                                TURN_OFF => *flags = OptimizationFlags::none(),
                            }
                            if string_flags.len() > 1 {
                                match on_off {
                                    TURN_ON => println!("Turning on single optimizations is redundant because -Oall turns on all optimizations."),
                                    TURN_OFF => println!("Turning off single optimizations is redundant because -Xall turns off all optimizations."),
                                }
                            }
                        },
                        $(
                            stringify!($opt_name) => flags.$opt_name = on_off,
                        )*
                        _ => {
                            println!("Unknown optimization flag: {}", opt);
                            std::process::exit(1);
                        }
                    }
                }

            }

        }

    };
}


declare_optimizations!(
    evaluate_constants = true,
    remove_useless_code = false
);
