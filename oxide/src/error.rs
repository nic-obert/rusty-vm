use std::cmp::min;
use std::path::Path;

use indoc::{printdoc, formatdoc};
use colored::Colorize;
use rust_vm_lib::ir::IRCode;

use crate::data_types::DataType;
use crate::token::{StringToken, Token, TokenKind};


/// Number of lines of source code to include before and after the highlighted line in error messages
const SOURCE_CONTEXT_RADIUS: u8 = 3;


#[must_use]
pub enum WarnResult<T> {
    Ok,
    Warning(T)
}

impl<T> WarnResult<T> {
    
    pub fn warning(&self) -> Option<&T> {
        match self {
            WarnResult::Ok => None,
            WarnResult::Warning(warning) => Some(warning)
        }
    }

}


pub fn warn(token: &Token, source: &IRCode, message: &str) {
    println!("{}", formatdoc!("
        ⚠️  Warning at line {}:{} in ir unit \"{}\":
        ",
        token.token.line_number(), token.token.column, token.unit_path.display()
    ).bright_yellow());

    print_source_context(source, token.token.line_index(), token.token.column);

    println!("\n{}\n", message);
}


/// Print the source code context around the specified line.
fn print_source_context(source: &IRCode, line_index: usize, char_pointer: usize) {

    // Calculate the beginning of the context. Saturating subtraction is used interpret underflow as 0.
    let mut index = line_index.saturating_sub(SOURCE_CONTEXT_RADIUS as usize);
    let end_index = min(line_index + SOURCE_CONTEXT_RADIUS as usize + 1, source.len());
    
    // Print the source lines before the highlighted line.
    while index < line_index {
        println!("  {}", source[index]);
        index += 1;
    }

    // The highlighted line.
    println!("{} {}", ">".bright_red().bold(), source[line_index]);
    println!(" {:>char_pointer$}{}", "", "^".bright_red().bold());
    index += 1;

    // Lines after the highlighted line.
    while index < end_index {
        println!("  {}", source[index]);
        index += 1;
    }
}


pub fn invalid_number(unit_path: &Path, number: &str, token: &StringToken, source: &IRCode, hint: &str) -> ! {
    printdoc!("
        ❌ Error in ir unit \"{}\"

        Invalid number {} at line {}:{}:

        ",
        unit_path.display(), number, token.line_number(), token.column
    );

    print_source_context(source, token.line_index(), token.column);

    println!("\n{}\n", hint);
    std::process::exit(1);
}


pub fn unmatched_delimiter(unit_path: &Path, delimiter: char, token: &StringToken, source: &IRCode, hint: &str) -> ! {
    printdoc!("
        ❌ Error in ir unit \"{}\"

        Unmatched delimiter '{}' at line {}:{}:

        ",
        unit_path.display(), delimiter, token.line_number(), token.column 
    );

    print_source_context(source, token.line_index(), token.column);

    println!("\n{}\n", hint);
    std::process::exit(1);
}


pub fn invalid_char_literal(unit_path: &Path, literal: &str, token: &StringToken, source: &IRCode, hint: &str) -> ! {
    printdoc!("
        ❌ Error in ir unit \"{}\"

        Invalid character literal '{}' at line {}:{}:

        ",
        unit_path.display(), literal, token.line_number(), token.column
    );

    print_source_context(source, token.line_index(), token.column);

    println!("\n{}\n", hint);
    std::process::exit(1);
}


pub fn invalid_token(unit_path: &Path, token: &StringToken, source: &IRCode, hint: &str) -> ! {
    printdoc!("
        ❌ Error in ir unit \"{}\"

        Invalid token \"{}\" at line {}:{}:

        ",
        unit_path.display(), token.string, token.line_number(), token.column
    );

    print_source_context(source, token.line_index(), token.column);

    println!("\n{}\n", hint);
    std::process::exit(1);
}


pub fn invalid_escape_character(unit_path: &Path, character: char, start: usize, line_index: usize, source: &IRCode, hint: &str) -> ! {
    printdoc!("
        ❌ Error in ir unit \"{}\"

        Invalid escape character '{}' at line {}:{}:

        ",
        unit_path.display(), character, line_index + 1, start
    );

    print_source_context(source, line_index, start);

    println!("\n{}\n", hint);
    std::process::exit(1);
}


pub fn expected_argument(operator: &Token, source: &IRCode, hint: &str) -> ! {
    printdoc!("
        ❌ Error in ir unit \"{}\"

        Expected argment for operator {:?} at line {}:{}, but got none:
        
        ",
        operator.unit_path.display(), operator.value, operator.token.line_number(), operator.token.column
    );

    print_source_context(source, operator.token.line_index(), operator.token.column);

    println!("\n{}\n", hint);
    std::process::exit(1);
}


pub fn invalid_argument(operator: &TokenKind, arg: &Token, source: &IRCode, hint: &str) -> ! {
    printdoc!("
        ❌ Error in ir unit \"{}\"

        Invalid argument {:?} for operator {:?} at line {}:{}:

        ",
        arg.unit_path.display(), arg.token.string, operator, arg.token.line_number(), arg.token.column
    );

    print_source_context(source, arg.token.line_index(), arg.token.column);

    println!("\n{}\n", hint);
    std::process::exit(1);
}


pub fn unexpected_token(token: &Token, source: &IRCode, hint: &str) -> ! {
    printdoc!("
        ❌ Error in ir unit \"{}\"

        Unexpected token {:?} at line {}:{}:

        ",
        token.unit_path.display(), token.value, token.token.line_number(), token.token.column
    );

    print_source_context(source, token.token.line_index(), token.token.column);

    println!("\n{}\n", hint);
    std::process::exit(1);
}


pub fn type_error(token: &Token, expected: &[&str], got: &DataType, source: &IRCode, hint: &str) -> ! {
    printdoc!("
        ❌ Error in ir unit \"{}\"

        Expected type {}, but got {} at line {}:{}:

        ",
        token.unit_path.display(), expected.iter().map(|dt| dt.to_string()).collect::<Vec<_>>().join(" or "), got, token.token.line_number(), token.token.column
    );

    print_source_context(source, token.token.line_index(), token.token.column);

    println!("\n{}\n", hint);
    std::process::exit(1);
}


pub fn mismatched_call_arguments(token: &Token, expected: usize, got: usize, source: &IRCode, hint: &str) -> ! {
    printdoc!("
        ❌ Error in ir unit \"{}\"

        Expected {} arguments, but got {} at line {}:{}:

        ",
        token.unit_path.display(), expected, got, token.token.line_number(), token.token.column
    );

    print_source_context(source, token.token.line_index(), token.token.column);

    println!("\n{}\n", hint);
    std::process::exit(1);
}


pub fn symbol_undefined(token: &Token, symbol: &str, source: &IRCode, hint: &str) -> ! {
    printdoc!("
        ❌ Error in ir unit \"{}\"

        Undefined symbol \"{}\" at line {}:{}:

        ",
        token.unit_path.display(), symbol, token.token.line_number(), token.token.column
    );

    print_source_context(source, token.token.line_index(), token.token.column);

    println!("\n{}\n", hint);
    std::process::exit(1);
}


pub fn syntax_error(token: &Token, source: &IRCode, hint: &str) -> ! {
    printdoc!("
        ❌ Error in ir unit \"{}\"

        Syntax error at line {}:{}:

        ",
        token.unit_path.display(), token.token.line_number(), token.token.column
    );

    print_source_context(source, token.token.line_index(), token.token.column);

    println!("\n{}\n", hint);
    std::process::exit(1);
}


pub fn compile_time_operation_error(token: &Token, source: &IRCode, hint: &str) -> ! {
    printdoc!("
        ❌ Error in ir unit \"{}\"

        Compiletime operation error at line {}:{}:

        ",
        token.unit_path.display(), token.token.line_number(), token.token.column
    );

    print_source_context(source, token.token.line_index(), token.token.column);

    println!("\n{}\n", hint);
    std::process::exit(1);
}


pub fn immutable_change(token: &Token, source: &IRCode, hint: &str) -> ! {
    printdoc!("
        ❌ Error in ir unit \"{}\"

        Attempt to change immutable symbol at line {}:{}:

        ",
        token.unit_path.display(), token.token.line_number(), token.token.column
    );

    print_source_context(source, token.token.line_index(), token.token.column);

    println!("\n{}\n", hint);
    std::process::exit(1);
}

