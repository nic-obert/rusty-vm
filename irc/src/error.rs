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
        ",
        unit_path.display(), number, line_number, start, line, hint
    );
    std::process::exit(1);
}

