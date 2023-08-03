use rust_vm_lib::token::Token;


pub fn invalid_data_declaration(line_number: usize, line: &str, hint: &str) -> ! {
    println!(
        "Invalid data declaration at line {}:\n{}\n\n{}",
        line_number, line, hint
    );
    std::process::exit(1);
}


pub fn out_of_section(line_number: usize, line: &str) -> ! {
    println!(
        "Instruction or data declaration outside of a section at line {}:\n{}\n\n",
        line_number, line
    );
    std::process::exit(1);
}


pub fn invalid_section_declaration(name: &str, line_number: usize, line: &str, hint: &str) -> ! {
    println!(
        "Invalid section declaration \"{}\" at line {}:\n{}\n\n{}",
        name, line_number, line, hint
    );
    std::process::exit(1);
}


pub fn invalid_character(c: char, line_number: usize, char_index: usize, line: &str, hint: &str) -> ! {
    println!(
        "Invalid character '{}' at line {};{}:\n{}\n\n{}",
        c, line_number, char_index, line, hint
    );
    std::process::exit(1);
}


pub fn invalid_instruction_name(name: &str, line_number: usize, line: &str) -> ! {
    println!(
        "Invalid instruction name '{}' at line {}:\n{}\n\n",
        name, line_number, line
    );
    std::process::exit(1);
}


pub fn invalid_arg_number(given: usize, expected: usize, line_number: usize, line: &str, instruction: &str) -> ! {
    println!(
        "Invalid number of arguments for instruction `{}` at line {}:\n{}\n\nExpected {} arguments, got {}.",
        instruction, line_number, line, expected, given
    );
    std::process::exit(1);
}


pub fn undeclared_label(label: &str, line_number: usize, line: &str) -> ! {
    println!(
        "Undeclared label \"{}\" at line {}:\n{}",
        label, line_number, line
    );
    std::process::exit(1);
}


pub fn invalid_label_name(name: &str, line_number: usize, line: &str) -> ! {
    println!(
        "Invalid label name \"{}\" at line {}:\n{}",
        name, line_number, line
    );
    std::process::exit(1);
}


pub fn invalid_token(token: &Token, line_number: usize, line: &str, hint: &str) -> ! {
    println!(
        "Invalid token \"{:?}\" at line {}:\n{}\n\n{}",
        token, line_number, line, hint
    );
    std::process::exit(1);
}


pub fn invalid_token_argument(instruction: &str, arg: &Token, line_number: usize, line: &str) -> ! {
    println!(
        "Invalid argument \"{}\" for instruction `{}` at line {}:\n{}",
        arg, instruction, line_number, line
    );
    std::process::exit(1);
}


pub fn invalid_instruction_arguments(instruction: &str, line_number: usize, line: &str, hint: &str) -> ! {
    println!(
        "Invalid arguments for instruction `{}` at line {}:\n{}\n\n{}",
        instruction, line_number, line, hint
    );
    std::process::exit(1);
}


pub fn invalid_address(address: usize, line_number: usize, line: &str, hint: &str) -> ! {
    println!(
        "Invalid address {} at line {}:\n{}\n\n{}",
        address, line_number, line, hint
    );
    std::process::exit(1);
}


pub fn invalid_register_name(name: &str, line_number: usize, line: &str) -> ! {
    println!(
        "Invalid register name \"{}\" at line {}:\n{}",
        name, line_number, line
    );
    std::process::exit(1);
}


pub fn number_out_of_range(number: i64, line_number: usize, line: &str) -> ! {
    println!(
        "Number {} is out of range at line {}:\n{}",
        number, line_number, line
    );
    std::process::exit(1);
}


pub fn unclosed_string_literal(line_number: usize, char_index: usize, line: &str) -> ! {
    println!(
        "Unclosed string literal at line {};{}:\n{}",
        line_number, char_index, line
    );
    std::process::exit(1);
}


pub fn unclosed_char_literal(line_number: usize, char_index: usize, line: &str) -> ! {
    println!(
        "Unclosed character literal at line {};{}:\n{}",
        line_number, char_index, line
    );
    std::process::exit(1);
}

