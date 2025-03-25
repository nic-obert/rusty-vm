use std::mem;
use std::fmt;


macro_rules! declare_bytecodes {
    ($($name:ident),+) => {

/// Represents the byte code instruction set
#[derive(Debug, Clone, Copy)]
#[allow(dead_code, non_camel_case_types)]
#[repr(u8)]
pub enum ByteCodes {
    $($name),+
}

impl ByteCodes {

    /// Get the name of the instruction.
    pub fn name(&self) -> &'static str {
        match self {
            $(Self::$name => stringify!($name),)+
        }
    }

}

    };
}


declare_bytecodes! {

    INTEGER_ADD,
    INTEGER_SUB,
    INTEGER_MUL,
    INTEGER_DIV,
    INTEGER_MOD,

    FLOAT_ADD,
    FLOAT_SUB,
    FLOAT_MUL,
    FLOAT_DIV,
    FLOAT_MOD,

    INC_REG,
    INC_ADDR_IN_REG,
    INC_ADDR_LITERAL,

    DEC_REG,
    DEC_ADDR_IN_REG,
    DEC_ADDR_LITERAL,

    NO_OPERATION,

    MOVE_INTO_REG_FROM_REG,
    MOVE_INTO_REG_FROM_REG_SIZED,
    MOVE_INTO_REG_FROM_ADDR_IN_REG,
    MOVE_INTO_REG_FROM_CONST,
    MOVE_INTO_REG_FROM_ADDR_LITERAL,
    MOVE_INTO_ADDR_IN_REG_FROM_REG,
    MOVE_INTO_ADDR_IN_REG_FROM_ADDR_IN_REG,
    MOVE_INTO_ADDR_IN_REG_FROM_CONST,
    MOVE_INTO_ADDR_IN_REG_FROM_ADDR_LITERAL,
    MOVE_INTO_ADDR_LITERAL_FROM_REG,
    MOVE_INTO_ADDR_LITERAL_FROM_ADDR_IN_REG,
    MOVE_INTO_ADDR_LITERAL_FROM_CONST,
    MOVE_INTO_ADDR_LITERAL_FROM_ADDR_LITERAL,

    MEM_COPY_BLOCK_REG,
    MEM_COPY_BLOCK_REG_SIZED,
    MEM_COPY_BLOCK_ADDR_IN_REG,
    MEM_COPY_BLOCK_CONST,
    MEM_COPY_BLOCK_ADDR_LITERAL,

    PUSH_FROM_REG,
    PUSH_FROM_REG_SIZED,
    PUSH_FROM_ADDR_IN_REG,
    PUSH_FROM_CONST,
    PUSH_FROM_ADDR_LITERAL,

    PUSH_STACK_POINTER_REG,
    PUSH_STACK_POINTER_REG_SIZED,
    PUSH_STACK_POINTER_ADDR_IN_REG,
    PUSH_STACK_POINTER_CONST,
    PUSH_STACK_POINTER_ADDR_LITERAL,

    POP_INTO_REG,
    POP_INTO_ADDR_IN_REG,
    POP_INTO_ADDR_LITERAL,

    POP_STACK_POINTER_REG,
    POP_STACK_POINTER_REG_SIZED,
    POP_STACK_POINTER_ADDR_IN_REG,
    POP_STACK_POINTER_CONST,
    POP_STACK_POINTER_ADDR_LITERAL,

    JUMP,
    JUMP_NOT_ZERO,
    JUMP_ZERO,
    JUMP_GREATER,
    JUMP_LESS,
    JUMP_GREATER_OR_EQUAL,
    JUMP_LESS_OR_EQUAL,
    JUMP_CARRY,
    JUMP_NOT_CARRY,
    JUMP_OVERFLOW,
    JUMP_NOT_OVERFLOW,
    JUMP_SIGN,
    JUMP_NOT_SIGN,

    CALL_CONST,
    CALL_REG,
    RETURN,

    COMPARE_REG_REG,
    COMPARE_REG_REG_SIZED,
    COMPARE_REG_ADDR_IN_REG,
    COMPARE_REG_CONST,
    COMPARE_REG_ADDR_LITERAL,
    COMPARE_ADDR_IN_REG_REG,
    COMPARE_ADDR_IN_REG_ADDR_IN_REG,
    COMPARE_ADDR_IN_REG_CONST,
    COMPARE_ADDR_IN_REG_ADDR_LITERAL,
    COMPARE_CONST_REG,
    COMPARE_CONST_ADDR_IN_REG,
    COMPARE_CONST_CONST,
    COMPARE_CONST_ADDR_LITERAL,
    COMPARE_ADDR_LITERAL_REG,
    COMPARE_ADDR_LITERAL_ADDR_IN_REG,
    COMPARE_ADDR_LITERAL_CONST,
    COMPARE_ADDR_LITERAL_ADDR_LITERAL,

    AND,
    OR,
    XOR,
    NOT,
    SHIFT_LEFT,
    SHIFT_RIGHT,
    SWAP_BYTES_ENDIANNESS,

    INTERRUPT,
    BREAKPOINT,

    EXIT

}


pub const BYTE_CODE_COUNT: usize = mem::variant_count::<ByteCodes>();
pub const OPCODE_SIZE: usize = mem::size_of::<ByteCodes>();


impl fmt::Display for ByteCodes {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}


impl std::convert::From<u8> for ByteCodes {
    fn from(value: u8) -> Self {
        if value < BYTE_CODE_COUNT as u8 {
            unsafe { std::mem::transmute::<u8, ByteCodes>(value) }
        } else {
            panic!("Invalid byte code: {}", value);
        }
    }
}


/// Return whether the given instruction is a jump instruction
pub fn is_jump_instruction(instruction: ByteCodes) -> bool {
    ByteCodes::JUMP as usize <= instruction as usize && instruction as usize <= ByteCodes::RETURN as usize
}
