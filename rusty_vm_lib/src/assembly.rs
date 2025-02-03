use crate::registers::Registers;
use crate::byte_code::ByteCodes;
use crate::vm::{ADDRESS_SIZE, Address, HANDLED_SIZE_SPECIFIER, REGISTER_ID_SIZE};

use std::path::Path;
use std::mem;
use std::borrow::Cow;
use std::rc::Rc;
use core::fmt;

use static_assertions::const_assert;


pub type ByteCode = Vec<u8>;

pub const LIBRARY_ENV_VARIABLE: &str = "RUSTYVM_ASM_LIB";


pub const ENTRY_SECTION_NAME: &str = "text";
pub const INCLUDE_SECTION_NAME: &str = "include";
pub const CURRENT_POSITION_TOKEN: &str = "$";


macro_rules! asm_arg_size {

    (Register, $_handled_size:ident) => {
        REGISTER_ID_SIZE
    };

    (AddressInRegister, $_handled_size:ident) => {
        REGISTER_ID_SIZE
    };

    (Number, $handled_size:ident) => {
        $handled_size
    };


    (AddressLiteral, $_handled_size:ident) => {
        ADDRESS_SIZE
    };

    (AddressAtLabel, $_handled_size:ident) => {
        ADDRESS_SIZE
    };

    (Label, $_handled_size:ident) => {
        ADDRESS_SIZE
    };
}


const DO_INCREASE_I: () = ();


#[derive(Debug)]
pub struct InvalidASMArgsError {}
impl fmt::Display for InvalidASMArgsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Invalid ASM argument")
    }
}
impl std::error::Error for InvalidASMArgsError { }


macro_rules! parse_asm_arg {

    (Register, $i:ident, $bytecode_after:ident, $_handled_size: ident $(, $do_increase_i:ident)?) => {{

        let reg = $bytecode_after[$i];

        $(
            let _ = $do_increase_i;
            $i += REGISTER_ID_SIZE;
        )?

        AsmValue::Register(Registers::from(reg))
    }};

    (AddressInRegister, $i:ident, $bytecode_after:ident, $_handled_size: ident $(, $do_increase_i:ident)?) => {{

        let reg = $bytecode_after[$i];

        $(
            let _ = $do_increase_i;
            $i += REGISTER_ID_SIZE;
        )?

        AsmValue::AddressInRegister(Registers::from(reg))
    }};

    (Number, $i:ident, $bytecode_after:ident, $handled_size: ident $(, $do_increase_i:ident)?) => {{

        let n = Number::uint_from_bytes(&$bytecode_after[$i..$i + $handled_size])?;

        $(
            let _ = $do_increase_i;
            $i += $handled_size;
        )?

        AsmValue::Number(n)
    }};


    (AddressLiteral, $i:ident, $bytecode_after:ident, $_handled_size: ident $(, $do_increase_i:ident)?) => {{

        let addr = Address::from_le_bytes($bytecode_after[$i..$i + ADDRESS_SIZE].try_into().unwrap()); // TODO: the try_into() never fails because [u8; ADDRESS_SIZE] is a valid Address

        $(
            let _ =$do_increase_i;
            $i += ADDRESS_SIZE;
        )?

        AsmValue::AddressLiteral(Number::UnsignedInt(addr as u64))
    }};

    (AddressAtLabel, $i:ident, $bytecode_after:ident, $_handled_size: ident $(, $do_increase_i:ident)?) => {
        // Labels get compiled to raw numbers, so this is equivalent to an address literal
        parse_asm_arg!(AddressLiteral, $i, $bytecode_after, $_handled_size $(, $do_increase_i)?)
    };

    (Label, $i:ident, $bytecode_after:ident, $_handled_size: ident $(, $do_increase_i:ident)?) => {
        // Labels get compiled to raw numbers
        parse_asm_arg!(Number, $i, $bytecode_after, ADDRESS_SIZE $(, $do_increase_i)?)
    };
}


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
                        const_assert!($argc >= 1);

                        let arg1 = &args[0];

                        match arg1.value {

                            $(
                                AsmValue::$arg1_type (_) => {
                                    $(
                                        const_assert!($argc == 2);

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


/// Return the arguments to the operation.
/// The Ok return value is a tuple containing the actual handled size and a list of asm arguments
pub fn parse_bytecode_args(instruction: ByteCodes, bytecode_after: &[u8]) -> Result<(usize, Box<[AsmValue<'static>]>), String> {

    let mut i = 0;

    Ok(match instruction {
        // For every opcode
        $(

            // At least one arg is expected
            $(

                // For every first arg
                $(

                    // Two args are expected
                    $(

                        // For every second arg
                        $(
                            // Allow duplicated pattterns beacause multiple groups of asm instructions and arguments can be compiled to the same bytecode
                            #[allow(unreachable_patterns)]
                            ByteCodes::$bytecode_two_args => {

                                // Allow to not use `actual_handled_size` because some sized instructions have arguments whose size don't depend on the handled size specifier
                                #[allow(unused_variables)]
                                let actual_handled_size = if $handled_size > 0 {
                                    // A handled size specifier is present
                                    let handled_size = bytecode_after[0] as usize;
                                    i += HANDLED_SIZE_SPECIFIER;
                                    handled_size
                                } else {
                                    0
                                };

                                (
                                    actual_handled_size,
                                    Box::new([
                                        parse_asm_arg!($arg1_type, i, bytecode_after, actual_handled_size, DO_INCREASE_I), // Increase i only if some arguments are expected to follow
                                        parse_asm_arg!($arg2_type, i, bytecode_after, actual_handled_size)
                                    ])
                                )
                            },
                        )+

                    )?

                    // One arg is expected
                    $(
                        // Allow duplicated pattterns beacause multiple groups of asm instructions and arguments can be compiled to the same bytecode
                        #[allow(unreachable_patterns)]
                        ByteCodes::$bytecode_one_arg => {

                            // Allow to not use `actual_handled_size` because some sized instructions have arguments whose size don't depend on the handled size specifier
                            #[allow(unused_variables)]
                            let actual_handled_size = if $handled_size > 0 {
                                // A handled size specifier is present
                                let handled_size = bytecode_after[0] as usize;
                                i += HANDLED_SIZE_SPECIFIER;
                                handled_size
                            } else {
                                0
                            };

                            (
                                actual_handled_size,
                                Box::new([
                                    parse_asm_arg!($arg1_type, i, bytecode_after, actual_handled_size)
                                ])
                            )
                        },
                    )?

                )+

            )?

            // No args are expected
            $(
                ByteCodes::$bytecode_no_args => (0, Box::new([])),
            )?
        )+
    })

}


pub fn bytecode_args_size(instruction: ByteCodes, bytecode_after: &[u8]) -> Result<usize, InvalidASMArgsError> {

    Ok(match instruction {
        // For every opcode
        $(

            // At least one arg
            $(

                // For every first arg
                $(

                    // Two args
                    $(

                        // For every second arg
                        $(
                            #[allow(unreachable_patterns)]
                            ByteCodes::$bytecode_two_args => {

                                #[allow(unused_variables)]
                                let (base_arg_size, actual_handled_size) = if $handled_size > 0 {
                                    (
                                        HANDLED_SIZE_SPECIFIER,
                                        *bytecode_after.get(0).ok_or(InvalidASMArgsError {})? as usize
                                    )
                                } else {
                                    (0, 0)
                                };

                                base_arg_size
                                + asm_arg_size!($arg1_type, actual_handled_size)
                                + asm_arg_size!($arg2_type, actual_handled_size)
                            }
                        ),+
                    )?

                    // Only one arg
                    $(
                        #[allow(unreachable_patterns)]
                        ByteCodes::$bytecode_one_arg => {

                            #[allow(unused_variables)]
                            let (base_arg_size, actual_handled_size) = if $handled_size > 0 {
                                (
                                    HANDLED_SIZE_SPECIFIER,
                                    *bytecode_after.get(0).ok_or(InvalidASMArgsError {})? as usize
                                )
                            } else {
                                (0, 0)
                            };

                            base_arg_size
                            + asm_arg_size!($arg1_type, actual_handled_size)
                        }
                    )?

                ),+

            )?

            // No args
            $(ByteCodes::$bytecode_no_args => 0,)?

        )+
    })
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
            AddressInRegister = DEC_ADDR_IN_REG,
            AddressLiteral = DEC_ADDR_LITERAL,
            AddressAtLabel = DEC_ADDR_LITERAL
        ],
    dec2 size:2 argc:1
        [
            AddressInRegister = DEC_ADDR_IN_REG,
            AddressLiteral = DEC_ADDR_LITERAL,
            AddressAtLabel = DEC_ADDR_LITERAL
        ],
    dec4 size:4 argc:1
        [
            AddressInRegister = DEC_ADDR_IN_REG,
            AddressLiteral = DEC_ADDR_LITERAL,
            AddressAtLabel = DEC_ADDR_LITERAL
        ],
    dec8 size:8 argc:1
        [
            AddressInRegister = DEC_ADDR_IN_REG,
            AddressLiteral = DEC_ADDR_LITERAL,
            AddressAtLabel = DEC_ADDR_LITERAL
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
    memcpyb size:0 argc:1
        [
            Register = MEM_COPY_BLOCK_REG
        ],
    memcpyb1 size:1 argc:1
        [
            Register = MEM_COPY_BLOCK_REG_SIZED,
            AddressInRegister = MEM_COPY_BLOCK_ADDR_IN_REG,
            Number = MEM_COPY_BLOCK_CONST,
            AddressLiteral = MEM_COPY_BLOCK_ADDR_LITERAL,
            AddressAtLabel = MEM_COPY_BLOCK_ADDR_LITERAL,
            Label = MEM_COPY_BLOCK_CONST
        ],
    memcpyb2 size:2 argc:1
        [
            Register = MEM_COPY_BLOCK_REG_SIZED,
            AddressInRegister = MEM_COPY_BLOCK_ADDR_IN_REG,
            Number = MEM_COPY_BLOCK_CONST,
            AddressLiteral = MEM_COPY_BLOCK_ADDR_LITERAL,
            AddressAtLabel = MEM_COPY_BLOCK_ADDR_LITERAL,
            Label = MEM_COPY_BLOCK_CONST
        ],
    memcpyb4 size:4 argc:1
        [
            Register = MEM_COPY_BLOCK_REG_SIZED,
            AddressInRegister = MEM_COPY_BLOCK_ADDR_IN_REG,
            Number = MEM_COPY_BLOCK_CONST,
            AddressLiteral = MEM_COPY_BLOCK_ADDR_LITERAL,
            AddressAtLabel = MEM_COPY_BLOCK_ADDR_LITERAL,
            Label = MEM_COPY_BLOCK_CONST
        ],
    memcpyb8 size:8 argc:1
        [
            Register = MEM_COPY_BLOCK_REG_SIZED,
            AddressInRegister = MEM_COPY_BLOCK_ADDR_IN_REG,
            Number = MEM_COPY_BLOCK_CONST,
            AddressLiteral = MEM_COPY_BLOCK_ADDR_LITERAL,
            AddressAtLabel = MEM_COPY_BLOCK_ADDR_LITERAL,
            Label = MEM_COPY_BLOCK_CONST
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
            Register = POP_STACK_POINTER_REG_SIZED,
            AddressInRegister = POP_STACK_POINTER_ADDR_IN_REG,
            Number = POP_STACK_POINTER_CONST,
            AddressLiteral = POP_STACK_POINTER_ADDR_LITERAL,
            AddressAtLabel = POP_STACK_POINTER_ADDR_LITERAL
        ],
    popsp2 size:2 argc:1
        [
            Register = POP_STACK_POINTER_REG_SIZED,
            AddressInRegister = POP_STACK_POINTER_ADDR_IN_REG,
            Number = POP_STACK_POINTER_CONST,
            AddressLiteral = POP_STACK_POINTER_ADDR_LITERAL,
            AddressAtLabel = POP_STACK_POINTER_ADDR_LITERAL
        ],
    popsp4 size:4 argc:1
        [
            Register = POP_STACK_POINTER_REG_SIZED,
            AddressInRegister = POP_STACK_POINTER_ADDR_IN_REG,
            Number = POP_STACK_POINTER_CONST,
            AddressLiteral = POP_STACK_POINTER_ADDR_LITERAL,
            AddressAtLabel = POP_STACK_POINTER_ADDR_LITERAL
        ],
    popsp8 size:8 argc:1
        [
            Register = POP_STACK_POINTER_REG_SIZED,
            AddressInRegister = POP_STACK_POINTER_ADDR_IN_REG,
            Number = POP_STACK_POINTER_CONST,
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
            Register = CALL_REG,
            Label = CALL_CONST
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
    swpe size:0 argc:0 = SWAP_BYTES_ENDIANNESS,
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

    DefineNumber  "dn"         { size: NumberSize, number: Number },
    DefineBytes   "db"         { bytes: Box<[u8]> },
    DefineString  "ds"         { string: Cow<'a, str> },
    DefineCString "dcs"        { string: Cow<'a, str> },
    DefineArray   "da"         { array: ArrayData },
    OffsetFrom    "offsetfrom" { label: &'a str },
    PrintString   "printstr"   { string: Cow<'a, str> }

}


/// Represents the number of bytes needed to represent a number
#[repr(u8)]
#[derive(Debug, Clone, Copy)]
// These variants are never directly constructed, but they are created as the result of a transmute
#[allow(dead_code)]
pub enum NumberSize {

    B1 = 1,
    B2 = 2,
    B4 = 4,
    B8 = 8

}

impl NumberSize {

    pub fn new(n: u64) -> Option<Self> {
        match n {

            1 | 2 | 4 | 8
            => Some( unsafe {
                mem::transmute::<u8, Self>(n as u8)
            }),

            _ => None
        }
    }


    pub fn as_usize(self) -> usize {
        self as usize
    }

}


/// It's the programmer's responsibility to correctly instantiate the struct with matching array data and element type
#[derive(Debug)]
pub struct ArrayData {

    pub array: Box<[PrimitiveData]>,
    pub element_type: DataType

}

impl ArrayData {

    pub fn to_le_bytes(&self) -> Box<[u8]> {
        // Assume the array elements and data type match

        let size = self.element_type.size()*self.array.len();
        let mut bytes = Vec::with_capacity(size);

        self.append_bytes_to(&mut bytes);

        bytes.into_boxed_slice()
    }


    fn append_bytes_to(&self, buf: &mut Vec<u8>) {

        for elem in &self.array {

            match elem {

                PrimitiveData::Number(number)
                => buf.extend(
                    &number.as_bytes()[..self.element_type.size()]
                ),

                PrimitiveData::Array(array)
                    => array.append_bytes_to(buf)
            }

        }
    }

}


#[derive(Debug)]
pub enum PrimitiveData {

    Number (Number),
    Array (ArrayData)

}


#[derive(Debug, Clone)]
pub enum DataType {

    Int { size: NumberSize },
    Uint { size: NumberSize },
    Float { size: NumberSize },
    Array { element_type: Box<DataType>, len: usize }

}

impl DataType {

    pub fn size(&self) -> usize {
        match self {

            DataType::Int { size } |
            DataType::Uint { size } |
            DataType::Float { size }
                => size.as_usize(),

            DataType::Array { element_type, len }
                => element_type.size() * len
        }
    }


    pub fn from_name_not_array(name: &str) -> Option<Self> {
        match name {

            "u8" => Some(DataType::Uint { size: NumberSize::B1 }),
            "u16" => Some(DataType::Uint { size: NumberSize::B2 }),
            "u32" => Some(DataType::Uint { size: NumberSize::B4 }),
            "u64" => Some(DataType::Uint { size: NumberSize::B8 }),
            "i8" => Some(DataType::Int { size: NumberSize::B1 }),
            "i16" => Some(DataType::Int { size: NumberSize::B2 }),
            "i32" => Some(DataType::Int { size: NumberSize::B4 }),
            "i64" => Some(DataType::Int { size: NumberSize::B8 }),
            "f64" => Some(DataType::Float { size: NumberSize::B8 }),

            _ => None
        }
    }

}

impl fmt::Display for DataType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {

            Self::Int { size }
                => write!(f, "i{}", size.as_usize()),

            Self::Uint { size }
                => write!(f, "u{}", size.as_usize()),

            Self::Float { size }
                => write!(f, "f{}", size.as_usize()),

            Self::Array { element_type, len }
                => write!(f, "[{}: {}]", element_type, len)
        }
    }
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
            Number::SignedInt(_) => 8, // Conservative approach.
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


    pub fn uint_from_bytes(bytes: &[u8]) -> Result<Self, String> {
        // TODO: could be made in a more efficient way, eventually.
        match bytes.len() {
            1 => Ok(Self::UnsignedInt(bytes[0] as u64)),
            2 => Ok(Self::UnsignedInt(u16::from_le_bytes([bytes[0], bytes[1]]) as u64)),
            4 => Ok(Self::UnsignedInt(u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]) as u64)),
            8 => Ok(Self::UnsignedInt(u64::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7]]))),
            n => Err(format!("Cannot construct a uint from buffer {:?} of {n} bytes", bytes))
        }
    }

}

impl fmt::Display for Number {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Number::SignedInt(i) => write!(f, "{i}"),
            Number::UnsignedInt(u) => write!(f, "{u}"),
            Number::Float(fl) => write!(f, "{fl}"),
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

impl fmt::Display for AsmValue<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AsmValue::Register(reg) => write!(f, "{}", reg.name()),
            AsmValue::AddressInRegister(reg) => write!(f, "[{}]", reg.name()),
            AsmValue::Number(n) => write!(f, "{}", n),
            AsmValue::AddressLiteral(addr) => write!(f, "[{addr}]"),
            AsmValue::AddressAtLabel(label) => write!(f, "[{label}]"),
            AsmValue::Label(label) => write!(f, "{label}"),
        }
    }
}


#[derive(Debug, Clone)]
pub struct AsmOperand<'a> {

    pub value: AsmValue<'a>,
    pub source: Rc<SourceToken<'a>>

}


/// A canonicalized absolute path
#[derive(Hash, Eq, PartialEq, Debug, Clone, Copy)]
pub struct UnitPath<'a> {

    path: &'a Path

}

impl UnitPath<'_> {

    /// Construct a new `UnitPath` from a canonicalized path.
    /// This function assumes that `path` is correctly canonicalized.
    pub fn new_canonicalized(path: &Path) -> UnitPath<'_> {
        UnitPath {
            path
        }
    }

    pub fn as_path(&self) -> &Path {
        self.path
    }

}

impl fmt::Display for UnitPath<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.path.display())
    }
}


#[derive(Debug)]
pub struct SourceToken<'a> {

    pub string: &'a str,
    pub line_index: usize,
    pub column: usize,
    pub unit_path: UnitPath<'a>

}

impl SourceToken<'_> {

    #[inline]
    pub fn line_number(&self) -> usize {
        self.line_index + 1
    }

}
