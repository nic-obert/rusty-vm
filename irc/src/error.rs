use std::path::Path;

use indoc::{printdoc, formatdoc};
use colored::Colorize;

use crate::token::{Token, TokenKind};


pub fn warn(message: &str) {
    println!("{}", formatdoc!("
        ⚠️  Warning: {}
        ",
        message
    ).bright_yellow());
}


pub fn invalid_number(unit_path: &Path, number: &str, line_number: usize, start: usize, line: &str, hint: &str) -> ! {
    printdoc!("
        ❌ Error in ir unit \"{}\"

        Invalid number {} at line {}:{}:
        {}
        {}

        {}
        ",
        unit_path.display(), number, line_number, start, line, format!("{:>1$}^", "", start), hint
    );
    std::process::exit(1);
}


pub fn unmatched_delimiter(unit_path: &Path, delimiter: char, line_number: usize, start: usize, line: &str, hint: &str) -> ! {
    printdoc!("
        ❌ Error in ir unit \"{}\"

        Unmatched delimiter '{}' at line {}:{}:
        {}
        {}

        {}
        ",
        unit_path.display(), delimiter, line_number, start, line, format!("{:>1$}^", "", start), hint
    );
    std::process::exit(1);
}


pub fn invalid_char_literal(unit_path: &Path, literal: &str, line_number: usize, start: usize, line: &str, hint: &str) -> ! {
    printdoc!("
        ❌ Error in ir unit \"{}\"

        Invalid character literal '{}' at line {}:{}:
        {}
        {}

        {}
        ",
        unit_path.display(), literal, line_number, start, line, format!("{:>1$}^", "", start), hint
    );
    std::process::exit(1);
}


pub fn invalid_token(unit_path: &Path, token: &str, line_number: usize, start: usize, line: &str, hint: &str) -> ! {
    printdoc!("
        ❌ Error in ir unit \"{}\"

        Invalid token \"{}\" at line {}:{}:
        {}
        {}

        {}
        ",
        unit_path.display(), token, line_number, start, line, format!("{:>1$}^", "", start), hint
    );
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
        unit_path.display(), character, line_number, start, line, format!("{:>1$}^", "", start), hint
    );
    std::process::exit(1);
}


pub fn expected_argument(unit_path: &Path, operator: &TokenKind, line_number: usize, start: usize, line: &str, hint: &str) -> ! {
    printdoc!("
        ❌ Error in ir unit \"{}\"

        Expected argment for operator {:?} at line {}:{}, but got none:
        {}
        {}

        {}
        ",
        unit_path.display(), operator, line_number, start, line, format!("{:>1$}^", "", start), hint
    );
    std::process::exit(1);
}


pub fn invalid_argument(unit_path: &Path, operator: &TokenKind, line_number: usize, start: usize, line: &str, hint: &str) -> ! {
    printdoc!("
        ❌ Error in ir unit \"{}\"

        Invalid argument for operator {:?} at line {}:{}:
        {}
        {}

        {}
        ",
        unit_path.display(), operator, line_number, start, line, format!("{:>1$}^", "", start), hint
    );
    std::process::exit(1);
}


pub fn unexpected_token(unit_path: &Path, token: &Token, line_number: usize, start: usize, line: &str, hint: &str) -> ! {
    printdoc!("
        ❌ Error in ir unit \"{}\"

        Unexpected token {} at line {}:{}:
        {}
        {}

        {}
        ",
        unit_path.display(), token, line_number, start, line, format!("{:>1$}^", "", start), hint
    );
    std::process::exit(1);
}

