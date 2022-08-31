use crate::tokenizer::tokenize_operands;
use std::collections::HashMap;


pub type AssemblyCode = Vec<String>;
pub type ByteCode = Vec<u8>;


pub fn assemble(assembly: AssemblyCode) -> ByteCode {

    let byte_code = ByteCode::new();

    let label_map: HashMap<&str, usize> = HashMap::new();

    let mut line_number: usize = 0;
    for line in assembly {
        line_number += 1;

        let stripped_line = line.strip_prefix(' ').unwrap().strip_suffix(' ').unwrap();
        if stripped_line.is_empty() || stripped_line.starts_with(';') {
            // The line is empty or a comment
            continue;
        }

        // List containing either a single operator or an operator and its arguments
        let raw_tokens = stripped_line.split_once(' ').unwrap();
        let operator = raw_tokens.0;



    }

    byte_code
}

