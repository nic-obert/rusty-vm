use std::fmt;
use std::convert::TryFrom;


pub const BYTE_CODE_NAMES: [&str; 44] = [
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


#[derive(Debug, Clone, Copy)]
#[allow(dead_code, non_camel_case_types)]
pub enum ByteCodes {
    ADD = 0,
    SUB = 1,
    MUL = 2,
    DIV = 3,
    MOD = 4,

    INC_REG = 5,
    INC_ADDR_IN_REG = 6,
    INC_ADDR_LITERAL = 7,

    DEC_REG = 8,
    DEC_ADDR_IN_REG = 9,
    DEC_ADDR_LITERAL = 10,

    NO_OPERATION = 11,

    MOVE_INTO_REG_FROM_REG = 12,
    MOVE_INTO_REG_FROM_ADDR_IN_REG = 13,
    MOVE_INTO_REG_FROM_CONST = 14,
    MOVE_INTO_REG_FROM_ADDR_LITERAL = 15,
    MOVE_INTO_ADDR_IN_REG_FROM_REG = 16,
    MOVE_INTO_ADDR_IN_REG_FROM_ADDR_IN_REG = 17,
    MOVE_INTO_ADDR_IN_REG_FROM_CONST = 18,
    MOVE_INTO_ADDR_IN_REG_FROM_ADDR_LITERAL = 19,
    MOVE_INTO_ADDR_LITERAL_FROM_REG = 20,
    MOVE_INTO_ADDR_LITERAL_FROM_ADDR_IN_REG = 21,
    MOVE_INTO_ADDR_LITERAL_FROM_CONST = 22,
    MOVE_INTO_ADDR_LITERAL_FROM_ADDR_LITERAL = 23,

    PUSH_FROM_REG = 24,
    PUSH_FROM_ADDR_IN_REG = 25,
    PUSH_FROM_CONST = 26,
    PUSH_FROM_ADDR_LITERAL = 27,

    POP_INTO_REG = 28,
    POP_INTO_ADDR_IN_REG = 29,
    POP_INTO_ADDR_LITERAL = 30,

    LABEL = 31,

    JUMP = 32,
    JUMP_IF_TRUE_REG = 33,
    JUMP_IF_FALSE_REG = 34,

    COMPARE_REG_REG = 35,
    COMPARE_REG_CONST = 36,
    COMPARE_CONST_REG = 37,
    COMPARE_CONST_CONST = 38,

    PRINT = 39,
    PRINT_STRING = 40,

    INPUT_INT = 41,
    INPUT_STRING = 42,

    EXIT = 43,
}


impl fmt::Display for ByteCodes {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", BYTE_CODE_NAMES[*self as usize])
    }
}


impl TryFrom<u8> for ByteCodes {
    type Error = &'static str;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(ByteCodes::ADD),
            1 => Ok(ByteCodes::SUB),
            2 => Ok(ByteCodes::MUL),
            3 => Ok(ByteCodes::DIV),
            4 => Ok(ByteCodes::MOD),

            5 => Ok(ByteCodes::INC_REG),
            6 => Ok(ByteCodes::INC_ADDR_IN_REG),
            7 => Ok(ByteCodes::INC_ADDR_LITERAL),

            8 => Ok(ByteCodes::DEC_REG),
            9 => Ok(ByteCodes::DEC_ADDR_IN_REG),
            10 => Ok(ByteCodes::DEC_ADDR_LITERAL),

            11 => Ok(ByteCodes::NO_OPERATION),

            12 => Ok(ByteCodes::MOVE_INTO_REG_FROM_REG),
            13 => Ok(ByteCodes::MOVE_INTO_REG_FROM_ADDR_IN_REG),
            14 => Ok(ByteCodes::MOVE_INTO_REG_FROM_CONST),
            15 => Ok(ByteCodes::MOVE_INTO_REG_FROM_ADDR_LITERAL),
            16 => Ok(ByteCodes::MOVE_INTO_ADDR_IN_REG_FROM_REG),
            17 => Ok(ByteCodes::MOVE_INTO_ADDR_IN_REG_FROM_ADDR_IN_REG),
            18 => Ok(ByteCodes::MOVE_INTO_ADDR_IN_REG_FROM_CONST),
            19 => Ok(ByteCodes::MOVE_INTO_ADDR_IN_REG_FROM_ADDR_LITERAL),
            20 => Ok(ByteCodes::MOVE_INTO_ADDR_LITERAL_FROM_REG),
            21 => Ok(ByteCodes::MOVE_INTO_ADDR_LITERAL_FROM_ADDR_IN_REG),
            22 => Ok(ByteCodes::MOVE_INTO_ADDR_LITERAL_FROM_CONST),
            23 => Ok(ByteCodes::MOVE_INTO_ADDR_LITERAL_FROM_ADDR_LITERAL),

            24 => Ok(ByteCodes::PUSH_FROM_REG),
            25 => Ok(ByteCodes::PUSH_FROM_ADDR_IN_REG),
            26 => Ok(ByteCodes::PUSH_FROM_CONST),
            27 => Ok(ByteCodes::PUSH_FROM_ADDR_LITERAL),

            28 => Ok(ByteCodes::POP_INTO_REG),
            29 => Ok(ByteCodes::POP_INTO_ADDR_IN_REG),
            30 => Ok(ByteCodes::POP_INTO_ADDR_LITERAL),

            31 => Ok(ByteCodes::LABEL),

            32 => Ok(ByteCodes::JUMP),
            33 => Ok(ByteCodes::JUMP_IF_TRUE_REG),
            34 => Ok(ByteCodes::JUMP_IF_FALSE_REG),

            35 => Ok(ByteCodes::COMPARE_REG_REG),
            36 => Ok(ByteCodes::COMPARE_REG_CONST),
            37 => Ok(ByteCodes::COMPARE_CONST_REG),
            38 => Ok(ByteCodes::COMPARE_CONST_CONST),

            39 => Ok(ByteCodes::PRINT),
            40 => Ok(ByteCodes::PRINT_STRING),

            41 => Ok(ByteCodes::INPUT_INT),
            42 => Ok(ByteCodes::INPUT_STRING),

            43 => Ok(ByteCodes::EXIT),

            _ => Err("Invalid ByteCode"),
        }
    }
}


pub fn is_jump_instruction(instruction: ByteCodes) -> bool {
    ByteCodes::JUMP as usize <= instruction as usize && instruction as usize <= ByteCodes::JUMP_IF_FALSE_REG as usize
}

