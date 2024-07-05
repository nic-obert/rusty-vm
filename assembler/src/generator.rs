
use std::rc::Rc;

use rusty_vm_lib::assembly::ByteCode;
use rusty_vm_lib::vm::{Address, ADDRESS_SIZE};

use crate::error;
use crate::symbol_table::SymbolTable;
use crate::module_manager::ModuleManager;
use crate::lang::{AsmNode, AsmNodeValue, AsmValue, PseudoInstructionNode, CURRENT_POSITION_TOKEN};
use crate::tokenizer::SourceToken;


pub fn generate_bytecode<'a>(asm: Box<[AsmNode<'a>]>, symbol_table: &SymbolTable<'a>, module_manager: &ModuleManager<'a>, bytecode: &mut ByteCode) {
    
    /// A placeholder for the real address of a label. Used as a placeholder for unresolved labels.
    const LABEL_PLACEHOLDER: [u8; ADDRESS_SIZE] = (0 as Address).to_le_bytes();

    let mut unresolved_labels: Vec<(&str, usize, Rc<SourceToken>)> = Vec::new();

    macro_rules! current_pos {
        () => {
            bytecode.len()
        };
    }

    macro_rules! push_byte {
        ($bytes:expr) => {
            bytecode.push($bytes as u8)
        }
    }

    macro_rules! push_bytes {
        ($bytes:expr) => {
            bytecode.extend($bytes)
        };
    }

    macro_rules! push_sized_number {
        ($n:expr, $size:expr, $source:expr) => {{
            if $n.least_bytes_repr() > $size {
                error::invalid_number_size($source, module_manager, $n.least_bytes_repr(), $size);
            }

            push_bytes!(&$n.as_bytes()[..$size]);
        }};
    }

    for node in asm {

        match node.value {

            AsmNodeValue::Label(name)
                => symbol_table.define_label(name, current_pos!()),
            
            AsmNodeValue::Instruction(ref instruction) => {

                let instruction_code = instruction.byte_code();
                push_byte!(instruction_code);

                let handled_size = instruction.handled_size();
                if handled_size != 0 {
                    push_byte!(handled_size);
                }

                for arg in instruction.get_args() {

                    match &arg.value {

                        AsmValue::Register(reg) |
                        AsmValue::AddressInRegister(reg)
                            => push_byte!(*reg),

                        AsmValue::Number(n)
                            => push_sized_number!(n, handled_size, &arg.source),

                        AsmValue::AddressLiteral(addr)
                            => push_bytes!(addr.as_bytes()),

                        AsmValue::Label(label) | 
                        AsmValue::AddressAtLabel(label)
                        => {
                            if let Some(label) = symbol_table.get_resolved_label(label) {
                                push_bytes!(label.to_le_bytes());

                            } else if *label == CURRENT_POSITION_TOKEN {
                                println!("Pushing current position: {}", bytecode.len());
                                push_bytes!(current_pos!().to_le_bytes())

                            } else {
                                unresolved_labels.push((label, current_pos!(), Rc::clone(&arg.source)));
                                push_bytes!(LABEL_PLACEHOLDER);
                            }
                        }
                    }
                }              
            },

            AsmNodeValue::PseudoInstruction (instruction) => {

                match instruction {

                    PseudoInstructionNode::DefineNumber { size, data }
                        => push_sized_number!(data.0, size.0 as usize, &data.1),

                    PseudoInstructionNode::DefineString { data }
                        => push_bytes!(data.0.as_bytes()),
                    
                    PseudoInstructionNode::DefineBytes {  } => todo!(),

                }
            }
        }
    }

    for (label_name, location, source) in unresolved_labels {

        if let Some(resolved_label) = symbol_table.get_resolved_label(label_name) {

            bytecode[location..location+ADDRESS_SIZE]
                .copy_from_slice(&resolved_label.to_le_bytes());

        } else {
            error::unresolved_label(&source, module_manager);
        }
    }

}

