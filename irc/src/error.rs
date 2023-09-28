use std::path::Path;

use indoc::{printdoc, formatdoc};
use colored::Colorize;


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

