use rust_vm_lib::token::Token;



pub fn invalid_character(c: char, line_number: usize, index: usize, line: &str, hint: &str) -> ! {
    println!(
        "Invalid character '{}' at line {};{}:
        {}
        
        {}
        ", c, line_number, index, line, hint
    );
    std::process::exit(1);
}


pub fn invalid_instruction_name(name: &str, line_number: usize, line: &str) -> ! {
    println!(
        "Invalid instruction name '{}' at line {}:
        {}
        
        ", name, line_number, line
    );
    std::process::exit(1);
}


pub fn invalid_arg_number(given: usize, expected: usize, line_number: usize, line: &str, instruction: &str) -> ! {
    println!(
        "Invalid number of arguments for instruction `{}` at line {}:
        {}
        
        Expected {} arguments, got {}.
        ", instruction, line_number, line, expected, given
    );
    std::process::exit(1);
}


pub fn undeclared_label(label: &str, line_number: usize, line: &str) -> ! {
    println!(
        "Undeclared label \"{}\" at line {}:
        {}
        
        ", label, line_number, line
    );
    std::process::exit(1);
}


pub fn invalid_token(token: &Token, line_number: usize, line: &str, hint: &str) -> ! {
    println!(
        "Invalid token \"{:?}\" at line {}:
        {}
        
        {}
        ", token, line_number, line, hint
    );
    std::process::exit(1);
}


pub fn invalid_token_argument(instruction: &str, arg: &Token, line_number: usize, line: &str) -> ! {
    println!(
        "Invalid argument \"{:?}\" for instruction `{}` at line {}:
        {}
        
        ", arg, instruction, line_number, line
    );
    std::process::exit(1);
}


pub fn invalid_instruction_arguments(instruction: &str, line_number: usize, line: &str, hint: &str) -> ! {
    println!(
        "Invalid arguments for instruction `{}` at line {}:
        {}
        
        {}
        ", instruction, line_number, line, hint
    );
    std::process::exit(1);
}


pub fn invalid_address(address: usize, line_number: usize, line: &str, hint: &str) -> ! {
    println!(
        "Invalid address {} at line {}:
        {}
        
        {}
        ", address, line_number, line, hint
    );
    std::process::exit(1);
}


pub fn invalid_register_name(name: &str, line_number: usize, line: &str) -> ! {
    println!(
        "Invalid register name \"{}\" at line {}:
        {}
        
        ", name, line_number, line
    );
    std::process::exit(1);
}


pub fn number_out_of_range(number: i64, line_number: usize, line: &str) -> ! {
    println!(
        "Number {} is out of range at line {}:
        {}
        
        ", number, line_number, line
    );
    std::process::exit(1);
}

