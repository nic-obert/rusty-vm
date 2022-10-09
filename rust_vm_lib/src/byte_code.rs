use std::fmt;


// Max is 255
pub const BYTE_CODE_COUNT: usize = 46;


pub const BYTE_CODE_NAMES: [&str; BYTE_CODE_COUNT] = [
    "ADD",
    "SUB",
    "MUL",
    "DIV",
    "MOD",

    "INC_REG",
    "INC_ADDR_IN_REG",
    "INC_ADDR_LITERAL",

    "DEC_REG",
    "DEC_ADDR_IN_REG",
    "DEC_ADDR_LITERAL",

    "NO_OPERATION",

    "MOVE_REG_REG",
    "MOVE_REG_ADDR_IN_REG",
    "MOVE_REG_CONST",
    "MOVE_REG_ADDR_LITERAL",
    "MOVE_ADDR_IN_REG_REG",
    "MOVE_ADDR_IN_REG_ADDR_IN_REG",
    "MOVE_ADDR_IN_REG_CONST",
    "MOVE_ADDR_IN_REG_ADDR_LITERAL",
    "MOVE_ADDR_LITERAL_REG",
    "MOVE_ADDR_LITERAL_ADDR_IN_REG",   
    "MOVE_ADDR_LITERAL_CONST",
    "MOVE_ADDR_LITERAL_ADDR_LITERAL",

    "PUSH_REG",
    "PUSH_ADDR_IN_REG",
    "PUSH_CONST",
    "PUSH_ADDR_LITERAL",

    "POP_REG",
    "POP_ADDR_IN_REG",
    "POP_ADDR_LITERAL",

    "LABEL",

    "JUMP",
    "JUMP_IF_NOT_ZERO_REG",
    "JUMP_IF_ZERO_REG",

    "COMPARE_REG_REG",
    "COMPARE_REG_CONST",
    "COMPARE_CONST_REG",
    "COMPARE_CONST_CONST",

    "PRINT_SIGNED",
    "PRINT_UNSIGNED",
    "PRINT_CHAR",
    "PRINT_STRING",

    "INPUT_INT",
    "INPUT_STRING",

    "EXIT"
];


#[derive(Debug, Clone, Copy)]
#[allow(dead_code, non_camel_case_types)]
pub enum ByteCodes {
    ADD = 0,
    SUB,
    MUL,
    DIV,
    MOD,

    INC_REG,
    INC_ADDR_IN_REG,
    INC_ADDR_LITERAL,

    DEC_REG,
    DEC_ADDR_IN_REG,
    DEC_ADDR_LITERAL,

    NO_OPERATION,

    MOVE_INTO_REG_FROM_REG,
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

    PUSH_FROM_REG,
    PUSH_FROM_ADDR_IN_REG,
    PUSH_FROM_CONST,
    PUSH_FROM_ADDR_LITERAL,

    POP_INTO_REG,
    POP_INTO_ADDR_IN_REG,
    POP_INTO_ADDR_LITERAL,

    LABEL,

    JUMP,
    JUMP_IF_NOT_ZERO_REG,
    JUMP_IF_ZERO_REG,

    COMPARE_REG_REG,
    COMPARE_REG_CONST,
    COMPARE_CONST_REG,
    COMPARE_CONST_CONST,

    PRINT_SIGNED,
    PRINT_UNSIGNED,
    PRINT_CHAR,
    PRINT_STRING,

    INPUT_INT,
    INPUT_STRING,

    EXIT,
}


impl fmt::Display for ByteCodes {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", BYTE_CODE_NAMES[*self as usize])
    }
}


impl std::convert::From<u8> for ByteCodes {

    fn from(value: u8) -> Self {
        if value < BYTE_CODE_COUNT as u8 {
            unsafe { std::mem::transmute(value) }
        } else {
            panic!("Invalid byte code: {}", value);
        }
    }
}


pub fn is_jump_instruction(instruction: ByteCodes) -> bool {
    ByteCodes::JUMP as usize <= instruction as usize && instruction as usize <= ByteCodes::JUMP_IF_ZERO_REG as usize
}

