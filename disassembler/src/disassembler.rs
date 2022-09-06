use rust_vm_lib::assembly::{ByteCode, AssemblyCode};
use crate::disassembly_table::{DISASSEMBLY_TABLE, OPERATOR_DISASSEMBLY_TABLE, Argument};


pub fn disassemble(byte_code: ByteCode) -> AssemblyCode {

    let mut assembly = AssemblyCode::new();

    let mut i: usize = 0;
    while i < byte_code.len() {

        let start_index = i;
        let operator = byte_code[start_index];
        i += 1;

        let (base_name, sizes, args) = DISASSEMBLY_TABLE.get(operator as usize).unwrap_or_else(
            || panic!("Invalid operator: {} at byte {}", operator, start_index)
        );
        
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
                let operand_string = OPERATOR_DISASSEMBLY_TABLE.get(arg.kind as usize).unwrap_or_else(
                    || panic!("Invalid operand: {}", arg.kind as u8)
                )(&byte_code[i..i + arg.size as usize]);
                i += arg.size as usize;

                line += &format!(" {}", operand_string);
            }

        }

        assembly.push(line);
    }

    assembly
}

