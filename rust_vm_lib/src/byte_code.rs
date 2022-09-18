use std::fmt;
use std::convert::TryFrom;


// Max is 255
pub const BYTE_CODE_COUNT: usize = 45;


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
    "JUMP_IF_TRUE_REG",
    "JUMP_IF_FALSE_REG",

    "COMPARE_REG_REG",
    "COMPARE_REG_CONST",
    "COMPARE_CONST_REG",
    "COMPARE_CONST_CONST",

    "PRINT",
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
    JUMP_IF_TRUE_REG,
    JUMP_IF_FALSE_REG,

    COMPARE_REG_REG,
    COMPARE_REG_CONST,
    COMPARE_CONST_REG,
    COMPARE_CONST_CONST,

    PRINT,
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


const BYTE_CODE_FROM_U8_TABLE: [ByteCodes; BYTE_CODE_COUNT] = [
    ByteCodes::ADD,
    ByteCodes::SUB,
    ByteCodes::MUL,
    ByteCodes::DIV,
    ByteCodes::MOD,

    ByteCodes::INC_REG,
    ByteCodes::INC_ADDR_IN_REG,
    ByteCodes::INC_ADDR_LITERAL,

    ByteCodes::DEC_REG,
    ByteCodes::DEC_ADDR_IN_REG,
    ByteCodes::DEC_ADDR_LITERAL,

    ByteCodes::NO_OPERATION,

    ByteCodes::MOVE_INTO_REG_FROM_REG,
    ByteCodes::MOVE_INTO_REG_FROM_ADDR_IN_REG,
    ByteCodes::MOVE_INTO_REG_FROM_CONST,
    ByteCodes::MOVE_INTO_REG_FROM_ADDR_LITERAL,
    ByteCodes::MOVE_INTO_ADDR_IN_REG_FROM_REG,
    ByteCodes::MOVE_INTO_ADDR_IN_REG_FROM_ADDR_IN_REG,
    ByteCodes::MOVE_INTO_ADDR_IN_REG_FROM_CONST,
    ByteCodes::MOVE_INTO_ADDR_IN_REG_FROM_ADDR_LITERAL,
    ByteCodes::MOVE_INTO_ADDR_LITERAL_FROM_REG,
    ByteCodes::MOVE_INTO_ADDR_LITERAL_FROM_ADDR_IN_REG,
    ByteCodes::MOVE_INTO_ADDR_LITERAL_FROM_CONST,
    ByteCodes::MOVE_INTO_ADDR_LITERAL_FROM_ADDR_LITERAL,

    ByteCodes::PUSH_FROM_REG,
    ByteCodes::PUSH_FROM_ADDR_IN_REG,
    ByteCodes::PUSH_FROM_CONST,
    ByteCodes::PUSH_FROM_ADDR_LITERAL,

    ByteCodes::POP_INTO_REG,
    ByteCodes::POP_INTO_ADDR_IN_REG,
    ByteCodes::POP_INTO_ADDR_LITERAL,

    ByteCodes::LABEL,

    ByteCodes::JUMP,
    ByteCodes::JUMP_IF_TRUE_REG,
    ByteCodes::JUMP_IF_FALSE_REG,

    ByteCodes::COMPARE_REG_REG,
    ByteCodes::COMPARE_REG_CONST,
    ByteCodes::COMPARE_CONST_REG,
    ByteCodes::COMPARE_CONST_CONST,

    ByteCodes::PRINT,
    ByteCodes::PRINT_CHAR,
    ByteCodes::PRINT_STRING,

    ByteCodes::INPUT_INT,
    ByteCodes::INPUT_STRING,

    ByteCodes::EXIT
];


impl TryFrom<u8> for ByteCodes {
    type Error = &'static str;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        if value < BYTE_CODE_FROM_U8_TABLE.len() as u8 {
            Ok(BYTE_CODE_FROM_U8_TABLE[value as usize])
        } else {
            Err("Invalid byte code")
        }
    }
}


pub fn is_jump_instruction(instruction: ByteCodes) -> bool {
    ByteCodes::JUMP as usize <= instruction as usize && instruction as usize <= ByteCodes::JUMP_IF_FALSE_REG as usize
}

