use std::path::Path;

use indoc::{printdoc, formatdoc};
use colored::Colorize;
use num::{FromPrimitive, Num};
use num::traits::ToBytes;

use rusty_vm_lib::token::Token;

use crate::assembler::{LabelMap, MacroMap};


pub fn warn(message: &str) {
    println!("{}", formatdoc!("
        ⚠️  Warning: {}
        ",
        message
    ).bright_yellow());
}


pub fn invalid_data_declaration(unit_path: &Path, line_number: usize, line: &str, hint: &str) -> ! {
    printdoc!("
        ❌ Error in assembly unit_path \"{}\"

        Invalid data declaration at line {}:
        {}

        {}
        ",
        unit_path.display(), line_number, line, hint
    );
    std::process::exit(1);
}


pub fn invalid_macro_declaration(unit_path: &Path, macro_name: &str, line_number: usize, line: &str, hint: &str) -> ! {
    printdoc!("
        ❌ Error in assembly unit \"{}\"

        Invalid macro declaration \"{}\" at line {}:
        {}

        {}
        ",
        unit_path.display(), macro_name, line_number, line, hint
    );
    std::process::exit(1);
}


pub fn invalid_macro_call(unit_path: &Path, line_number: usize, line: &str, hint: &str) -> ! {
    printdoc!("
        ❌ Error in assembly unit \"{}\"

        Invalid macro call at line {}:
        {}

        {}
        ",
        unit_path.display(), line_number, line, hint
    );
    std::process::exit(1);
}


pub fn macro_redeclaration(unit_path: &Path, macro_name: &str, prev_declaration_line: usize, prev_declaration_unit: &Path, line_number: usize, line: &str) -> ! {
    printdoc!("
        ❌ Error in assembly unit \"{}\"

        Macro \"{}\" redeclaration at line {}:
        {}

        Previously declared at line {} in unit \"{}\".
        ",
        unit_path.display(), macro_name, line_number, line, prev_declaration_line, prev_declaration_unit.display()
    );
    std::process::exit(1);
}


pub fn out_of_section(unit_path: &Path, line_number: usize, line: &str) -> ! {
    printdoc!("
        ❌ Error in assembly unit \"{}\"
        
        Instruction or data declaration outside of a section at line {}:
        {}
        ",
        unit_path.display(), line_number, line
    );
    std::process::exit(1);
}


pub fn invalid_section_declaration(unit_path: &Path, name: &str, line_number: usize, line: &str, hint: &str) -> ! {
    printdoc!("
        ❌ Error in assembly unit \"{}\"

        Invalid section declaration \"{}\" at line {}:
        {}

        {}
        ",
        unit_path.display(), name, line_number, line, hint
    );
    std::process::exit(1);
}


pub fn label_redeclaration(unit_path: &Path, label: &str, line_number: usize, line: &str) -> ! {
    printdoc!("
        ❌ Error in assembly unit \"{}\"

        Label \"{}\" redeclaration at line {}:
        {}
        ",
        unit_path.display(), label, line_number, line
    );
    std::process::exit(1);
}


pub fn invalid_character(unit_path: &Path, c: char, line_number: usize, char_index: usize, line: &str, hint: &str) -> ! {
    printdoc!("
        ❌ Error in assembly unit \"{}\"

        Invalid character '{}' at line {};{}:
        {}

        {}
        ",
        unit_path.display(), c, line_number, char_index, line, hint
    );
    std::process::exit(1);
}


pub fn invalid_instruction_name(unit_path: &Path, name: &str, line_number: usize, line: &str) -> ! {
    printdoc!("
        ❌ Error in assembly unit \"{}\"

        Invalid instruction name '{}' at line {}:
        {}
        ",
        unit_path.display(), name, line_number, line
    );
    std::process::exit(1);
}


pub fn invalid_arg_number(unit_path: &Path, given: usize, expected: usize, line_number: usize, line: &str, instruction: &str) -> ! {
    printdoc!("
        ❌ Error in assembly unit \"{}\"

        Invalid number of arguments for instruction `{}` at line {}:
        {}

        Expected {} arguments, got {}.
        ",
        unit_path.display(), instruction, line_number, line, expected, given
    );
    std::process::exit(1);
}


pub fn unclosed_macro_definition(unit_path: &Path, macro_name: &str, line_number: usize, line: &str) -> ! {
    printdoc!("
        ❌ Error in assembly unit \"{}\"

        Unclosed macro definition \"{}\" at line {}:
        {}
        ",
        unit_path.display(), macro_name, line_number, line
    );
    std::process::exit(1);
}


pub fn undeclared_label(unit_path: &Path, label: &str, local_labels: &LabelMap, line_number: usize, line: &str) -> ! {
    printdoc!("
        ❌ Error in assembly unit \"{}\"

        Undeclared label \"{}\" at line {}:
        {}

        Available labels are:
        {}
        ",
        unit_path.display(), label, line_number, line,
        // Cloning is fine since this is the program exit point
        local_labels.keys().cloned().collect::<Vec<String>>().join("\n")
    );
    std::process::exit(1);
}


pub fn invalid_bss_declaration(unit_path: &Path, line_number: usize, line: &str, hint: &str) -> ! {
    printdoc!("
        ❌ Error in assembly unit \"{}\"

        Invalid BSS declaration at line {}:
        {}

        {}
        ",
        unit_path.display(), line_number, line, hint
    );
    std::process::exit(1);
}


pub fn undeclared_macro(unit_path: &Path, macro_name: &str, local_macros: &MacroMap, line_number: usize, line: &str) -> ! {
    printdoc!("
        ❌ Error in assembly unit \"{}\"

        Undeclared macro \"{}\" at line {}:
        {}

        Available macros are:
        {}
        ",
        unit_path.display(), macro_name, line_number, line,
        local_macros.iter().map(
            |(name, def)| format!("{} in {}", name, def.unit_path.display())
        ).collect::<Vec<String>>().join("\n")
    );
    std::process::exit(1);
}


pub fn invalid_label_name(unit_path: &Path, name: &str, line_number: usize, line: &str, hint: &str) -> ! {
    printdoc!("
        ❌ Error in assembly unit \"{}\"

        Invalid label name \"{}\" at line {}:
        {}

        {}
        ",
        unit_path.display(), name, line_number, line, hint
    );
    std::process::exit(1);
}


pub fn invalid_token_argument(unit_path: &Path, instruction: &str, arg: &Token, line_number: usize, line: &str, possible_arguments: &[String]) -> ! {
    printdoc!("
        ❌ Error in assembly unit \"{}\"

        Invalid argument \"{}\" for instruction `{}` at line {}:
        {}

        Possible arguments for instruction {} are:
        {}
        ",
        unit_path.display(), arg, instruction, line_number, line, instruction,
        possible_arguments.iter().map(
            |args| format!("{} {}", instruction, args)
        ).collect::<Vec<String>>().join("\n")
    );
    
    std::process::exit(1);
}


pub fn number_out_of_range<N>(unit_path: &Path, number: &str, radix: u32, size_bytes: u8, line_number: usize, line: &str) -> ! 
    where N: Num + ToBytes + FromPrimitive
{
    printdoc!("
        ❌ Error in assembly unit \"{}\"

        Number {} is out of range at line {}:
        {}

        The number must fit in {} bytes.
        Bytes: {:?}
        ",
        unit_path.display(), number, line_number, line, size_bytes,
        N::from_str_radix(number, radix).map(|n| n.to_le_bytes()).unwrap_or(N::from_f32(f32::NAN).unwrap().to_le_bytes())
    );
    std::process::exit(1);
}


pub fn invalid_float_number(unit_path: &Path, number: &str, line_number: usize, line: &str, hint: &str) -> ! {
    printdoc!("
        ❌ Error in assembly unit \"{}\"

        Invalid float number {} at line {}:
        {}

        {}
        ",
        unit_path.display(), number, line_number, line, hint
    );
    std::process::exit(1);
}


pub fn unclosed_string_literal(unit_path: &Path, line_number: usize, char_index: usize, line: &str) -> ! {
    printdoc!("
        ❌ Error in assembly unit \"{}\"

        Unclosed string literal at line {};{}:
        {}
        ",
        unit_path.display(), line_number, char_index, line
    );
    std::process::exit(1);
}


pub fn unclosed_char_literal(unit_path: &Path, line_number: usize, char_index: usize, line: &str) -> ! {
    printdoc!("
        ❌ Error in assembly unit \"{}\"

        Unclosed character literal at line {};{}:
        {}
        ",
        unit_path.display(), line_number, char_index, line
    );
    std::process::exit(1);
}


pub fn io_error(unit_path: &Path, error: &std::io::Error, hint: &str) -> ! {
    printdoc!("
        ❌ Error in assembly unit \"{}\"

        IO error: {}

        {}
        ",
        unit_path.display(), error, hint
    );
    std::process::exit(1);
}


pub fn include_error(unit_path: &Path, error: &std::io::Error, file_path: &str, line_number: usize, line: &str) -> ! {
    printdoc!("
        ❌ Error in assembly unit \"{}\"

        Failed to include file \"{}\" at line {}:
        {}

        {}
        ",
        unit_path.display(), file_path, line_number, line, error
    );
    std::process::exit(1);
}


pub fn invalid_address_identifier(unit_path: &Path, name: &str, line_number: usize, line: &str) -> ! {
    printdoc!("
        ❌ Error in assembly unit \"{}\"

        Invalid address identifier \"{}\" at line {}:
        {}
        ",
        unit_path.display(), name, line_number, line
    );
    std::process::exit(1);
}

