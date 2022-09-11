use crate::disassembly_table::Argument;



pub fn invalid_instruction_code(code: u8, byte_index: usize) -> ! {
    println!(
        "Invalid instruction code: {} at byte {}",
        code, byte_index
    );
    std::process::exit(1);
}


pub fn invalid_arguments(expected: &Argument, got: &[u8], byte_index: usize, hint: &str) -> ! {
    println!(
        "Invalid arguments for instruction at byte {}. Expected {}, got:
        {:?}
        
        {}",
        byte_index, expected, got, hint
    );
    std::process::exit(1);
}

