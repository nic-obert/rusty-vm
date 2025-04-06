
use std::rc::Rc;

use rusty_vm_lib::assembly::{self, AsmValue, ByteCode, PseudoInstructionNode, SourceToken, CURRENT_POSITION_TOKEN};
use rusty_vm_lib::byte_code::ByteCodes;
use rusty_vm_lib::registers::Registers;
use rusty_vm_lib::vm::{Address, ADDRESS_SIZE};
use rusty_vm_lib::interrupts::Interrupts;

use crate::debug_info::{DebugInfoTable, InstructionInfo, LabelInfo};
use crate::error;
use crate::symbol_table::SymbolTable;
use crate::module_manager::ModuleManager;
use crate::lang::{AsmNode, AsmNodeValue};


pub fn generate_bytecode<'a>(asm: Box<[AsmNode<'a>]>, symbol_table: &SymbolTable<'a>, module_manager: &ModuleManager<'a>, bytecode: &mut ByteCode, debug_info: &mut Option<DebugInfoTable<'a>>) {

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

            AsmNodeValue::Label(name) => {
                symbol_table.define_label(name, current_pos!());
                if let Some(debug_info) = debug_info {
                    debug_info.add_label(LabelInfo { address: current_pos!(), name, source: node.source });
                }
            },

            AsmNodeValue::Instruction(ref instruction) => {

                if let Some(debug_info) = debug_info {
                    debug_info.add_instruction(InstructionInfo { address: current_pos!(), source: Rc::clone(&node.source) });
                }

                let instruction_code = instruction.byte_code();
                push_byte!(instruction_code);

                #[cfg(debug_assertions)]
                let args_start = current_pos!();

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

                // Check that the size of the arguments is coherent
                if cfg!(debug_assertions) {
                    let args_size = current_pos!() - args_start;
                    let expected_args_size = assembly::bytecode_args_size(instruction_code, &bytecode[args_start..])
                        .expect("Arguments should be correctly formed");
                    assert_eq!(args_size, expected_args_size, "Instruction node {:#?} that corresponds to opcode {instruction_code} was expected to have arguments of size {expected_args_size} bytes, but got {args_size} bytes", node);
                }

            },

            AsmNodeValue::PseudoInstruction (instruction) => {

                match instruction {

                    PseudoInstructionNode::DefineNumber { size, number }
                        // The number size hasn't been checked, do it now
                        => push_sized_number!(number.0, size.0 as usize, &number.1),

                    PseudoInstructionNode::DefineString { string }
                        => push_bytes!(string.0.as_bytes()),

                    PseudoInstructionNode::DefineCString { string } => {
                        push_bytes!(string.0.as_bytes());
                        push_byte!(b'\0');
                    },

                    PseudoInstructionNode::DefineBytes { bytes }
                        => push_bytes!(bytes.0),

                    PseudoInstructionNode::OffsetFrom { label } => {

                        let label_addr = symbol_table.get_resolved_label(label.0).unwrap_or_else(
                            || error::unresolved_label(&label.1, module_manager)
                        );

                        push_bytes!((current_pos!() - label_addr).to_le_bytes());
                    },

                    PseudoInstructionNode::DefineArray { array }
                        => push_bytes!(array.0.to_le_bytes()),

                    PseudoInstructionNode::PrintString { string } => {

                        if let Some(debug_info) = debug_info {
                            debug_info.add_instruction(InstructionInfo { address: current_pos!(), source: node.source });
                        }

                        // jmp <after the string>
                        push_byte!(ByteCodes::JUMP);
                        let after = current_pos!() + string.0.len() + ADDRESS_SIZE;
                        push_bytes!(after.to_le_bytes());

                        // @str_addr
                        let str_addr = current_pos!();

                        // db <string bytes>
                        push_bytes!(string.0.as_bytes());

                        // mov8 print str_addr
                        push_byte!(ByteCodes::MOVE_INTO_REG_FROM_CONST);
                        push_byte!(8);
                        push_byte!(Registers::PRINT);
                        push_bytes!(str_addr.to_le_bytes());

                        // mov8 r1 <string length>
                        push_byte!(ByteCodes::MOVE_INTO_REG_FROM_CONST);
                        push_byte!(8);
                        push_byte!(Registers::R1);
                        push_bytes!(string.0.len().to_le_bytes());

                        // mov1 int =PRINT_BYTES
                        push_byte!(ByteCodes::MOVE_INTO_REG_FROM_CONST);
                        push_byte!(1);
                        push_byte!(Registers::INTERRUPT);
                        push_byte!(Interrupts::PrintBytes);

                        // intr
                        push_byte!(ByteCodes::INTERRUPT);
                    },

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
