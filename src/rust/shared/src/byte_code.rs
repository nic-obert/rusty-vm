use std::fmt;


pub static BYTE_CODE_NAMES: [&str; 44] = [
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
    "PRINT_STRING",

    "INPUT_INT",
    "INPUT_STRING",

    "EXIT"
];


#[derive(Clone, Copy)]
pub enum ByteCodes {
    ADD,
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

    MOVE_REG_REG,
    MOVE_REG_ADDR_IN_REG,
    MOVE_REG_CONST,
    MOVE_REG_ADDR_LITERAL,
    MOVE_ADDR_IN_REG_REG,
    MOVE_ADDR_IN_REG_ADDR_IN_REG,
    MOVE_ADDR_IN_REG_CONST,
    MOVE_ADDR_IN_REG_ADDR_LITERAL,
    MOVE_ADDR_LITERAL_REG,
    MOVE_ADDR_LITERAL_ADDR_IN_REG,   
    MOVE_ADDR_LITERAL_CONST,
    MOVE_ADDR_LITERAL_ADDR_LITERAL,

    PUSH_REG,
    PUSH_ADDR_IN_REG,
    PUSH_CONST,
    PUSH_ADDR_LITERAL,

    POP_REG,
    POP_ADDR_IN_REG,
    POP_ADDR_LITERAL,

    LABEL,

    JUMP,
    JUMP_IF_TRUE_REG,
    JUMP_IF_FALSE_REG,

    COMPARE_REG_REG,
    COMPARE_REG_CONST,
    COMPARE_CONST_REG,
    COMPARE_CONST_CONST,

    PRINT,
    PRINT_STRING,

    INPUT_INT,
    INPUT_STRING,

    EXIT,
}


// Implement printable for ByteCodes
impl fmt::Display for ByteCodes {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", BYTE_CODE_NAMES[*self as usize])
    }
}


pub fn is_jump_instruction(instruction: ByteCodes) -> bool {
    ByteCodes::JUMP as usize <= instruction as usize && instruction as usize <= ByteCodes::JUMP_IF_FALSE_REG as usize
}


