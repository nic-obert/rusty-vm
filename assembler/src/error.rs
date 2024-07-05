use std::cmp::min;

use indoc::{printdoc, formatdoc};
use colored::Colorize;

use crate::module_manager::{ModuleManager, UnitPath};
use crate::tokenizer::{SourceCode, SourceToken};
use crate::lang::ENTRY_SECTION_NAME;


pub fn print_source_context(source: SourceCode, line_index: usize, char_pointer: usize) {

    /// Number of lines of source code to include before and after the highlighted line in error messages
    const SOURCE_CONTEXT_RADIUS: u8 = 5;

    // Calculate the beginning of the context. Saturating subtraction is used interpret underflow as 0.
    let mut index = line_index.saturating_sub(SOURCE_CONTEXT_RADIUS as usize);
    let end_index = min(line_index + SOURCE_CONTEXT_RADIUS as usize + 1, source.len());

    let line_number_width = end_index.to_string().len();

    // Print the source lines before the highlighted line.
    while index < line_index {
        println!(" {:line_number_width$}  {}", index + 1, source[index]);
        index += 1;
    }

    // The highlighted line.
    println!("{}{:line_number_width$}  {}", ">".bright_red().bold(), index + 1, source[line_index]);
    println!(" {:line_number_width$} {:>char_pointer$}{}", "", "", "^".bright_red().bold());
    index += 1;

    // Lines after the highlighted line.
    while index < end_index {
        println!(" {:line_number_width$}  {}", index + 1, source[index]);
        index += 1;
    }
}



pub fn warn(message: &str) {
    println!("{}", formatdoc!("
        ⚠️  Warning: {}
        ",
        message
    ).bright_yellow());
}


pub fn invalid_escape_sequence(token: &SourceToken, sequence: char, char_at: usize, source: SourceCode) -> ! {
    printdoc!("
        ❌ Error in assembly unit_path \"{}\"

        Invalid escape sequence `{sequence}` at line {}:{}:
        ",
        token.unit_path, token.line_number(), token.column
    );

    print_source_context(source, token.line_index, char_at);

    std::process::exit(1);   
}


pub fn parsing_error<'a>(token: &SourceToken<'a>, module_manager: &ModuleManager<'a>, message: &str) -> ! {
    eprintln!("Assembly unit \"{}\"", token.unit_path);
    eprintln!("Parsing error at {}:{} on token `{}`:\n{}", token.line_number(), token.column, token.string, message);

    print_source_context(module_manager.get_unit(token.unit_path).lines(), token.line_index, token.column);

    std::process::exit(1);
}


pub fn invalid_number_format(token: &SourceToken, source: SourceCode, hint: &str) -> ! {
    printdoc!("
        ❌ Error in assembly unit \"{}\"

        Invalid number `{}` at line {}:{}:
        {hint}
        ",
        token.unit_path, token.string, token.line_number(), token.column
    );

    print_source_context(source, token.line_index, token.column);

    std::process::exit(1);
}


pub fn invalid_number_size<'a>(token: &SourceToken<'a>, module_manager: &ModuleManager<'a>, actual_size: usize, expected_size: usize) {
    printdoc!("
        ❌ Error in assembly unit \"{}\"

        Invalid number `{}` at line {}:{}:
        The number is expected to have a size of {expected_size} bytes, but the least amount of bytes needed to correctly represent it is {actual_size}.
        
        ",
        token.unit_path, token.string, token.line_number(), token.column
    );

    print_source_context(module_manager.get_unit(token.unit_path).lines(), token.line_index, token.column);

    std::process::exit(1);
}


pub fn symbol_redeclaration(old_def: &SourceToken, new_def: &SourceToken, module_manager: &ModuleManager, message: &str) -> ! {
    printdoc!("
        ❌ Error in assembly unit \"{}\"

        Redeclaration of symbol '{}` at line {}:{}:

        New declaration in assembly unit \"{}\":

    ",
        new_def.unit_path, new_def.string, new_def.line_number(), new_def.column, new_def.unit_path
    );

    print_source_context(module_manager.get_unit(new_def.unit_path).lines(), new_def.line_index, new_def.column);

    println!("\nOld declaration in assembly unit \"{}\":\n", old_def.unit_path);

    print_source_context(module_manager.get_unit(old_def.unit_path).lines(), old_def.line_index, old_def.column);

    println!("\n\n{message}\n");

    std::process::exit(1);
}


pub fn tokenizer_error(token: &SourceToken, source: SourceCode, message: &str) -> ! {
    printdoc!("
        ❌ Error in assembly unit \"{}\"

        Tokenization error on token `{}` at {}:{}:
        {message}
        ",
        token.unit_path, token.string, token.line_number(), token.column
    );

    print_source_context(source, token.line_index, token.column);

    std::process::exit(1);
}


pub fn unresolved_label<'a>(token: &SourceToken<'a>, module_manager: &ModuleManager<'a>) -> ! {
    printdoc!("
        ❌ Error in assembly unit \"{}\"

        Could not resolve label `{}` at {}:{}:
        
        ",
        token.unit_path, token.string, token.line_number(), token.column
    );

    print_source_context(module_manager.get_unit(token.unit_path).lines(), token.line_index, token.column);

    std::process::exit(1);
}


pub fn undefined_macro<'a>(token: &SourceToken<'a>, module_manager: &ModuleManager<'a>, available_macros: impl Iterator<Item = &'a str>) -> ! {
    printdoc!("
        ❌ Error in assembly unit \"{}\"

        Undefined macro name `{}` at {}:{}:
        
        ",
        token.unit_path, token.string, token.line_number(), token.column
    );

    print_source_context(module_manager.get_unit(token.unit_path).lines(), token.line_index, token.column);

    println!("\nAvailable inline macros are:");

    for name in available_macros {
        println!("{name}")
    }

    std::process::exit(1);
}


pub fn missing_entry_point(unit_path: UnitPath) -> ! {
    printdoc!("
        ❌ Error in assembly unit \"{}\"

        Could not find an entry point. An executable must have an entry section named \"{ENTRY_SECTION_NAME}\"
        ",
        unit_path
    );

    std::process::exit(1);
}


pub fn io_error(error: std::io::Error, hint: &str) -> ! {
    printdoc!("
        ❌ IO error:
        {}
        
        {}
        ",
        error, hint
    );

    std::process::exit(1);
}

