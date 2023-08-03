use rust_vm_lib::byte_code::ByteCodes;
use std::collections::HashMap;
use lazy_static::lazy_static;


/// A pair of operation code and handled size
type Operation = (ByteCodes, u8);
type OneArgument = Vec<Option<Operation>>;
type TwoArguments = Vec<Option<OneArgument>>;


pub enum Args {
    Zero(Operation),
    One(OneArgument),
    Two(TwoArguments),
}


lazy_static! {

pub static ref ARGUMENTS_TABLE: HashMap<&'static str, Args> = HashMap::from([

    // Arithmetic

    ("add", Args::Zero((ByteCodes::ADD, 0))), // No arguments

    ("sub", Args::Zero((ByteCodes::SUB, 0))), // No arguments

    ("mul", Args::Zero((ByteCodes::MUL, 0))), // No arguments

    ("div", Args::Zero((ByteCodes::DIV, 0))), // No arguments

    ("mod", Args::Zero((ByteCodes::MOD, 0))), // No arguments

    ("inc", Args::One(vec![ 
        Some((ByteCodes::INC_REG, 0)), // Register
    ])), 
  
    ("inc1", Args::One(vec![
        None, // Register
        Some((ByteCodes::INC_ADDR_IN_REG, 1)), // Address in register
        None, // Constant
        Some((ByteCodes::INC_ADDR_LITERAL, 1)), // Address literal
    ])),
    
    ("inc2", Args::One(vec![
        None, // Register
        Some((ByteCodes::INC_ADDR_IN_REG, 2)), // Address in register
        None, // Constant
        Some((ByteCodes::INC_ADDR_LITERAL, 2)), // Address literal
    ])),

    ("inc4", Args::One(vec![
        None, // Register
        Some((ByteCodes::INC_ADDR_IN_REG, 4)), // Address in register
        None, // Constant
        Some((ByteCodes::INC_ADDR_LITERAL, 4)), // Address literal
    ])),
 
    ("inc8", Args::One(vec![
        None, // Register
        Some((ByteCodes::INC_ADDR_IN_REG, 8)), // Address in register
        None, // Constant
        Some((ByteCodes::INC_ADDR_LITERAL, 8)), // Address literal
    ])),

    ("dec", Args::One(vec![
        Some((ByteCodes::DEC_REG, 0)), // Register
    ])),
    
    ("dec1", Args::One(vec![
        None, // Register
        Some((ByteCodes::DEC_ADDR_IN_REG, 1)), // Address in register
        None, // Constant
        Some((ByteCodes::DEC_ADDR_LITERAL, 1)), // Address literal
    ])),
    
    ("dec2", Args::One(vec![
        None, // Register
        Some((ByteCodes::DEC_ADDR_IN_REG, 2)), // Address in register
        None, // Constant
        Some((ByteCodes::DEC_ADDR_LITERAL, 2)), // Address literal
    ])),
    
    ("dec4", Args::One(vec![
        None, // Register
        Some((ByteCodes::DEC_ADDR_IN_REG, 4)), // Address in register
        None, // Constant
        Some((ByteCodes::DEC_ADDR_LITERAL, 4)), // Address literal
    ])),
    
    ("dec8", Args::One(vec![
        None, // Register
        Some((ByteCodes::DEC_ADDR_IN_REG, 8)), // Address in register
        None, // Constant
        Some((ByteCodes::DEC_ADDR_LITERAL, 8)), // Address literal
    ])),
   
    // No operation

    ("nop", Args::Zero((ByteCodes::NO_OPERATION, 0))), // No arguments

    // Memory

    ("mov", Args::Two(vec![
        // Register
        Some(vec![
            Some((ByteCodes::MOVE_INTO_REG_FROM_REG, 0)), // Register
        ])
    ])),

    ("mov1", Args::Two(vec![
        // Register
        Some(vec![
            None, // Register
            Some((ByteCodes::MOVE_INTO_REG_FROM_ADDR_IN_REG, 1)), // Address in register
            Some((ByteCodes::MOVE_INTO_REG_FROM_CONST, 1)), // Constant
            Some((ByteCodes::MOVE_INTO_REG_FROM_ADDR_LITERAL, 1)), // Address literal
        ]),
        // Address in register
        Some(vec![
            Some((ByteCodes::MOVE_INTO_ADDR_IN_REG_FROM_REG, 1)), // Register
            Some((ByteCodes::MOVE_INTO_ADDR_IN_REG_FROM_ADDR_IN_REG, 1)), // Address in register
            Some((ByteCodes::MOVE_INTO_ADDR_IN_REG_FROM_CONST, 1)), // Constant
            Some((ByteCodes::MOVE_INTO_ADDR_IN_REG_FROM_ADDR_LITERAL, 1)), // Address literal
        ]),
        None, // Constant
        // Address literal
        Some(vec![
            Some((ByteCodes::MOVE_INTO_ADDR_LITERAL_FROM_REG, 1)), // Register
            Some((ByteCodes::MOVE_INTO_ADDR_LITERAL_FROM_ADDR_IN_REG, 1)), // Address in register
            Some((ByteCodes::MOVE_INTO_ADDR_LITERAL_FROM_CONST, 1)), // Constant
            Some((ByteCodes::MOVE_INTO_ADDR_LITERAL_FROM_ADDR_LITERAL, 1)), // Address literal
        ]),
    ])),

    ("mov2", Args::Two(vec![
        // Register
        Some(vec![
            None, // Register
            Some((ByteCodes::MOVE_INTO_REG_FROM_ADDR_IN_REG, 2)), // Address in register
            Some((ByteCodes::MOVE_INTO_REG_FROM_CONST, 2)), // Constant
            Some((ByteCodes::MOVE_INTO_REG_FROM_ADDR_LITERAL, 2)), // Address literal
        ]),
        // Address in register
        Some(vec![
            Some((ByteCodes::MOVE_INTO_ADDR_IN_REG_FROM_REG, 2)), // Register
            Some((ByteCodes::MOVE_INTO_ADDR_IN_REG_FROM_ADDR_IN_REG, 2)), // Address in register
            Some((ByteCodes::MOVE_INTO_ADDR_IN_REG_FROM_CONST, 2)), // Constant
            Some((ByteCodes::MOVE_INTO_ADDR_IN_REG_FROM_ADDR_LITERAL, 2)), // Address literal
        ]),
        None, // Constant
        // Address literal
        Some(vec![
            Some((ByteCodes::MOVE_INTO_ADDR_LITERAL_FROM_REG, 2)), // Register
            Some((ByteCodes::MOVE_INTO_ADDR_LITERAL_FROM_ADDR_IN_REG, 2)), // Address in register
            Some((ByteCodes::MOVE_INTO_ADDR_LITERAL_FROM_CONST, 2)), // Constant
            Some((ByteCodes::MOVE_INTO_ADDR_LITERAL_FROM_ADDR_LITERAL, 2)), // Address literal
        ]),
    ])),
    
    ("mov4", Args::Two(vec![
        // Register
        Some(vec![
            None, // Register
            Some((ByteCodes::MOVE_INTO_REG_FROM_ADDR_IN_REG, 4)), // Address in register
            Some((ByteCodes::MOVE_INTO_REG_FROM_CONST, 4)), // Constant
            Some((ByteCodes::MOVE_INTO_REG_FROM_ADDR_LITERAL, 4)), // Address literal
        ]),
        // Address in register
        Some(vec![
            Some((ByteCodes::MOVE_INTO_ADDR_IN_REG_FROM_REG, 4)), // Register
            Some((ByteCodes::MOVE_INTO_ADDR_IN_REG_FROM_ADDR_IN_REG, 4)), // Address in register
            Some((ByteCodes::MOVE_INTO_ADDR_IN_REG_FROM_CONST, 4)), // Constant
            Some((ByteCodes::MOVE_INTO_ADDR_IN_REG_FROM_ADDR_LITERAL, 4)), // Address literal
        ]),
        None, // Constant
        // Address literal
        Some(vec![
            Some((ByteCodes::MOVE_INTO_ADDR_LITERAL_FROM_REG, 4)), // Register
            Some((ByteCodes::MOVE_INTO_ADDR_LITERAL_FROM_ADDR_IN_REG, 4)), // Address in register
            Some((ByteCodes::MOVE_INTO_ADDR_LITERAL_FROM_CONST, 4)), // Constant
            Some((ByteCodes::MOVE_INTO_ADDR_LITERAL_FROM_ADDR_LITERAL, 4)), // Address literal
        ]),
    ])),

    ("mov8", Args::Two(vec![
        // Register
        Some(vec![
            None, // Register
            Some((ByteCodes::MOVE_INTO_REG_FROM_ADDR_IN_REG, 8)), // Address in register
            Some((ByteCodes::MOVE_INTO_REG_FROM_CONST, 8)), // Constant
            Some((ByteCodes::MOVE_INTO_REG_FROM_ADDR_LITERAL, 8)), // Address literal
        ]),
        // Address in register
        Some(vec![
            Some((ByteCodes::MOVE_INTO_ADDR_IN_REG_FROM_REG, 8)), // Register
            Some((ByteCodes::MOVE_INTO_ADDR_IN_REG_FROM_ADDR_IN_REG, 8)), // Address in register
            Some((ByteCodes::MOVE_INTO_ADDR_IN_REG_FROM_CONST, 8)), // Constant
            Some((ByteCodes::MOVE_INTO_ADDR_IN_REG_FROM_ADDR_LITERAL, 8)), // Address literal
        ]),
        None, // Constant
        // Address literal
        Some(vec![
            Some((ByteCodes::MOVE_INTO_ADDR_LITERAL_FROM_REG, 8)), // Register
            Some((ByteCodes::MOVE_INTO_ADDR_LITERAL_FROM_ADDR_IN_REG, 8)), // Address in register
            Some((ByteCodes::MOVE_INTO_ADDR_LITERAL_FROM_CONST, 8)), // Constant
            Some((ByteCodes::MOVE_INTO_ADDR_LITERAL_FROM_ADDR_LITERAL, 8)), // Address literal
        ]),
    ])),
    
    ("push", Args::One(vec![
        Some((ByteCodes::PUSH_FROM_REG, 0)), // Register
    ])),

    ("push1", Args::One(vec![
        None, // Register
        Some((ByteCodes::PUSH_FROM_ADDR_IN_REG, 1)), // Address in register
        Some((ByteCodes::PUSH_FROM_CONST, 1)), // Constant
        Some((ByteCodes::PUSH_FROM_ADDR_LITERAL, 1)), // Address literal
    ])),

    ("push2", Args::One(vec![
        None, // Register
        Some((ByteCodes::PUSH_FROM_ADDR_IN_REG, 2)), // Address in register
        Some((ByteCodes::PUSH_FROM_CONST, 2)), // Constant
        Some((ByteCodes::PUSH_FROM_ADDR_LITERAL, 2)), // Address literal
    ])),

    ("push4", Args::One(vec![
        None, // Register
        Some((ByteCodes::PUSH_FROM_ADDR_IN_REG, 4)), // Address in register
        Some((ByteCodes::PUSH_FROM_CONST, 4)), // Constant
        Some((ByteCodes::PUSH_FROM_ADDR_LITERAL, 4)), // Address literal
    ])),

    ("push8", Args::One(vec![
        None, // Register
        Some((ByteCodes::PUSH_FROM_ADDR_IN_REG, 8)), // Address in register
        Some((ByteCodes::PUSH_FROM_CONST, 8)), // Constant
        Some((ByteCodes::PUSH_FROM_ADDR_LITERAL, 8)), // Address literal
    ])),
    
    ("pop1", Args::One(vec![
        Some((ByteCodes::POP_INTO_REG, 1)), // Register
        Some((ByteCodes::POP_INTO_ADDR_IN_REG, 1)), // Address in register
        None, // Constant
        Some((ByteCodes::POP_INTO_ADDR_LITERAL, 1)), // Address literal
    ])),
    
    ("pop2", Args::One(vec![
        Some((ByteCodes::POP_INTO_REG, 2)), // Register
        Some((ByteCodes::POP_INTO_ADDR_IN_REG, 2)), // Address in register
        None, // Constant
        Some((ByteCodes::POP_INTO_ADDR_LITERAL, 2)), // Address literal
    ])),
    
    ("pop4", Args::One(vec![
        Some((ByteCodes::POP_INTO_REG, 4)), // Register
        Some((ByteCodes::POP_INTO_ADDR_IN_REG, 4)), // Address in register
        None, // Constant
        Some((ByteCodes::POP_INTO_ADDR_LITERAL, 4)), // Address literal
    ])),
    
    ("pop8", Args::One(vec![
        Some((ByteCodes::POP_INTO_REG, 8)), // Register
        Some((ByteCodes::POP_INTO_ADDR_IN_REG, 8)), // Address in register
        None, // Constant
        Some((ByteCodes::POP_INTO_ADDR_LITERAL, 8)), // Address literal
    ])),
    
    // Control flow

    ("@", Args::One(vec![
        None, // Register
        None, // Address in register
        None, // Constant
        None, // Address literal
        Some((ByteCodes::LABEL, 0)), // Label
    ])),

    ("jmp", Args::One(vec![
        None, // Register
        None, // Address in register
        None, // Constant
        None, // Address literal
        Some((ByteCodes::JUMP, 0)), // Label
    ])),

    ("jmpnz", Args::Two(vec![
        None, // Register
        None, // Address in register
        None, // Constant
        None, // Address literal
        // Label
        Some(vec![
            Some((ByteCodes::JUMP_IF_NOT_ZERO_REG, 0)) // Register
        ])
    ])),

    ("jmpz", Args::Two(vec![
        None, // Register
        None, // Address in register
        None, // Constant
        None, // Address literal
        // Label
        Some(vec![
            Some((ByteCodes::JUMP_IF_ZERO_REG, 0)) // Register
        ])
    ])),

    // Comparison

    ("cmp", Args::Two(vec![
        // Register
        Some(vec![
            Some((ByteCodes::COMPARE_REG_REG, 0)), // Register
        ])
    ])),

    ("cmp1", Args::Two(vec![
        // Register
        Some(vec![
            None, // Register
            None, // Address in register
            Some((ByteCodes::COMPARE_REG_CONST, 1)), // Constant
        ]),
        None, // Address in register
        // Constant
        Some(vec![
            Some((ByteCodes::COMPARE_CONST_REG, 1)), // Register
            None, // Address in register
            Some((ByteCodes::COMPARE_CONST_CONST, 1)), // Constant
        ]),
    ])),

    ("cmp2", Args::Two(vec![
        // Register
        Some(vec![
            None, // Register
            None, // Address in register
            Some((ByteCodes::COMPARE_REG_CONST, 2)), // Constant
        ]),
        None, // Address in register
        // Constant
        Some(vec![
            Some((ByteCodes::COMPARE_CONST_REG, 2)), // Register
            None, // Address in register
            Some((ByteCodes::COMPARE_CONST_CONST, 2)), // Constant
        ]),
    ])),

    ("cmp4", Args::Two(vec![
        // Register
        Some(vec![
            None, // Register
            None, // Address in register
            Some((ByteCodes::COMPARE_REG_CONST, 4)), // Constant
        ]),
        None, // Address in register
        // Constant
        Some(vec![
            Some((ByteCodes::COMPARE_CONST_REG, 4)), // Register
            None, // Address in register
            Some((ByteCodes::COMPARE_CONST_CONST, 4)), // Constant
        ]),
    ])),

    ("cmp8", Args::Two(vec![
        // Register
        Some(vec![
            None, // Register
            None, // Address in register
            Some((ByteCodes::COMPARE_REG_CONST, 8)), // Constant
        ]),
        None, // Address in register
        // Constant
        Some(vec![
            Some((ByteCodes::COMPARE_CONST_REG, 8)), // Register
            None, // Address in register
            Some((ByteCodes::COMPARE_CONST_CONST, 8)), // Constant
        ]),
    ])),

    // Interrupts

    ("iprint", Args::Zero((ByteCodes::PRINT_SIGNED, 0))), // No arguments

    ("uprint", Args::Zero((ByteCodes::PRINT_UNSIGNED, 0))), // No arguments

    ("printc", Args::Zero((ByteCodes::PRINT_CHAR, 0))), // No arguments

    ("printstr", Args::Zero((ByteCodes::PRINT_STRING, 0))), // No argumets

    ("inputint", Args::Zero((ByteCodes::INPUT_INT, 0))), // No arguments

    ("inputstr", Args::Zero((ByteCodes::INPUT_STRING, 0))), // No arguments

    ("exit", Args::Zero((ByteCodes::EXIT, 0))), // No arguments

]);

}

