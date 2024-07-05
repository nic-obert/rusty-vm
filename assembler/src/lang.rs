use std::collections::HashMap;
use std::{borrow::Cow, mem};
use std::rc::Rc;

use num::traits::ToBytes;

use rusty_vm_lib::registers::Registers;
use rusty_vm_lib::vm::Address;
use rusty_vm_lib::byte_code::ByteCodes;

use crate::tokenizer::{SourceToken, Token};


pub const ENTRY_SECTION_NAME: &'static str = "text";
pub const INCLUDE_SECTION_NAME: &'static str = "include";
pub const CURRENT_POSITION_TOKEN: &'static str = "$";


macro_rules! declare_asm_instructions {
(
    $(
        $name:ident

        size: $handled_size:literal

        argc: $argc:literal

        // Let's call this optional pattern ARGS
        $([
            $(
                $arg1_type:ident

                // Let's call this optional pattern ARG2
                $((
                    $(
                        $arg2_type:ident = $bytecode_two_args:ident
                    ),+
                ))?

                // This optional pattern below and ARG2 are mutually exclusive
                $( = $bytecode_one_arg:ident)?
            ),+
        ])?

        // This optional pattern below and ARGS are mutually exclusive (an instruction cannot have both args and no args)
        $( = $bytecode_no_args:ident)?
    ),+
) => {
    
#[derive(Debug, Clone)]
#[allow(non_camel_case_types)]
#[repr(u8)]
pub enum AsmInstruction {
    
    $($name),+

}

impl AsmInstruction {

    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            $(stringify!($name) => Some(Self::$name),)+
            _ => None
        }
    }

}


#[allow(non_camel_case_types)]
#[allow(dead_code)]
#[derive(Debug)]
pub enum AsmInstructionNode<'a> {

    $($name (Box<[AsmOperand<'a>]>)),+

}

impl<'a> AsmInstructionNode<'a> {

    pub fn handled_size(&self) -> usize {
        match self {
            $(
                Self::$name (_) => $handled_size,
            )+
        }
    }


    pub fn get_args(&self) -> &[AsmOperand<'a>] {
        match self {
            $(
                Self::$name (args)
            )|+
            => args
        }
    }


    /// Checks if the arguments are valid for the given instruction and constructs an instruction node.
    /// Returns an error if the arguments are incorrect.
    pub fn build(instruction: AsmInstruction, args: Box<[AsmOperand<'a>]>) -> Result<AsmInstructionNode<'a>, (Option<Rc<SourceToken<'a>>>, String)> {        
        match instruction {
            $(
                AsmInstruction::$name => {

                    if args.len() != $argc {
                        return Err((
                            None,
                            format!("Operator {:?} expects exactly {} arguments, but {} were given.", instruction, $argc, args.len())
                        ))
                    }

                    $(
                        let arg1 = &args[0];

                        match arg1.value {

                            $(
                                AsmValue::$arg1_type (_) => {
                                    $(
                                        let arg2 = &args[1];

                                        if !matches!(arg2.value, $(AsmValue::$arg2_type (_))|+) {
                                            return Err((
                                                Some(Rc::clone(&arg2.source)),
                                                format!("Wrong second argument type for instruction {:?}: {:?}\nThe second argument is expected to be of type {}", instruction, arg2.value, stringify!($($arg2_type),+))
                                            ))
                                        }
                                    )?
                                },
                            )+

                            #[allow(unreachable_patterns)]
                            _ => return Err((
                                Some(Rc::clone(&arg1.source)),
                                format!("Wrong first argument type for instruction {:?}: {:?}\nThe first argument is expected to be of type {}", instruction, arg1.value, stringify!($($arg1_type),+))
                            ))
                        }
                    )?

                    Ok(Self::$name (args))
                },
            )+
        }
    }


    /// Return the byte code that encodes the specific instruction-arguments case
    /// Assumes the arguments are valid, which is guaranteed if `self` is instantiated through the `build()` function
    pub fn byte_code(&self) -> ByteCodes {
        match self {
            $(
                #[allow(unused_variables)]
                AsmInstructionNode::$name(args) => {

                    $(
                        match &args[0].value {

                            $(
                                AsmValue::$arg1_type (_) => {

                                    $(
                                        match &args[1].value {

                                            $(
                                                AsmValue::$arg2_type (_) => ByteCodes::$bytecode_two_args,
                                            )+

                                            #[allow(unreachable_patterns)]
                                            _ => unreachable!()
                                        }
                                    )?

                                    $(
                                        ByteCodes::$bytecode_one_arg
                                    )?

                                },
                            )+
                            
                            #[allow(unreachable_patterns)]
                            _ => unreachable!()
                        }
                    )?

                    $(
                        ByteCodes::$bytecode_no_args
                    )?

                },
            )+
        }
    }

}

    };
}

declare_asm_instructions! {

    iadd size:0 argc:0 = INTEGER_ADD,
    isub size:0 argc:0 = INTEGER_SUB,
    imul size:0 argc:0 = INTEGER_MUL,
    idiv size:0 argc:0 = INTEGER_DIV,
    imod size:0 argc:0 = INTEGER_MOD,
    fadd size:0 argc:0 = FLOAT_ADD,
    fsub size:0 argc:0 = FLOAT_SUB,
    fmul size:0 argc:0 = FLOAT_MUL,
    fdiv size:0 argc:0 = FLOAT_DIV,
    fmod size:0 argc:0 = FLOAT_MOD,
    inc size:0 argc:1
        [
            Register = INC_REG
        ],
    inc1 size:1 argc:1
        [
            AddressInRegister = INC_ADDR_IN_REG,
            AddressLiteral = INC_ADDR_LITERAL,
            AddressAtLabel = INC_ADDR_LITERAL
        ],
    inc2 size:2 argc:1
        [
            AddressInRegister = INC_ADDR_IN_REG,
            AddressLiteral = INC_ADDR_LITERAL,
            AddressAtLabel = INC_ADDR_LITERAL
        ],
    inc4 size:4 argc:1
        [
            AddressInRegister = INC_ADDR_IN_REG,
            AddressLiteral = INC_ADDR_LITERAL,
            AddressAtLabel = INC_ADDR_LITERAL
        ],
    inc8 size:8 argc:1
        [
            AddressInRegister = INC_ADDR_IN_REG,
            AddressLiteral = INC_ADDR_LITERAL,
            AddressAtLabel = INC_ADDR_LITERAL
        ],
    dec size:0 argc:1
        [
            Register = DEC_REG
        ],
    dec1 size:1 argc:1 
        [
            AddressInRegister = INC_ADDR_IN_REG,
            AddressLiteral = INC_ADDR_LITERAL,
            AddressAtLabel = INC_ADDR_LITERAL
        ],
    dec2 size:2 argc:1 
        [
            AddressInRegister = INC_ADDR_IN_REG,
            AddressLiteral = INC_ADDR_LITERAL,
            AddressAtLabel = INC_ADDR_LITERAL
        ],
    dec4 size:4 argc:1 
        [
            AddressInRegister = INC_ADDR_IN_REG,
            AddressLiteral = INC_ADDR_LITERAL,
            AddressAtLabel = INC_ADDR_LITERAL
        ],
    dec8 size:8 argc:1 
        [
            AddressInRegister = INC_ADDR_IN_REG,
            AddressLiteral = INC_ADDR_LITERAL,
            AddressAtLabel = INC_ADDR_LITERAL
        ],
    nop size:0 argc:0 = NO_OPERATION,
        mov size:0 argc:2 
        [
            Register (
                Register = MOVE_INTO_REG_FROM_REG
            )
        ],
    mov1 size:1 argc:2 
        [
            Register (
                Register = MOVE_INTO_REG_FROM_REG_SIZED,
                AddressInRegister = MOVE_INTO_REG_FROM_ADDR_IN_REG,
                Number = MOVE_INTO_REG_FROM_CONST,
                AddressLiteral = MOVE_INTO_REG_FROM_ADDR_LITERAL,
                AddressAtLabel = MOVE_INTO_REG_FROM_ADDR_LITERAL
            ),
            AddressInRegister (
                Register = MOVE_INTO_ADDR_IN_REG_FROM_REG,
                AddressInRegister = MOVE_INTO_ADDR_IN_REG_FROM_ADDR_IN_REG,
                Number = MOVE_INTO_ADDR_IN_REG_FROM_CONST,
                AddressLiteral = MOVE_INTO_ADDR_IN_REG_FROM_ADDR_LITERAL,
                AddressAtLabel = MOVE_INTO_ADDR_IN_REG_FROM_ADDR_LITERAL
            ),
            AddressLiteral (
                Register = MOVE_INTO_ADDR_LITERAL_FROM_REG,
                AddressInRegister = MOVE_INTO_ADDR_LITERAL_FROM_ADDR_IN_REG,
                Number = MOVE_INTO_ADDR_LITERAL_FROM_CONST,
                AddressLiteral = MOVE_INTO_ADDR_LITERAL_FROM_ADDR_LITERAL,
                AddressAtLabel = MOVE_INTO_ADDR_LITERAL_FROM_ADDR_LITERAL
            ),
            AddressAtLabel (
                Register = MOVE_INTO_ADDR_LITERAL_FROM_REG,
                AddressInRegister = MOVE_INTO_ADDR_LITERAL_FROM_ADDR_IN_REG,
                Number = MOVE_INTO_ADDR_LITERAL_FROM_CONST,
                AddressLiteral = MOVE_INTO_ADDR_LITERAL_FROM_ADDR_LITERAL,
                AddressAtLabel = MOVE_INTO_ADDR_LITERAL_FROM_ADDR_LITERAL
            )
        ],
    mov2 size:2 argc:2
        [
            Register (
                Register = MOVE_INTO_REG_FROM_REG_SIZED,
                AddressInRegister = MOVE_INTO_REG_FROM_ADDR_IN_REG,
                Number = MOVE_INTO_REG_FROM_CONST,
                AddressLiteral = MOVE_INTO_REG_FROM_ADDR_LITERAL,
                AddressAtLabel = MOVE_INTO_REG_FROM_ADDR_LITERAL
            ),
            AddressInRegister (
                Register = MOVE_INTO_ADDR_IN_REG_FROM_REG,
                AddressInRegister = MOVE_INTO_ADDR_IN_REG_FROM_ADDR_IN_REG,
                Number = MOVE_INTO_ADDR_IN_REG_FROM_CONST,
                AddressLiteral = MOVE_INTO_ADDR_IN_REG_FROM_ADDR_LITERAL,
                AddressAtLabel = MOVE_INTO_ADDR_IN_REG_FROM_ADDR_LITERAL
            ),
            AddressLiteral (
                Register = MOVE_INTO_ADDR_LITERAL_FROM_REG,
                AddressInRegister = MOVE_INTO_ADDR_LITERAL_FROM_ADDR_IN_REG,
                Number = MOVE_INTO_ADDR_LITERAL_FROM_CONST,
                AddressLiteral = MOVE_INTO_ADDR_LITERAL_FROM_ADDR_LITERAL,
                AddressAtLabel = MOVE_INTO_ADDR_LITERAL_FROM_ADDR_LITERAL
            ),
            AddressAtLabel (
                Register = MOVE_INTO_ADDR_LITERAL_FROM_REG,
                AddressInRegister = MOVE_INTO_ADDR_LITERAL_FROM_ADDR_IN_REG,
                Number = MOVE_INTO_ADDR_LITERAL_FROM_CONST,
                AddressLiteral = MOVE_INTO_ADDR_LITERAL_FROM_ADDR_LITERAL,
                AddressAtLabel = MOVE_INTO_ADDR_LITERAL_FROM_ADDR_LITERAL
            )
        ],
    mov4 size:4 argc:2
        [
            Register (
                Register = MOVE_INTO_REG_FROM_REG_SIZED,
                AddressInRegister = MOVE_INTO_REG_FROM_ADDR_IN_REG,
                Number = MOVE_INTO_REG_FROM_CONST,
                AddressLiteral = MOVE_INTO_REG_FROM_ADDR_LITERAL,
                AddressAtLabel = MOVE_INTO_REG_FROM_ADDR_LITERAL
            ),
            AddressInRegister (
                Register = MOVE_INTO_ADDR_IN_REG_FROM_REG,
                AddressInRegister = MOVE_INTO_ADDR_IN_REG_FROM_ADDR_IN_REG,
                Number = MOVE_INTO_ADDR_IN_REG_FROM_CONST,
                AddressLiteral = MOVE_INTO_ADDR_IN_REG_FROM_ADDR_LITERAL,
                AddressAtLabel = MOVE_INTO_ADDR_IN_REG_FROM_ADDR_LITERAL
            ),
            AddressLiteral (
                Register = MOVE_INTO_ADDR_LITERAL_FROM_REG,
                AddressInRegister = MOVE_INTO_ADDR_LITERAL_FROM_ADDR_IN_REG,
                Number = MOVE_INTO_ADDR_LITERAL_FROM_CONST,
                AddressLiteral = MOVE_INTO_ADDR_LITERAL_FROM_ADDR_LITERAL,
                AddressAtLabel = MOVE_INTO_ADDR_LITERAL_FROM_ADDR_LITERAL
            ),
            AddressAtLabel (
                Register = MOVE_INTO_ADDR_LITERAL_FROM_REG,
                AddressInRegister = MOVE_INTO_ADDR_LITERAL_FROM_ADDR_IN_REG,
                Number = MOVE_INTO_ADDR_LITERAL_FROM_CONST,
                AddressLiteral = MOVE_INTO_ADDR_LITERAL_FROM_ADDR_LITERAL,
                AddressAtLabel = MOVE_INTO_ADDR_LITERAL_FROM_ADDR_LITERAL
            )
        ],
    mov8 size:8 argc:2
        [
            Register (
                Register = MOVE_INTO_REG_FROM_REG_SIZED,
                AddressInRegister = MOVE_INTO_REG_FROM_ADDR_IN_REG,
                Number = MOVE_INTO_REG_FROM_CONST,
                AddressLiteral = MOVE_INTO_REG_FROM_ADDR_LITERAL,
                AddressAtLabel = MOVE_INTO_REG_FROM_ADDR_LITERAL,
                Label = MOVE_INTO_REG_FROM_CONST
            ),
            AddressInRegister (
                Register = MOVE_INTO_ADDR_IN_REG_FROM_REG,
                AddressInRegister = MOVE_INTO_ADDR_IN_REG_FROM_ADDR_IN_REG,
                Number = MOVE_INTO_ADDR_IN_REG_FROM_CONST,
                AddressLiteral = MOVE_INTO_ADDR_IN_REG_FROM_ADDR_LITERAL,
                AddressAtLabel = MOVE_INTO_ADDR_IN_REG_FROM_ADDR_LITERAL,
                Label = MOVE_INTO_ADDR_IN_REG_FROM_CONST
            ),
            AddressLiteral (
                Register = MOVE_INTO_ADDR_LITERAL_FROM_REG,
                AddressInRegister = MOVE_INTO_ADDR_LITERAL_FROM_ADDR_IN_REG,
                Number = MOVE_INTO_ADDR_LITERAL_FROM_CONST,
                AddressLiteral = MOVE_INTO_ADDR_LITERAL_FROM_ADDR_LITERAL,
                AddressAtLabel = MOVE_INTO_ADDR_LITERAL_FROM_ADDR_LITERAL,
                Label = MOVE_INTO_ADDR_LITERAL_FROM_CONST
            ),
            AddressAtLabel (
                Register = MOVE_INTO_ADDR_LITERAL_FROM_REG,
                AddressInRegister = MOVE_INTO_ADDR_LITERAL_FROM_ADDR_IN_REG,
                Number = MOVE_INTO_ADDR_LITERAL_FROM_CONST,
                AddressLiteral = MOVE_INTO_ADDR_LITERAL_FROM_ADDR_LITERAL,
                AddressAtLabel = MOVE_INTO_ADDR_LITERAL_FROM_ADDR_LITERAL,
                Label = MOVE_INTO_ADDR_LITERAL_FROM_CONST
            )
        ],
    push size:0 argc:1
        [
            Register = PUSH_FROM_REG
        ],
    push1 size:1 argc:1
        [
            Register = PUSH_FROM_REG_SIZED,
            AddressInRegister = PUSH_FROM_ADDR_IN_REG,
            Number = PUSH_FROM_CONST,
            AddressLiteral = PUSH_FROM_ADDR_LITERAL,
            AddressAtLabel = PUSH_FROM_ADDR_LITERAL
        ],
    push2 size:2 argc:1
        [
            Register = PUSH_FROM_REG_SIZED,
            AddressInRegister = PUSH_FROM_ADDR_IN_REG,
            Number = PUSH_FROM_CONST,
            AddressLiteral = PUSH_FROM_ADDR_LITERAL,
            AddressAtLabel = PUSH_FROM_ADDR_LITERAL
        ],
    push4 size:4 argc:1
        [
            Register = PUSH_FROM_REG_SIZED,
            AddressInRegister = PUSH_FROM_ADDR_IN_REG,
            Number = PUSH_FROM_CONST,
            AddressLiteral = PUSH_FROM_ADDR_LITERAL,
            AddressAtLabel = PUSH_FROM_ADDR_LITERAL
        ],
    push8 size:8 argc:1
        [
            Register = PUSH_FROM_REG_SIZED,
            AddressInRegister = PUSH_FROM_ADDR_IN_REG,
            Number = PUSH_FROM_CONST,
            AddressLiteral = PUSH_FROM_ADDR_LITERAL,
            AddressAtLabel = PUSH_FROM_ADDR_LITERAL,
            Label = PUSH_FROM_CONST
        ],
    pushsp size:0 argc:1
        [
            Register = PUSH_STACK_POINTER_REG
        ],
    pushsp1 size:1 argc:1
        [
            Register = PUSH_STACK_POINTER_REG_SIZED,
            AddressInRegister = PUSH_STACK_POINTER_ADDR_IN_REG,
            Number = PUSH_STACK_POINTER_CONST,
            AddressLiteral = PUSH_STACK_POINTER_ADDR_LITERAL,
            AddressAtLabel = PUSH_STACK_POINTER_ADDR_LITERAL
        ],
    pushsp2 size:2 argc:1
        [
            Register = PUSH_STACK_POINTER_REG_SIZED,
            AddressInRegister = PUSH_STACK_POINTER_ADDR_IN_REG,
            Number = PUSH_STACK_POINTER_CONST,
            AddressLiteral = PUSH_STACK_POINTER_ADDR_LITERAL,
            AddressAtLabel = PUSH_STACK_POINTER_ADDR_LITERAL
        ],
    pushsp4 size:4 argc:1
        [
            Register = PUSH_STACK_POINTER_REG_SIZED,
            AddressInRegister = PUSH_STACK_POINTER_ADDR_IN_REG,
            Number = PUSH_STACK_POINTER_CONST,
            AddressLiteral = PUSH_STACK_POINTER_ADDR_LITERAL,
            AddressAtLabel = PUSH_STACK_POINTER_ADDR_LITERAL
        ],
    pushsp8 size:8 argc:1
        [
            Register = PUSH_STACK_POINTER_REG_SIZED,
            AddressInRegister = PUSH_STACK_POINTER_ADDR_IN_REG,
            Number = PUSH_STACK_POINTER_CONST,
            AddressLiteral = PUSH_STACK_POINTER_ADDR_LITERAL,
            AddressAtLabel = PUSH_STACK_POINTER_ADDR_LITERAL,
            Label = PUSH_STACK_POINTER_CONST
        ],
    pop1 size:1 argc:1
        [
            Register = POP_INTO_REG,
            AddressInRegister = POP_INTO_ADDR_IN_REG,
            AddressLiteral = POP_INTO_ADDR_LITERAL,
            AddressAtLabel = POP_INTO_ADDR_LITERAL
        ],
    pop2 size:2 argc:1
        [
            Register = POP_INTO_REG,
            AddressInRegister = POP_INTO_ADDR_IN_REG,
            AddressLiteral = POP_INTO_ADDR_LITERAL,
            AddressAtLabel = POP_INTO_ADDR_LITERAL
        ],
    pop4 size:4 argc:1
        [
            Register = POP_INTO_REG,
            AddressInRegister = POP_INTO_ADDR_IN_REG,
            AddressLiteral = POP_INTO_ADDR_LITERAL,
            AddressAtLabel = POP_INTO_ADDR_LITERAL
        ],
    pop8 size:8 argc:1
        [
            Register = POP_INTO_REG,
            AddressInRegister = POP_INTO_ADDR_IN_REG,
            AddressLiteral = POP_INTO_ADDR_LITERAL,
            AddressAtLabel = POP_INTO_ADDR_LITERAL
        ],
    popsp size:0 argc:1
        [
            Register = POP_STACK_POINTER_REG
        ],
    popsp1 size:1 argc:1
        [
            Register = POP_STACK_POINTER_REG,
            AddressInRegister = POP_STACK_POINTER_ADDR_IN_REG,
            AddressLiteral = POP_STACK_POINTER_ADDR_LITERAL,
            AddressAtLabel = POP_STACK_POINTER_ADDR_LITERAL
        ],
    popsp2 size:2 argc:1
        [
            Register = POP_STACK_POINTER_REG,
            AddressInRegister = POP_STACK_POINTER_ADDR_IN_REG,
            AddressLiteral = POP_STACK_POINTER_ADDR_LITERAL,
            AddressAtLabel = POP_STACK_POINTER_ADDR_LITERAL
        ],
    popsp4 size:4 argc:1
        [
            Register = POP_STACK_POINTER_REG,
            AddressInRegister = POP_STACK_POINTER_ADDR_IN_REG,
            AddressLiteral = POP_STACK_POINTER_ADDR_LITERAL,
            AddressAtLabel = POP_STACK_POINTER_ADDR_LITERAL
        ],
    popsp8 size:8 argc:1
        [
            Register = POP_STACK_POINTER_REG,
            AddressInRegister = POP_STACK_POINTER_ADDR_IN_REG,
            AddressLiteral = POP_STACK_POINTER_ADDR_LITERAL,
            AddressAtLabel = POP_STACK_POINTER_ADDR_LITERAL
        ],
    jmp size:0 argc:1
        [
            Label = JUMP
        ],
    jmpnz size:0 argc:1
        [
            Label = JUMP_NOT_ZERO
        ],
    jmpz size:0 argc:1
        [
            Label = JUMP_ZERO
        ],
    jmpgr size:0 argc:1
        [
            Label = JUMP_GREATER
        ],
    jmpge size:0 argc:1
        [
            Label = JUMP_GREATER_OR_EQUAL
        ],
    jmplt size:0 argc:1
        [
            Label = JUMP_LESS
        ],
    jmple size:0 argc:1
        [
            Label = JUMP_LESS_OR_EQUAL
        ],
    jmpof size:0 argc:1
        [
            Label = JUMP_OVERFLOW
        ],
    jmpnof size:0 argc:1 
        [
            Label = JUMP_NOT_OVERFLOW
        ],
    jmpcr size:0 argc:1
        [
            Label = JUMP_CARRY
        ],
    jmpncr size:0 argc:1
        [
            Label = JUMP_NOT_CARRY
        ],
    jmpsn size:0 argc:1
        [
            Label = JUMP_SIGN
        ],
    jmpnsn size:0 argc:1
        [
            Label = JUMP_NOT_SIGN
        ],
    call size:0 argc:1
        [
            Label = CALL
        ],
    ret size:0 argc:0 = RETURN,
    cmp size:0 argc:2
        [
            Register (
                Register = COMPARE_REG_REG
            )
        ],
        cmp1 size:1 argc:2
        [
            Register (
                Register = COMPARE_REG_REG_SIZED,
                AddressInRegister = COMPARE_REG_ADDR_IN_REG,
                Number = COMPARE_REG_CONST,
                AddressLiteral = COMPARE_REG_ADDR_LITERAL,
                AddressAtLabel = COMPARE_REG_ADDR_LITERAL
            ),
            AddressInRegister (
                Register = COMPARE_ADDR_IN_REG_REG,
                AddressInRegister = COMPARE_ADDR_IN_REG_ADDR_IN_REG,
                Number = COMPARE_ADDR_IN_REG_CONST,
                AddressLiteral = COMPARE_ADDR_IN_REG_ADDR_LITERAL,
                AddressAtLabel = COMPARE_ADDR_IN_REG_ADDR_LITERAL
            ),
            Number (
                Register = COMPARE_CONST_REG,
                AddressInRegister = COMPARE_CONST_ADDR_IN_REG,
                Number = COMPARE_CONST_CONST,
                AddressLiteral = COMPARE_CONST_ADDR_LITERAL,
                AddressAtLabel = COMPARE_CONST_ADDR_LITERAL
            ),
            AddressLiteral (
                Register = COMPARE_ADDR_LITERAL_REG,
                AddressInRegister = COMPARE_ADDR_LITERAL_ADDR_IN_REG,
                Number = COMPARE_ADDR_LITERAL_CONST,
                AddressLiteral = COMPARE_ADDR_LITERAL_ADDR_LITERAL,
                AddressAtLabel = COMPARE_ADDR_LITERAL_ADDR_LITERAL
            ),
            AddressAtLabel (
                Register = COMPARE_ADDR_LITERAL_REG,
                AddressInRegister = COMPARE_ADDR_LITERAL_ADDR_IN_REG,
                Number = COMPARE_ADDR_LITERAL_CONST,
                AddressLiteral = COMPARE_ADDR_LITERAL_ADDR_LITERAL,
                AddressAtLabel = COMPARE_ADDR_LITERAL_ADDR_LITERAL
            )
        ],
    cmp2 size:2 argc:2
        [
            Register (
                Register = COMPARE_REG_REG_SIZED,
                AddressInRegister = COMPARE_REG_ADDR_IN_REG,
                Number = COMPARE_REG_CONST,
                AddressLiteral = COMPARE_REG_ADDR_LITERAL,
                AddressAtLabel = COMPARE_REG_ADDR_LITERAL
            ),
            AddressInRegister (
                Register = COMPARE_ADDR_IN_REG_REG,
                AddressInRegister = COMPARE_ADDR_IN_REG_ADDR_IN_REG,
                Number = COMPARE_ADDR_IN_REG_CONST,
                AddressLiteral = COMPARE_ADDR_IN_REG_ADDR_LITERAL,
                AddressAtLabel = COMPARE_ADDR_IN_REG_ADDR_LITERAL
            ),
            Number (
                Register = COMPARE_CONST_REG,
                AddressInRegister = COMPARE_CONST_ADDR_IN_REG,
                Number = COMPARE_CONST_CONST,
                AddressLiteral = COMPARE_CONST_ADDR_LITERAL,
                AddressAtLabel = COMPARE_CONST_ADDR_LITERAL
            ),
            AddressLiteral (
                Register = COMPARE_ADDR_LITERAL_REG,
                AddressInRegister = COMPARE_ADDR_LITERAL_ADDR_IN_REG,
                Number = COMPARE_ADDR_LITERAL_CONST,
                AddressLiteral = COMPARE_ADDR_LITERAL_ADDR_LITERAL,
                AddressAtLabel = COMPARE_ADDR_LITERAL_ADDR_LITERAL
            ),
            AddressAtLabel (
                Register = COMPARE_ADDR_LITERAL_REG,
                AddressInRegister = COMPARE_ADDR_LITERAL_ADDR_IN_REG,
                Number = COMPARE_ADDR_LITERAL_CONST,
                AddressLiteral = COMPARE_ADDR_LITERAL_ADDR_LITERAL,
                AddressAtLabel = COMPARE_ADDR_LITERAL_ADDR_LITERAL
            )
        ],
    cmp4 size:4 argc:2
        [
            Register (
                Register = COMPARE_REG_REG_SIZED,
                AddressInRegister = COMPARE_REG_ADDR_IN_REG,
                Number = COMPARE_REG_CONST,
                AddressLiteral = COMPARE_REG_ADDR_LITERAL,
                AddressAtLabel = COMPARE_REG_ADDR_LITERAL
            ),
            AddressInRegister (
                Register = COMPARE_ADDR_IN_REG_REG,
                AddressInRegister = COMPARE_ADDR_IN_REG_ADDR_IN_REG,
                Number = COMPARE_ADDR_IN_REG_CONST,
                AddressLiteral = COMPARE_ADDR_IN_REG_ADDR_LITERAL,
                AddressAtLabel = COMPARE_ADDR_IN_REG_ADDR_LITERAL
            ),
            Number (
                Register = COMPARE_CONST_REG,
                AddressInRegister = COMPARE_CONST_ADDR_IN_REG,
                Number = COMPARE_CONST_CONST,
                AddressLiteral = COMPARE_CONST_ADDR_LITERAL,
                AddressAtLabel = COMPARE_CONST_ADDR_LITERAL
            ),
            AddressLiteral (
                Register = COMPARE_ADDR_LITERAL_REG,
                AddressInRegister = COMPARE_ADDR_LITERAL_ADDR_IN_REG,
                Number = COMPARE_ADDR_LITERAL_CONST,
                AddressLiteral = COMPARE_ADDR_LITERAL_ADDR_LITERAL,
                AddressAtLabel = COMPARE_ADDR_LITERAL_ADDR_LITERAL
            ),
            AddressAtLabel (
                Register = COMPARE_ADDR_LITERAL_REG,
                AddressInRegister = COMPARE_ADDR_LITERAL_ADDR_IN_REG,
                Number = COMPARE_ADDR_LITERAL_CONST,
                AddressLiteral = COMPARE_ADDR_LITERAL_ADDR_LITERAL,
                AddressAtLabel = COMPARE_ADDR_LITERAL_ADDR_LITERAL
            )
        ],
    cmp8 size:8 argc:2
        [
            Register (
                Register = COMPARE_REG_REG_SIZED,
                AddressInRegister = COMPARE_REG_ADDR_IN_REG,
                Number = COMPARE_REG_CONST,
                AddressLiteral = COMPARE_REG_ADDR_LITERAL,
                AddressAtLabel = COMPARE_REG_ADDR_LITERAL,
                Label = COMPARE_REG_CONST
            ),
            AddressInRegister (
                Register = COMPARE_ADDR_IN_REG_REG,
                AddressInRegister = COMPARE_ADDR_IN_REG_ADDR_IN_REG,
                Number = COMPARE_ADDR_IN_REG_CONST,
                AddressLiteral = COMPARE_ADDR_IN_REG_ADDR_LITERAL,
                AddressAtLabel = COMPARE_ADDR_IN_REG_ADDR_LITERAL,
                Label = COMPARE_ADDR_IN_REG_CONST
            ),
            Number (
                Register = COMPARE_CONST_REG,
                AddressInRegister = COMPARE_CONST_ADDR_IN_REG,
                Number = COMPARE_CONST_CONST,
                AddressLiteral = COMPARE_CONST_ADDR_LITERAL,
                AddressAtLabel = COMPARE_CONST_ADDR_LITERAL,
                Label = COMPARE_CONST_CONST
            ),
            AddressLiteral (
                Register = COMPARE_ADDR_LITERAL_REG,
                AddressInRegister = COMPARE_ADDR_LITERAL_ADDR_IN_REG,
                Number = COMPARE_ADDR_LITERAL_CONST,
                AddressLiteral = COMPARE_ADDR_LITERAL_ADDR_LITERAL,
                AddressAtLabel = COMPARE_ADDR_LITERAL_ADDR_LITERAL,
                Label = COMPARE_ADDR_LITERAL_CONST
            ),
            AddressAtLabel (
                Register = COMPARE_ADDR_LITERAL_REG,
                AddressInRegister = COMPARE_ADDR_LITERAL_ADDR_IN_REG,
                Number = COMPARE_ADDR_LITERAL_CONST,
                AddressLiteral = COMPARE_ADDR_LITERAL_ADDR_LITERAL,
                AddressAtLabel = COMPARE_ADDR_LITERAL_ADDR_LITERAL,
                Label = COMPARE_ADDR_LITERAL_CONST
            ),
            Label (
                Register = COMPARE_CONST_REG,
                AddressInRegister = COMPARE_CONST_ADDR_IN_REG,
                Number = COMPARE_CONST_CONST,
                AddressLiteral = COMPARE_CONST_ADDR_LITERAL,
                AddressAtLabel = COMPARE_CONST_ADDR_LITERAL,
                Label = COMPARE_CONST_CONST
            )
        ],
    and size:0 argc:0 = AND,
    or size:0 argc:0 = OR,
    xor size:0 argc:0 = XOR,
    not size:0 argc:0 = NOT,
    shl size:0 argc:0 = SHIFT_LEFT,
    shr size:0 argc:0 = SHIFT_RIGHT,
    intr size:0 argc:0 = INTERRUPT,
    exit size:0 argc:0 = EXIT

}


macro_rules! declare_pseudo_instructions {
(
    $(
        $name:ident
        $asm_name:literal
        {$(
            $field:ident: $field_type:ty
        ),*}
    ),+
) => {
        
/// Pseudo-instructions are assembler-specific instructions that get evaluated at compile-time and have effects on the generated output byte code.
/// Each instruction is represented by one byte.
#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum PseudoInstructions {
    $($name),+
}

impl PseudoInstructions {

    pub fn from_name(string: &str) -> Option<Self> {
        match string {
            $($asm_name => Some(Self::$name),)+
            _ => None
        }
    }

}


#[derive(Debug)]
pub enum PseudoInstructionNode<'a> {

    $($name { $( $field: ($field_type, Rc<SourceToken<'a>>) ),* }),+

}


    };
}

declare_pseudo_instructions! {

    DefineNumber "dn" { size: u8, data: Number },
    DefineBytes "db" { data: Box<[u8]> },
    DefineString "ds" { data: Cow<'a, str> }

}


#[derive(Debug, Clone)]
pub enum Number {

    SignedInt(i64),
    UnsignedInt(u64),
    Float(f64)
    
}

impl Number {

    pub fn as_bytes(&self) -> [u8; 8] {
        match self {
            Number::SignedInt(i) => i.to_le_bytes(),
            Number::UnsignedInt(u) => u.to_le_bytes(),
            Number::Float(f) => f.to_le_bytes(),
        }
    }


    /// Returns how many bytes are necessary to correctly represent the number.
    /// The returned number is always a power of 2 (including 1 as in 2^0=1)
    pub fn least_bytes_repr(&self) -> usize {
        match self {
            Number::SignedInt(_) => 8, // Conservative approach
            Number::Float(_) => 8, // Conservative approach
            Number::UnsignedInt(n) => { // Not very elegant, but it works
                if *n <= u8::MAX as u64 {
                    mem::size_of::<u8>()
                } else if *n <= u16::MAX as u64 {
                    mem::size_of::<u16>()
                } else if *n <= u32::MAX as u64 {
                    mem::size_of::<u32>()
                } else {
                    mem::size_of::<u64>()
                }
            }
        }
    }

}


#[derive(Debug, Clone)]
pub enum AsmValue<'a> {

    Register (Registers),
    AddressInRegister (Registers),
    Number (Number),
    AddressLiteral (Number),
    AddressAtLabel (&'a str),
    Label (&'a str),

}


#[derive(Debug, Clone)]
pub struct AsmOperand<'a> {

    pub value: AsmValue<'a>,
    pub source: Rc<SourceToken<'a>>

}


#[derive(Debug, Clone)]
/// Represents a macro definition in the assembly code
pub struct FunctionMacroDef<'a> {

    pub source: Rc<SourceToken<'a>>,
    /// Maps the parameter name to its position in the invocation
    pub params: HashMap<&'a str, usize>,
    pub body: Box<[Box<[Token<'a>]>]>,

}


#[derive(Debug, Clone)]
pub struct InlineMacroDef<'a> {

    pub source: Rc<SourceToken<'a>>,
    pub def: Box<[Token<'a>]>

}


#[derive(Debug, Clone)]
pub struct LabelDef<'a> {

    pub source: Rc<SourceToken<'a>>,
    pub value: Option<Address>

}


#[derive(Debug)]
pub struct AsmNode<'a> {

    pub source: Rc<SourceToken<'a>>,
    pub value: AsmNodeValue<'a>

}


#[derive(Debug)]
pub enum AsmNodeValue<'a> {

    Instruction (AsmInstructionNode<'a>),
    PseudoInstruction (PseudoInstructionNode<'a>),
    Label (&'a str),

}

