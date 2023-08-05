use rust_vm_lib::byte_code::ByteCodes;


/// A pair of operation code and handled size
type Operation = (ByteCodes, u8);
type OneArgument = Vec<Option<Operation>>;
type TwoArguments = Vec<Option<OneArgument>>;


pub enum ArgTable {
    Zero(Operation),
    One(OneArgument),
    Two(TwoArguments),
}


impl ArgTable {

    pub fn required_args(&self) -> usize {
        match self {
            ArgTable::Zero(_) => 0,
            ArgTable::One(_) => 1,
            ArgTable::Two(_) => 2,
        }
    }

}


/// Return the arguments table for the given operator
pub fn get_arguments_table(operator: &str) -> Option<ArgTable> {

    match operator {

        // Arithmetic

        "add" => Some(ArgTable::Zero((ByteCodes::ADD, 0))), // No arguments

        "sub" => Some(ArgTable::Zero((ByteCodes::SUB, 0))), // No arguments

        "mul" => Some(ArgTable::Zero((ByteCodes::MUL, 0))), // No arguments

        "div" => Some(ArgTable::Zero((ByteCodes::DIV, 0))), // No arguments

        "mod" => Some(ArgTable::Zero((ByteCodes::MOD, 0))), // No arguments

        "inc" => Some(ArgTable::One(vec![ 
            Some((ByteCodes::INC_REG, 0)), // Register
        ])), 
    
        "inc1" => Some(ArgTable::One(vec![
            None, // Register
            Some((ByteCodes::INC_ADDR_IN_REG, 1)), // Address in register
            None, // Constant
            Some((ByteCodes::INC_ADDR_LITERAL, 1)), // Address literal
        ])),
        
        "inc2" => Some(ArgTable::One(vec![
            None, // Register
            Some((ByteCodes::INC_ADDR_IN_REG, 2)), // Address in register
            None, // Constant
            Some((ByteCodes::INC_ADDR_LITERAL, 2)), // Address literal
        ])),

        "inc4" => Some(ArgTable::One(vec![
            None, // Register
            Some((ByteCodes::INC_ADDR_IN_REG, 4)), // Address in register
            None, // Constant
            Some((ByteCodes::INC_ADDR_LITERAL, 4)), // Address literal
        ])),
    
        "inc8" => Some(ArgTable::One(vec![
            None, // Register
            Some((ByteCodes::INC_ADDR_IN_REG, 8)), // Address in register
            None, // Constant
            Some((ByteCodes::INC_ADDR_LITERAL, 8)), // Address literal
        ])),

        "dec" => Some(ArgTable::One(vec![
            Some((ByteCodes::DEC_REG, 0)), // Register
        ])),
        
        "dec1" => Some(ArgTable::One(vec![
            None, // Register
            Some((ByteCodes::DEC_ADDR_IN_REG, 1)), // Address in register
            None, // Constant
            Some((ByteCodes::DEC_ADDR_LITERAL, 1)), // Address literal
        ])),
        
        "dec2" => Some(ArgTable::One(vec![
            None, // Register
            Some((ByteCodes::DEC_ADDR_IN_REG, 2)), // Address in register
            None, // Constant
            Some((ByteCodes::DEC_ADDR_LITERAL, 2)), // Address literal
        ])),
        
        "dec4" => Some(ArgTable::One(vec![
            None, // Register
            Some((ByteCodes::DEC_ADDR_IN_REG, 4)), // Address in register
            None, // Constant
            Some((ByteCodes::DEC_ADDR_LITERAL, 4)), // Address literal
        ])),
        
        "dec8" => Some(ArgTable::One(vec![
            None, // Register
            Some((ByteCodes::DEC_ADDR_IN_REG, 8)), // Address in register
            None, // Constant
            Some((ByteCodes::DEC_ADDR_LITERAL, 8)), // Address literal
        ])),
    
        // No operation

        "nop" => Some(ArgTable::Zero((ByteCodes::NO_OPERATION, 0))), // No arguments

        // Memory

        "mov" => Some(ArgTable::Two(vec![
            // Register
            Some(vec![
                Some((ByteCodes::MOVE_INTO_REG_FROM_REG, 0)), // Register
            ])
        ])),

        "mov1" => Some(ArgTable::Two(vec![
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

        "mov2" => Some(ArgTable::Two(vec![
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
        
        "mov4" => Some(ArgTable::Two(vec![
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

        "mov8" => Some(ArgTable::Two(vec![
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
        
        "push" => Some(ArgTable::One(vec![
            Some((ByteCodes::PUSH_FROM_REG, 0)), // Register
        ])),

        "push1" => Some(ArgTable::One(vec![
            None, // Register
            Some((ByteCodes::PUSH_FROM_ADDR_IN_REG, 1)), // Address in register
            Some((ByteCodes::PUSH_FROM_CONST, 1)), // Constant
            Some((ByteCodes::PUSH_FROM_ADDR_LITERAL, 1)), // Address literal
        ])),

        "push2" => Some(ArgTable::One(vec![
            None, // Register
            Some((ByteCodes::PUSH_FROM_ADDR_IN_REG, 2)), // Address in register
            Some((ByteCodes::PUSH_FROM_CONST, 2)), // Constant
            Some((ByteCodes::PUSH_FROM_ADDR_LITERAL, 2)), // Address literal
        ])),

        "push4" => Some(ArgTable::One(vec![
            None, // Register
            Some((ByteCodes::PUSH_FROM_ADDR_IN_REG, 4)), // Address in register
            Some((ByteCodes::PUSH_FROM_CONST, 4)), // Constant
            Some((ByteCodes::PUSH_FROM_ADDR_LITERAL, 4)), // Address literal
        ])),

        "push8" => Some(ArgTable::One(vec![
            None, // Register
            Some((ByteCodes::PUSH_FROM_ADDR_IN_REG, 8)), // Address in register
            Some((ByteCodes::PUSH_FROM_CONST, 8)), // Constant
            Some((ByteCodes::PUSH_FROM_ADDR_LITERAL, 8)), // Address literal
        ])),
        
        "pop1" => Some(ArgTable::One(vec![
            Some((ByteCodes::POP_INTO_REG, 1)), // Register
            Some((ByteCodes::POP_INTO_ADDR_IN_REG, 1)), // Address in register
            None, // Constant
            Some((ByteCodes::POP_INTO_ADDR_LITERAL, 1)), // Address literal
        ])),
        
        "pop2" => Some(ArgTable::One(vec![
            Some((ByteCodes::POP_INTO_REG, 2)), // Register
            Some((ByteCodes::POP_INTO_ADDR_IN_REG, 2)), // Address in register
            None, // Constant
            Some((ByteCodes::POP_INTO_ADDR_LITERAL, 2)), // Address literal
        ])),
        
        "pop4" => Some(ArgTable::One(vec![
            Some((ByteCodes::POP_INTO_REG, 4)), // Register
            Some((ByteCodes::POP_INTO_ADDR_IN_REG, 4)), // Address in register
            None, // Constant
            Some((ByteCodes::POP_INTO_ADDR_LITERAL, 4)), // Address literal
        ])),
        
        "pop8" => Some(ArgTable::One(vec![
            Some((ByteCodes::POP_INTO_REG, 8)), // Register
            Some((ByteCodes::POP_INTO_ADDR_IN_REG, 8)), // Address in register
            None, // Constant
            Some((ByteCodes::POP_INTO_ADDR_LITERAL, 8)), // Address literal
        ])),
        
        // Control flow

        "jmp" => Some(ArgTable::One(vec![
            None, // Register
            None, // Address in register
            None, // Constant
            None, // Address literal
            Some((ByteCodes::JUMP, 0)), // Label
        ])),

        "jmpnz" => Some(ArgTable::Two(vec![
            None, // Register
            None, // Address in register
            None, // Constant
            None, // Address literal
            // Label
            Some(vec![
                Some((ByteCodes::JUMP_IF_NOT_ZERO_REG, 0)) // Register
            ])
        ])),

        "jmpz" => Some(ArgTable::Two(vec![
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

        "cmp" => Some(ArgTable::Two(vec![
            // Register
            Some(vec![
                Some((ByteCodes::COMPARE_REG_REG, 0)), // Register
            ])
        ])),

        "cmp1" => Some(ArgTable::Two(vec![
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

        "cmp2" => Some(ArgTable::Two(vec![
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

        "cmp4" => Some(ArgTable::Two(vec![
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

        "cmp8" => Some(ArgTable::Two(vec![
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

        "iprint" => Some(ArgTable::Zero((ByteCodes::PRINT_SIGNED, 0))), // No arguments

        "uprint" => Some(ArgTable::Zero((ByteCodes::PRINT_UNSIGNED, 0))), // No arguments

        "printc" => Some(ArgTable::Zero((ByteCodes::PRINT_CHAR, 0))), // No arguments

        "printstr" => Some(ArgTable::Zero((ByteCodes::PRINT_STRING, 0))), // No argumets

        "inputint" => Some(ArgTable::Zero((ByteCodes::INPUT_INT, 0))), // No arguments

        "inputstr" => Some(ArgTable::Zero((ByteCodes::INPUT_STRING, 0))), // No arguments

        "exit" => Some(ArgTable::Zero((ByteCodes::EXIT, 0))), // No arguments

        _ => None

    }

}


