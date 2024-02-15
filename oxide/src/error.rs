use std::cmp::min;
use std::path::Path;

use indoc::{printdoc, formatdoc};
use colored::Colorize;
use rusty_vm_lib::ir::SourceCode;

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


pub fn warn(token: &StringToken, source: &SourceCode, message: &str) {
    println!("{}", formatdoc!("
        ⚠️  Warning at line {}:{} in unit \"{}\":
        ",
        token.line_number(), token.column, token.unit_path.display()
    ).bright_yellow());

    print_source_context(source, token.line_index(), token.column);

    println!("\n{}\n", message);
}


/// Print the source code context around the specified line.
fn print_source_context(source: &SourceCode, line_index: usize, char_pointer: usize) {

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
    println!("{}{:line_number_width$} {}", ">".bright_red().bold(), index + 1, source[line_index]);
    println!(" {:line_number_width$} {:>char_pointer$}{}", "", "", "^".bright_red().bold());
    index += 1;

    // Lines after the highlighted line.
    while index < end_index {
        println!(" {:line_number_width$}  {}", index + 1, source[index]);
        index += 1;
    }
}


pub fn invalid_number(token: &StringToken, source: &SourceCode, hint: &str) -> ! {
    printdoc!("
        ❌ Error in unit \"{}\"

        Invalid number \"{}\" at line {}:{}:

        ",
        token.unit_path.display(), token.string, token.line_number(), token.column
    );

    print_source_context(source, token.line_index(), token.column);

    println!("\n{}\n", hint);
    std::process::exit(1);
}


pub fn unmatched_delimiter(delimiter: char, token: &StringToken, source: &SourceCode, hint: &str) -> ! {
    printdoc!("
        ❌ Error in unit \"{}\"

        Unmatched delimiter '{}' at line {}:{}:

        ",
        token.unit_path.display(), delimiter, token.line_number(), token.column 
    );

    print_source_context(source, token.line_index(), token.column);

    println!("\n{}\n", hint);
    std::process::exit(1);
}


pub fn invalid_char_literal(literal: &str, token: &StringToken, source: &SourceCode, hint: &str) -> ! {
    printdoc!("
        ❌ Error in unit \"{}\"

        Invalid character literal '{}' at line {}:{}:

        ",
        token.unit_path.display(), literal, token.line_number(), token.column
    );

    print_source_context(source, token.line_index(), token.column);

    println!("\n{}\n", hint);
    std::process::exit(1);
}


pub fn invalid_token(token: &StringToken, source: &SourceCode, hint: &str) -> ! {
    printdoc!("
        ❌ Error in unit \"{}\"

        Invalid token \"{}\" at line {}:{}:

        ",
        token.unit_path.display(), token.string, token.line_number(), token.column
    );

    print_source_context(source, token.line_index(), token.column);

    println!("\n{}\n", hint);
    std::process::exit(1);
}


pub fn invalid_escape_character(unit_path: &Path, character: char, start: usize, line_index: usize, source: &SourceCode, hint: &str) -> ! {
    printdoc!("
        ❌ Error in unit \"{}\"

        Invalid escape character '{}' at line {}:{}:

        ",
        unit_path.display(), character, line_index + 1, start
    );

    print_source_context(source, line_index, start);

    println!("\n{}\n", hint);
    std::process::exit(1);
}


pub fn expected_argument(operator: &Token, source: &SourceCode, hint: &str) -> ! {
    printdoc!("
        ❌ Error in unit \"{}\"

        Expected argment for operator {:?} at line {}:{}, but got none:
        
        ",
        operator.token.unit_path.display(), operator.value, operator.token.line_number(), operator.token.column
    );

    print_source_context(source, operator.token.line_index(), operator.token.column);

    println!("\n{}\n", hint);
    std::process::exit(1);
}


pub fn invalid_argument(operator: &TokenKind, arg: &Token, source: &SourceCode, hint: &str) -> ! {
    printdoc!("
        ❌ Error in unit \"{}\"

        Invalid argument {:?} for operator {:?} at line {}:{}:

        ",
        arg.token.unit_path.display(), arg.token.string, operator, arg.token.line_number(), arg.token.column
    );

    print_source_context(source, arg.token.line_index(), arg.token.column);

    println!("\n{}\n", hint);
    std::process::exit(1);
}


pub fn unexpected_token(token: &Token, source: &SourceCode, hint: &str) -> ! {
    printdoc!("
        ❌ Error in unit \"{}\"

        Unexpected token {:?} at line {}:{}:

        ",
        token.token.unit_path.display(), token.value, token.token.line_number(), token.token.column
    );

    print_source_context(source, token.token.line_index(), token.token.column);

    println!("\n{}\n", hint);
    std::process::exit(1);
}


pub fn type_error(token: &Token, expected: &[&str], got: &DataType, source: &SourceCode, hint: &str) -> ! {
    printdoc!("
        ❌ Error in unit \"{}\"

        Expected type {}, but got {} at line {}:{}:

        ",
        token.token.unit_path.display(), expected.iter().map(|dt| dt.to_string()).collect::<Vec<_>>().join(" or "), got, token.token.line_number(), token.token.column
    );

    print_source_context(source, token.token.line_index(), token.token.column);

    println!("\n{}\n", hint);
    std::process::exit(1);
}


pub fn mismatched_call_arguments(token: &Token, expected: usize, got: usize, source: &SourceCode, hint: &str) -> ! {
    printdoc!("
        ❌ Error in unit \"{}\"

        Expected {} arguments, but got {} at line {}:{}:

        ",
        token.token.unit_path.display(), expected, got, token.token.line_number(), token.token.column
    );

    print_source_context(source, token.token.line_index(), token.token.column);

    println!("\n{}\n", hint);
    std::process::exit(1);
}


pub fn symbol_undefined(token: &Token, symbol: &str, source: &SourceCode, hint: &str) -> ! {
    printdoc!("
        ❌ Error in unit \"{}\"

        Undefined symbol \"{}\" at line {}:{}:

        ",
        token.token.unit_path.display(), symbol, token.token.line_number(), token.token.column
    );

    print_source_context(source, token.token.line_index(), token.token.column);

    println!("\n{}\n", hint);
    std::process::exit(1);
}


pub fn syntax_error(token: &Token, source: &SourceCode, hint: &str) -> ! {
    printdoc!("
        ❌ Error in unit \"{}\"

        Syntax error at line {}:{}:

        ",
        token.token.unit_path.display(), token.token.line_number(), token.token.column
    );

    print_source_context(source, token.token.line_index(), token.token.column);

    println!("\n{}\n", hint);
    std::process::exit(1);
}


pub fn compile_time_operation_error(token: &Token, source: &SourceCode, hint: &str) -> ! {
    printdoc!("
        ❌ Error in unit \"{}\"

        Compiletime operation error at line {}:{}:

        ",
        token.token.unit_path.display(), token.token.line_number(), token.token.column
    );

    print_source_context(source, token.token.line_index(), token.token.column);

    println!("\n{}\n", hint);
    std::process::exit(1);
}


pub fn immutable_change(token: &Token, type_of_immutable: &DataType, source: &SourceCode, hint: &str) -> ! {
    printdoc!("
        ❌ Error in unit \"{}\"

        Attempt to change immutable value of type {} at line {}:{}:

        ",
        token.token.unit_path.display(), type_of_immutable, token.token.line_number(), token.token.column
    );

    print_source_context(source, token.token.line_index(), token.token.column);

    println!("\n{}\n", hint);
    std::process::exit(1);
}


pub fn illegal_mutable_borrow(token: &Token, source: &SourceCode, hint: &str) -> ! {
    printdoc!("
        ❌ Error in unit \"{}\"

        Illegal mutable borrow at line {}:{}:

        ",
        token.token.unit_path.display(), token.token.line_number(), token.token.column
    );

    print_source_context(source, token.token.line_index(), token.token.column);

    println!("\n{}\n", hint);
    std::process::exit(1);
}


pub fn not_a_constant(token: &Token, source: &SourceCode, hint: &str) -> ! {
    printdoc!("
        ❌ Error in unit \"{}\"
        
        Expected constant, but got non-constant expression at line {}:{}:

        ",
        token.token.unit_path.display(), token.token.line_number(), token.token.column
    );

    print_source_context(source, token.token.line_index(), token.token.column);
    
    println!("\n{}\n", hint);
    std::process::exit(1);
}


pub fn use_of_uninitialized_value(token: &Token, data_type: &DataType, source: &SourceCode, hint: &str) -> ! {
    printdoc!("
        ❌ Error in unit \"{}\"
        
        Use of uninitialized value of type {} at line {}:{}:

        ",
        token.token.unit_path.display(), data_type, token.token.line_number(), token.token.column
    );

    print_source_context(source, token.token.line_index(), token.token.column);
    
    println!("\n{}\n", hint);
    std::process::exit(1);
}


pub fn already_defined(new_def: &StringToken, old_def: &StringToken, source: &SourceCode, hint: &str) -> ! {
    printdoc!("
        ❌ Error in unit \"{}\"
        
        Cannot redefine symbol at line {}:{}:

        ",
        new_def.unit_path.display(), new_def.line_number(), new_def.column
    );

    print_source_context(source, new_def.line_index(), new_def.column);
    
    println!("\n{}\n\n", hint);

    println!("Outshadows previous definition in unit \"{}\" at line {}:{}:\n\n{}\n", old_def.unit_path.display(), old_def.line_number(), old_def.column, source[old_def.line_index()]);
    std::process::exit(1);
}


pub fn illegal_symbol_capture(token: &Token, source: &SourceCode, hint: &str) -> ! {
    printdoc!("
        ❌ Error in unit \"{}\"
        
        Illegal capture of symbol at line {}:{}:

        ",
        token.token.unit_path.display(), token.token.line_number(), token.token.column
    );

    print_source_context(source, token.token.line_index(), token.token.column);
    
    println!("\n{}\n", hint);
    std::process::exit(1);
}

