
use std::rc::Rc;

use rusty_vm_lib::assembly::ByteCode;
use rusty_vm_lib::vm::{Address, ADDRESS_SIZE};

use crate::error;
use crate::symbol_table::SymbolTable;
use crate::module_manager::ModuleManager;
use crate::lang::{AsmNode, AsmNodeValue, AsmValue};
use crate::tokenizer::SourceToken;


pub fn generate_bytecode<'a>(asm: Box<[AsmNode<'a>]>, symbol_table: &SymbolTable<'a>, module_manager: &ModuleManager, bytecode: &mut ByteCode) {
    
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

    for node in asm {

        match node.value {

            AsmNodeValue::Label(name)
                => symbol_table.define_label(name, current_pos!()),
            
            AsmNodeValue::Instruction(instruction) => {

                let bytecode = instruction.byte_code();
                push_byte!(bytecode);

                let handled_size = instruction.handled_size();
                if handled_size != 0 {
                    push_byte!(handled_size);
                }

                for arg in instruction.get_args() {

                    match &arg.value {

                        AsmValue::Register(reg) |
                        AsmValue::AddressInRegister(reg)
                            => push_byte!(*reg),

                        AsmValue::Number(n) => {
                            if n.least_bytes_repr() > handled_size {
                                error::invalid_number_size(&arg.source, module_manager, n.least_bytes_repr(), handled_size);
                            }
                
                            push_bytes!(&n.as_bytes()[..handled_size]);
                        },

                        AsmValue::AddressLiteral(addr) => push_bytes!(addr.as_bytes()),

                        AsmValue::AddressAtLabel(label) => {
                            if let Some(label) = symbol_table.get_resolved_label(label) {
                                push_bytes!(label.to_le_bytes());
                            } else {
                                unresolved_labels.push((label, current_pos!(), Rc::clone(&arg.source)));
                                push_bytes!(LABEL_PLACEHOLDER);
                            }
                        },

                        AsmValue::CurrentPosition(_) => push_bytes!(current_pos!().to_le_bytes()),
                        
                        AsmValue::Label(_) |
                        AsmValue::StringLiteral(_) |
                        AsmValue::MacroParameter(_)
                            => unreachable!()
                    }
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

