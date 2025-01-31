use std::io;
use std::cmp::min;
use std::rc::Rc;

use colored::Colorize;

use crate::compiler::CompilationPhases;
use crate::module_manager::Module;
use crate::tokenizer::SourceToken;



/// Number of lines of source code to include before and after the highlighted line in error messages
const SOURCE_CONTEXT_RADIUS: u8 = 4;


/// Print the source code context around the specified line.
fn print_source_context(source: &[&str], line_index: usize, column_index: usize) {

    let char_pointer = column_index + 2;

    // Calculate the beginning of the context. Saturating subtraction is used interpret underflow as 0.
    let mut index = line_index.saturating_sub(SOURCE_CONTEXT_RADIUS as usize);
    let end_index = min(line_index + SOURCE_CONTEXT_RADIUS as usize + 1, source.len());

    let line_number_width = end_index.to_string().len();

    // Print the source lines before the highlighted line.
    while index < line_index {
        println!(" {:line_number_width$} | {}", index + 1, source[index]);
        index += 1;
    }

    // The highlighted line.
    println!("{}{:line_number_width$} | {}", ">".bright_red().bold(), index + 1, source[line_index]);
    println!(" {:line_number_width$} {:>char_pointer$}{}", "", "", "^".bright_red().bold());
    index += 1;

    // Lines after the highlighted line.
    while index < end_index {
        println!(" {:line_number_width$} | {}", index + 1, source[index]);
        index += 1;
    }
}


pub fn io_error(error: io::Error, hint: &str) -> ! {
    println!("\nIO Error\n{}\nHint:\n{}", error, hint);

    std::process::exit(1);
}


pub fn print_errors_and_exit(phase: CompilationPhases, errors: &[CompilationError], module: &Module) -> ! {

    println!("\n{} error(s) occurred during the {} phase on module `{}`:\n", errors.len(), phase, module.path.display());

    for (i, error) in errors.iter().enumerate() {
        println!("\nError #{}", i+1);
        error.print(module.lines());
    }

    std::process::exit(0);
}


pub enum ErrorKind {

    InvalidEscapeSequence { invalid_character: Option<char> },
    UnmatchedDelimiter { delimiter: &'static str },
    EmptyCharLiteral,
    CharLiteralTooLong { length: usize },
    UnexpectedCharacter { ch: char },
}


pub struct CompilationError<'a> {

    pub source: Rc<SourceToken<'a>>,
    pub kind: ErrorKind,
    pub hint: &'a str

}

impl CompilationError<'_> {

    fn print(&self, source: &[&str]) {

        println!("Line: {}, column: {}", self.source.line_number(), self.source.column_index);

        match self.kind {
            ErrorKind::InvalidEscapeSequence { invalid_character } => println!("Invalid escape sequence{}", if let Some(invalid_char) = invalid_character { format!(" `\\{}'", invalid_char) } else { "".to_string() }),
            ErrorKind::UnmatchedDelimiter { delimiter } => println!("Unmatched delimiter `{}`", delimiter),
            ErrorKind::EmptyCharLiteral => println!("Empty character literal"),
            ErrorKind::CharLiteralTooLong { length } => println!("Characer literal too long: {} characters", length),
            ErrorKind::UnexpectedCharacter { ch } => println!("Unexpected character {}", ch),
        }
        println!("\n");

        print_source_context(source, self.source.line_index(), self.source.column_index);

        println!("Hint:\n{}", self.hint);
    }

}
