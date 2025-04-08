#![feature(new_range_api)]

mod cli_parser;
mod file_utils;

use clap::Parser;
use cli_parser::CliParser;

use rusty_vm_lib::byte_code::{ByteCodes, OPCODE_SIZE};
use rusty_vm_lib::debugger::{self, DebugSectionsTable, DebugSectionsTableParseError};
use rusty_vm_lib::assembly;
use rusty_vm_lib::vm::Address;


fn main() {

    let args = CliParser::parse().args();

    match args.action {
        cli_parser::Action::JustCheckHasDebug => {

            let program = file_utils::read_file(args.input_file.as_path());

            let has_debug_info = debugger::has_debug_info(&program);

            if has_debug_info {
                println!("Program has debug information");
            } else {
                println!("Program doesn't have debug information");
            }
        },

        cli_parser::Action::ViewInfo { check_valid, include_section_table, include_source_files, include_label_names, include_labels, include_instructions } => {

            let program = file_utils::read_file(args.input_file.as_path());

            let sections_table = DebugSectionsTable::try_parse(&program)
                .unwrap_or_else(|err| {
                    match err {
                        DebugSectionsTableParseError::NoDebugInfo => println!("Program doesn't have debug information"),
                        DebugSectionsTableParseError::InvalidLabelNamesRange => println!("Invalid sections table: invalid label names range"),
                        DebugSectionsTableParseError::InvalidSourceFilesRange => println!("Invalid sections table: invalid source files range"),
                        DebugSectionsTableParseError::InvalidLabelsRange => println!("Invalid sections table: invalid labels range"),
                        DebugSectionsTableParseError::InvalidInstructionsRange => println!("Invalid sections table: invalid instructions range"),
                    }
                    std::process::exit(1);
                });

            if check_valid {
                // TODO
            }

            if include_section_table {
                println!("\nDebug section table:");
                println!("Label names: {} - {}   ({:#X} - {:#X})", sections_table.label_names.start, sections_table.label_names.end, sections_table.label_names.start, sections_table.label_names.end);
                println!("Source files: {} - {}   ({:#X} - {:#X})", sections_table.source_files.start, sections_table.source_files.end, sections_table.source_files.start, sections_table.source_files.end);
                println!("Labels: {} - {}   ({:#X} - {:#X})", sections_table.labels.start, sections_table.labels.end, sections_table.labels.start, sections_table.labels.end);
                println!("Instructions: {} - {}   ({:#X} - {:#X})", sections_table.instructions.start, sections_table.instructions.end, sections_table.instructions.start, sections_table.instructions.end);
            }

            if include_label_names {
                println!("\nLabel names section:");
                let label_names = debugger::read_label_names_section(&sections_table, &program);
                for label in label_names {
                    match label {
                        Ok(name) => println!("{}", name),
                        Err(err) => println!("Invalid label names section: {}", err),
                    }
                }
            }

            if include_source_files {
                println!("\nSource files section:");
                let source_files = debugger::read_source_files_section(&sections_table, &program);
                for file in source_files {
                    match file {
                        Ok(file) => println!("{}", file.display()),
                        Err(err) => println!("Invalid source files section: {}", err),
                    }
                }
            }

            if include_labels {
                println!("\nLabels section:");
                let labels = debugger::read_labels_section(&sections_table, &program);
                for label in labels {
                    match label {
                        Ok(label) => println!("`{}` {} {:#X} {}:{}:{}", label.name, label.address, label.address, label.source_file.display(), label.source_line, label.source_column),
                        Err(err) => println!("Invalid labels section: {}", err),
                    }
                }
            }

            if include_instructions {
                println!("\nInstructions section:");
                let instructions = debugger::read_instructions_section(&sections_table, &program);
                for instruction in instructions {
                    match instruction {
                        Ok(instruction) => {
                            let disassembly = disassemble_instruction_at(&program, instruction.pc);
                            println!("{} {:#X}: {} ; {}:{}:{}", instruction.pc, instruction.pc, disassembly, instruction.source_file.display(), instruction.source_line, instruction.source_column);
                        },
                        Err(err) => println!("Invalid instructions section: {}", err)
                    }
                }
            }
        },
    }

}


fn disassemble_instruction_at(program: &[u8], pc: Address) -> String {

    let operator = ByteCodes::from(program[pc]);

    let (handled_size, args) = assembly::parse_bytecode_args(operator, &program[pc+OPCODE_SIZE..])
        .unwrap_or_else(|err| panic!("Could not parse arguments for opcode {operator}:\n{err}"));

    let mut disassembly = if handled_size != 0 {
        format!("{operator} ({handled_size})")
    } else {
        format!("{operator}")
    };
    for arg in args {
        disassembly.push(' ');
        disassembly.push_str(arg.to_string().as_str());
    }

    disassembly
}
