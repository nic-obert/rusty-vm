use std::cmp::min;
use std::path::Path;

use indoc::{printdoc, formatdoc};
use colored::Colorize;
use rust_vm_lib::ir::IRCode;

use crate::data_types::DataType;
use crate::token::{StringToken, Token, TokenKind};


/// Number of lines of source code to include before and after the highlighted line in error messages
const SOURCE_CONTEXT_RADIUS: u8 = 2;


pub fn warn(message: &str) {
    println!("{}", formatdoc!("
        ⚠️  Warning: {}
        ",
        message
    ).bright_yellow());
}


macro_rules! char_pointer {
    ($start:expr) => {
        format!("  {:>1$}^", "", $start)
    };
}


/// Print the source code context around the specified line.
fn print_source_context(source: &IRCode, line_index: usize, char_pointer: usize) {

    // Calculate the beginning of the context. Saturating subtraction is used interpret underflow as 0.
    let mut index = line_index.saturating_sub(SOURCE_CONTEXT_RADIUS as usize);
    let end_index = min(line_index + SOURCE_CONTEXT_RADIUS as usize + 1, source.len());
    
    // Print the source lines before the highlighted line.
    while index < line_index {
        println!("{}", source[line_index]);
        index += 1;
    }

    // The highlighted line.
    println!("> {}", source[line_index]);
    println!("  {:>1$}^", "", char_pointer);
    index += 1;

    // Lines after the highlighted line.
    while index < end_index {
        println!("{}", source[index]);
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

    println!("{}", hint);
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

    println!("{}", hint);
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

    println!("{}", hint);
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

    println!("{}", hint);
    std::process::exit(1);
}


pub fn invalid_escape_character(unit_path: &Path, character: char, line_number: usize, start: usize, line: &str, hint: &str) -> ! {
    printdoc!("
        ❌ Error in ir unit \"{}\"

        Invalid escape character '{}' at line {}:{}:
        {}
        {}

        {}
        ",
        unit_path.display(), character, line_number, start, line, char_pointer!(start), hint
    );
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

    println!("{}", hint);
    std::process::exit(1);
}


pub fn invalid_argument(operator: &TokenKind, arg: &Token, source: &IRCode, hint: &str) -> ! {
    printdoc!("
        ❌ Error in ir unit \"{}\"

        Invalid argument {:?} for operator {:?} at line {}:{}:

        ",
        arg.unit_path.display(), arg.value, operator, arg.token.line_number(), arg.token.column
    );

    print_source_context(source, arg.token.line_index(), arg.token.column);

    println!("{}", hint);
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

    println!("{}", hint);
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

    println!("{}", hint);
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

    println!("{}", hint);
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

    println!("{}", hint);
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

    println!("{}", hint);
    std::process::exit(1);
}

