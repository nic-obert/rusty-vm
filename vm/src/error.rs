use std::path::Path;

use indoc::{printdoc, formatdoc};
use colored::Colorize;


pub fn io_error(path: &Path, error: &std::io::Error, hint: &str) -> ! {
    printdoc!("
        ❌ Error in file \"{}\"

        {}
        {}
        ",
        path.display(), error, hint
    );
    std::process::exit(1);
}


pub fn warn(message: &str) {
    println!("{}", formatdoc!("
        ⚠️  Warning: {}
        ",
        message
    ).bright_yellow());
}

