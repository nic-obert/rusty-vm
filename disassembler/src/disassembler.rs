use rust_vm_lib::assembly::{ByteCode, AssemblyCode};
use rust_vm_lib::byte_code::ByteCodes;
use crate::disassembly_table::{DISASSEMBLY_TABLE, OPERATOR_DISASSEMBLY_TABLE, Argument};
use crate::error;


pub fn disassemble(byte_code: ByteCode, verbose: bool) -> AssemblyCode {

    let mut assembly = AssemblyCode::new();

    let mut i: usize = 0;
    while i < byte_code.len() {

        let start_index = i;
        let operator = byte_code[start_index];
        i += 1;

        let (base_name, sizes, args) 
            = DISASSEMBLY_TABLE.get(operator as usize)
            .unwrap_or_else(
                || error::invalid_instruction_code(operator, start_index)
        );   

        if verbose {
            print!("\n{}: {} ({})", start_index, ByteCodes::from(operator), operator);
        }
        
        let mut line = base_name.to_string();

        if let Some(args) = args {
            // The operator has arguments

            // Need mutable args to update them
            let mut args: Vec<Argument> = args.to_vec();

            if let Some(sizes) = sizes {
                // The operands have a variable size
                // The first byte is the handled size for sized operands
                let handled_size = byte_code[i];
                i += 1;
                line += &handled_size.to_string();

                // Update the arguments with variable sizes
                for size in sizes {
                    args[*size as usize].size = handled_size;
                }
            }

            // Disassemble the operands
            for arg in args {
                let disassembler = match OPERATOR_DISASSEMBLY_TABLE.get(arg.kind as usize) {
                    Some(disassembler) => disassembler,
                    None => panic!("Invalid argument type: {}. This is a bug.", arg.kind)
                };
                
                let operand_string = match disassembler(&byte_code[i..i + arg.size as usize]) {
                    Ok(operand_string) => operand_string,
                    Err(message) => error::invalid_arguments(&arg, &byte_code[i..i + arg.size as usize], i, &message)
                };
                
                i += arg.size as usize;

                line += &format!(" {}", operand_string);
            }

        }

        if verbose {
            println!(" <= {:?}\n{}\n", &byte_code[start_index..i], line);
        }

        assembly.push(line);
    }

    assembly
}

