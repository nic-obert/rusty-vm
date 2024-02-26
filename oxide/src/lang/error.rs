use std::cmp::min;
use std::path::Path;

use indoc::{printdoc, formatdoc};
use colored::Colorize;
use rusty_vm_lib::ir::SourceCode;

use crate::lang::data_types::DataType;
use crate::tokenizer::{SourceToken, Token, TokenKind};


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


pub fn warn(token: &SourceToken, source: &SourceCode, message: &str) {
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
    println!("{}{:line_number_width$}  {}", ">".bright_red().bold(), index + 1, source[line_index]);
    println!(" {:line_number_width$} {:>char_pointer$}{}", "", "", "^".bright_red().bold());
    index += 1;

    // Lines after the highlighted line.
    while index < end_index {
        println!(" {:line_number_width$}  {}", index + 1, source[index]);
        index += 1;
    }
}


pub fn invalid_number(token: &SourceToken, source: &SourceCode, hint: &str) -> ! {
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


pub fn unmatched_delimiter(delimiter: char, token: &SourceToken, source: &SourceCode, hint: &str) -> ! {
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


pub fn invalid_char_literal(literal: &str, token: &SourceToken, source: &SourceCode, hint: &str) -> ! {
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


pub fn invalid_token(token: &SourceToken, source: &SourceCode, hint: &str) -> ! {
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
        operator.source_token.unit_path.display(), operator.value, operator.source_token.line_number(), operator.source_token.column
    );

    print_source_context(source, operator.source_token.line_index(), operator.source_token.column);

    println!("\n{}\n", hint);
    std::process::exit(1);
}


pub fn invalid_argument(operator: &TokenKind, arg: &SourceToken, source: &SourceCode, hint: &str) -> ! {
    printdoc!("
        ❌ Error in unit \"{}\"

        Invalid argument {:?} for operator {:?} at line {}:{}:

        ",
        arg.unit_path.display(), arg.string, operator, arg.line_number(), arg.column
    );

    print_source_context(source, arg.line_index(), arg.column);

    println!("\n{}\n", hint);
    std::process::exit(1);
}


pub fn unexpected_token(token: &Token, source: &SourceCode, hint: &str) -> ! {
    printdoc!("
        ❌ Error in unit \"{}\"

        Unexpected token {:?} at line {}:{}:

        ",
        token.source_token.unit_path.display(), token.value, token.source_token.line_number(), token.source_token.column
    );

    print_source_context(source, token.source_token.line_index(), token.source_token.column);

    println!("\n{}\n", hint);
    std::process::exit(1);
}


pub fn type_error(token: &SourceToken, expected: &[&str], got: &DataType, source: &SourceCode, hint: &str) -> ! {
    printdoc!("
        ❌ Error in unit \"{}\"

        Expected type {}, but got {} at line {}:{}:

        ",
        token.unit_path.display(), expected.iter().map(|dt| dt.to_string()).collect::<Vec<_>>().join(" or "), got, token.line_number(), token.column
    );

    print_source_context(source, token.line_index(), token.column);

    println!("\n{}\n", hint);
    std::process::exit(1);
}


pub fn mismatched_call_arguments(token: &SourceToken, expected: usize, got: usize, source: &SourceCode, hint: &str) -> ! {
    printdoc!("
        ❌ Error in unit \"{}\"

        Expected {} arguments, but got {} at line {}:{}:

        ",
        token.unit_path.display(), expected, got, token.line_number(), token.column
    );

    print_source_context(source, token.line_index(), token.column);

    println!("\n{}\n", hint);
    std::process::exit(1);
}


pub fn symbol_undefined(token: &SourceToken, symbol: &str, source: &SourceCode, hint: &str) -> ! {
    printdoc!("
        ❌ Error in unit \"{}\"

        Undefined symbol \"{}\" at line {}:{}:

        ",
        token.unit_path.display(), symbol, token.line_number(), token.column
    );

    print_source_context(source, token.line_index(), token.column);

    println!("\n{}\n", hint);
    std::process::exit(1);
}


pub fn syntax_error(token: &SourceToken, source: &SourceCode, hint: &str) -> ! {
    printdoc!("
        ❌ Error in unit \"{}\"

        Syntax error at line {}:{}:

        ",
        token.unit_path.display(), token.line_number(), token.column
    );

    print_source_context(source, token.line_index(), token.column);

    println!("\n{}\n", hint);
    std::process::exit(1);
}


pub fn compile_time_operation_error(token: &SourceToken, source: &SourceCode, hint: &str) -> ! {
    printdoc!("
        ❌ Error in unit \"{}\"

        Compiletime operation error at line {}:{}:

        ",
        token.unit_path.display(), token.line_number(), token.column
    );

    print_source_context(source, token.line_index(), token.column);

    println!("\n{}\n", hint);
    std::process::exit(1);
}


pub fn immutable_change(token: &SourceToken, type_of_immutable: &DataType, source: &SourceCode, hint: &str) -> ! {
    printdoc!("
        ❌ Error in unit \"{}\"

        Attempt to change immutable value of type {} at line {}:{}:

        ",
        token.unit_path.display(), type_of_immutable, token.line_number(), token.column
    );

    print_source_context(source, token.line_index(), token.column);

    println!("\n{}\n", hint);
    std::process::exit(1);
}


pub fn illegal_mutable_borrow(token: &SourceToken, source: &SourceCode, hint: &str) -> ! {
    printdoc!("
        ❌ Error in unit \"{}\"

        Illegal mutable borrow at line {}:{}:

        ",
        token.unit_path.display(), token.line_number(), token.column
    );

    print_source_context(source, token.line_index(), token.column);

    println!("\n{}\n", hint);
    std::process::exit(1);
}


pub fn not_a_constant(token: &SourceToken, source: &SourceCode, hint: &str) -> ! {
    printdoc!("
        ❌ Error in unit \"{}\"
        
        Expected constant, but got non-constant expression at line {}:{}:

        ",
        token.unit_path.display(), token.line_number(), token.column
    );

    print_source_context(source, token.line_index(), token.column);
    
    println!("\n{}\n", hint);
    std::process::exit(1);
}


pub fn use_of_uninitialized_value(token: &SourceToken, data_type: &DataType, source: &SourceCode, hint: &str) -> ! {
    printdoc!("
        ❌ Error in unit \"{}\"
        
        Use of uninitialized value of type {} at line {}:{}:

        ",
        token.unit_path.display(), data_type, token.line_number(), token.column
    );

    print_source_context(source, token.line_index(), token.column);
    
    println!("\n{}\n", hint);
    std::process::exit(1);
}


pub fn already_defined(new_def: &SourceToken, old_def: &SourceToken, source: &SourceCode, hint: &str) -> ! {
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


pub fn illegal_symbol_capture(token: &SourceToken, source: &SourceCode, hint: &str) -> ! {
    printdoc!("
        ❌ Error in unit \"{}\"
        
        Illegal capture of symbol at line {}:{}:

        ",
        token.unit_path.display(), token.line_number(), token.column
    );

    print_source_context(source, token.line_index(), token.column);
    
    println!("\n{}\n", hint);
    std::process::exit(1);
}

