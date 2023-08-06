use std::path::Path;

use rust_vm_lib::{byte_code::ByteCodes, token::{Token, TokenTypes}};

use crate::error;


/// A pair of operation code and handled size
type Operation = (ByteCodes, u8);
type OneArgument = Vec<Option<Operation>>;
type TwoArguments = Vec<Option<OneArgument>>;


/// Represents all the arguments an operator can take
pub enum ArgTable {
    Zero(Operation),
    One(OneArgument),
    Two(TwoArguments),
}


impl ArgTable {


    /// Return al the possible argument combinations in the argument table
    fn get_possible_combinations(&self) -> Vec<String> {
        match self {

            ArgTable::Zero(_) => Vec::new(),

            ArgTable::One(arg) => {

                let mut combinations = Vec::new();

                for (i, arg) in arg.iter().enumerate() {
                    if arg.is_some() {
                        let required_type = TokenTypes::from_ordinal(i as u8);
                        combinations.push(required_type.to_string());
                    }
                }

                combinations
            },

            ArgTable::Two(arg1) => {

                let mut combinations = Vec::new();

                for (i, arg2) in arg1.iter().enumerate() {

                    if let Some(arg2) = arg2 {

                        let required_type_1 = TokenTypes::from_ordinal(i as u8);

                        for (j, arg2) in arg2.iter().enumerate() {
                            if arg2.is_some() {
                                let required_type_2 = TokenTypes::from_ordinal(j as u8);
                                combinations.push(format!("{} {}", required_type_1, required_type_2));
                            }
                        }
                    }
                }

                combinations
            },

        }
    }


    /// Return the bytecode instruction and handled size for the given operands
    pub fn get_instruction(&self, operator_name: &str, operands: &[Token], unit_path: &Path, line_number: usize, line: &str) -> (ByteCodes, u8) {

        match self {
            
            ArgTable::Zero(op) => {

                // The operator requires zero arguments
                if !operands.is_empty() {
                    error::invalid_arg_number(unit_path, operands.len(), 0, line_number, line, operator_name);
                }

                *op
            },

            ArgTable::One(required_arg) => {

                // The operator requires one argument
                if operands.len() != 1 {
                    error::invalid_arg_number(unit_path, operands.len(), 1, line_number, line, operator_name);
                }

                // Return the instruction and handled size for the given operand
                required_arg.get(operands[0].value.to_ordinal() as usize).unwrap_or_else(
                    || error::invalid_token_argument(unit_path, operator_name, &operands[0], line_number, line, &self.get_possible_combinations())
                ).unwrap_or_else(
                    || error::invalid_token_argument(unit_path, operator_name, &operands[0], line_number, line, &self.get_possible_combinations())
                )
            },

            ArgTable::Two(required_arg1) => {

                // The operator requires two arguments
                if operands.len() != 2 {
                    error::invalid_arg_number(unit_path, operands.len(), 2, line_number, line, operator_name);
                }

                // Get the second required argument from the argument table
                let required_arg_2 = required_arg1.get(operands[0].value.to_ordinal() as usize).unwrap_or_else(
                    || error::invalid_token_argument(unit_path, operator_name, &operands[0], line_number, line, &self.get_possible_combinations())
                ).as_ref().unwrap_or_else(
                    || error::invalid_token_argument(unit_path, operator_name, &operands[0], line_number, line, &self.get_possible_combinations())
                );

                // Return the instruction and handled size for the given operand pair
                required_arg_2.get(operands[1].value.to_ordinal() as usize).unwrap_or_else(
                    || error::invalid_token_argument(unit_path, operator_name, &operands[1], line_number, line, &self.get_possible_combinations())
                ).unwrap_or_else(
                    || error::invalid_token_argument(unit_path, operator_name, &operands[1], line_number, line, &self.get_possible_combinations())
                )
            
            },

        }

    }

}


/// Return the arguments table for the given operator
pub fn get_arguments_table(operator_name: &str) -> Option<ArgTable> {

    match operator_name {

        // Arithmetic

        "add" => Some(ArgTable::Zero((ByteCodes::ADD, 0))), // No arguments

        "sub" => Some(ArgTable::Zero((ByteCodes::SUB, 0))), // No arguments

        "mul" => Some(ArgTable::Zero((ByteCodes::MUL, 0))), // No arguments

        "div" => Some(ArgTable::Zero((ByteCodes::DIV, 0))), // No arguments

        "mod" => Some(ArgTable::Zero((ByteCodes::MOD, 0))), // No arguments

        "inc" => Some(ArgTable::One(vec![ 
            // Register
            Some((ByteCodes::INC_REG, 0)),
        ])), 
    
        "inc1" => Some(ArgTable::One(vec![
            // Register
            None,
            // Address in register
            Some((ByteCodes::INC_ADDR_IN_REG, 1)),
            // Number
            None,
            // Address literal
            Some((ByteCodes::INC_ADDR_LITERAL, 1)),
            // Label
            None,
            // Address at label
            Some((ByteCodes::INC_ADDR_LITERAL, 1)),
        ])),
        
        "inc2" => Some(ArgTable::One(vec![
            // Register
            None,
            // Address in register
            Some((ByteCodes::INC_ADDR_IN_REG, 2)),
            // Number
            None,
            // Address literal
            Some((ByteCodes::INC_ADDR_LITERAL, 2)),
            // Label
            None,
            // Address at label
            Some((ByteCodes::INC_ADDR_LITERAL, 2)),
        ])),

        "inc4" => Some(ArgTable::One(vec![
            // Register
            None,
            // Address in register
            Some((ByteCodes::INC_ADDR_IN_REG, 4)),
            // Number
            None,
            // Address literal
            Some((ByteCodes::INC_ADDR_LITERAL, 4)),
            // Label
            None,
            // Address at label
            Some((ByteCodes::INC_ADDR_LITERAL, 4)),
        ])),
    
        "inc8" => Some(ArgTable::One(vec![
            // Register
            None,
            // Address in register
            Some((ByteCodes::INC_ADDR_IN_REG, 8)),
            // Number
            None,
            // Address literal
            Some((ByteCodes::INC_ADDR_LITERAL, 8)),
            // Label
            None,
            // Address at label
            Some((ByteCodes::INC_ADDR_LITERAL, 8)),
        ])),

        "dec" => Some(ArgTable::One(vec![
            // Register
            Some((ByteCodes::DEC_REG, 0)),
        ])),
        
        "dec1" => Some(ArgTable::One(vec![
            // Register
            None,
            // Address in register
            Some((ByteCodes::DEC_ADDR_IN_REG, 1)),
            // Number
            None,
            // Address literal
            Some((ByteCodes::DEC_ADDR_LITERAL, 1)),
            // Label
            None,
            // Address at label
            Some((ByteCodes::DEC_ADDR_LITERAL, 1)),
        ])),
        
        "dec2" => Some(ArgTable::One(vec![
            // Register
            None,
            // Address in register
            Some((ByteCodes::DEC_ADDR_IN_REG, 2)),
            // Number
            None,
            // Address literal
            Some((ByteCodes::DEC_ADDR_LITERAL, 2)),
            // Label
            None,
            // Address at label
            Some((ByteCodes::DEC_ADDR_LITERAL, 2)),
        ])),
        
        "dec4" => Some(ArgTable::One(vec![
            // Register
            None, 
            // Address in register
            Some((ByteCodes::DEC_ADDR_IN_REG, 4)),
            // Number
            None, 
            // Address literal
            Some((ByteCodes::DEC_ADDR_LITERAL, 4)),
            // Label
            None,
            // Address at label
            Some((ByteCodes::DEC_ADDR_LITERAL, 4)),
        ])),
        
        "dec8" => Some(ArgTable::One(vec![
            // Register
            None,
            // Address in register
            Some((ByteCodes::DEC_ADDR_IN_REG, 8)),
            // Number
            None, 
            // Address literal
            Some((ByteCodes::DEC_ADDR_LITERAL, 8)),
            // Label
            None,
            // Address at label
            Some((ByteCodes::DEC_ADDR_LITERAL, 8)),
        ])),
    
        // No operation

        "nop" => Some(ArgTable::Zero((ByteCodes::NO_OPERATION, 0))), // No arguments

        // Memory

        "mov" => Some(ArgTable::Two(vec![
            // Register
            Some(vec![
                // Register
                Some((ByteCodes::MOVE_INTO_REG_FROM_REG, 0)),
            ])
        ])),

        "mov1" => Some(ArgTable::Two(vec![
            // Register
            Some(vec![
                // Register
                None,
                // Address in register
                Some((ByteCodes::MOVE_INTO_REG_FROM_ADDR_IN_REG, 1)),
                // Number
                Some((ByteCodes::MOVE_INTO_REG_FROM_CONST, 1)),
                // Address literal
                Some((ByteCodes::MOVE_INTO_REG_FROM_ADDR_LITERAL, 1)),
                // Label
                Some((ByteCodes::MOVE_INTO_REG_FROM_CONST, 1)),
                // Address at label
                Some((ByteCodes::MOVE_INTO_REG_FROM_ADDR_LITERAL, 1)),
            ]),
            // Address in register
            Some(vec![
                // Register
                Some((ByteCodes::MOVE_INTO_ADDR_IN_REG_FROM_REG, 1)),
                // Address in register
                Some((ByteCodes::MOVE_INTO_ADDR_IN_REG_FROM_ADDR_IN_REG, 1)),
                // Number
                Some((ByteCodes::MOVE_INTO_ADDR_IN_REG_FROM_CONST, 1)), 
                // Address literal
                Some((ByteCodes::MOVE_INTO_ADDR_IN_REG_FROM_ADDR_LITERAL, 1)),
                // Label
                Some((ByteCodes::MOVE_INTO_ADDR_IN_REG_FROM_CONST, 1)),
                // Address at label
                Some((ByteCodes::MOVE_INTO_ADDR_IN_REG_FROM_ADDR_LITERAL, 1)),
            ]),
            // Number
            None,
            // Address literal
            Some(vec![
                // Register
                Some((ByteCodes::MOVE_INTO_ADDR_LITERAL_FROM_REG, 1)),
                // Address in register
                Some((ByteCodes::MOVE_INTO_ADDR_LITERAL_FROM_ADDR_IN_REG, 1)),
                // Number
                Some((ByteCodes::MOVE_INTO_ADDR_LITERAL_FROM_CONST, 1)),
                // Address literal
                Some((ByteCodes::MOVE_INTO_ADDR_LITERAL_FROM_ADDR_LITERAL, 1)),
                // Label
                Some((ByteCodes::MOVE_INTO_ADDR_LITERAL_FROM_CONST, 1)),
                // Address at label
                Some((ByteCodes::MOVE_INTO_ADDR_LITERAL_FROM_ADDR_LITERAL, 1)),
            ]),
            // Label
            None,
            // Address at label
            Some(vec![
                // Register
                Some((ByteCodes::MOVE_INTO_ADDR_LITERAL_FROM_REG, 1)),
                // Address in register
                Some((ByteCodes::MOVE_INTO_ADDR_LITERAL_FROM_ADDR_IN_REG, 1)),
                // Number
                Some((ByteCodes::MOVE_INTO_ADDR_LITERAL_FROM_CONST, 1)),
                // Address literal
                Some((ByteCodes::MOVE_INTO_ADDR_LITERAL_FROM_ADDR_LITERAL, 1)),
                // Label
                Some((ByteCodes::MOVE_INTO_ADDR_LITERAL_FROM_CONST, 1)),
                // Address at label
                Some((ByteCodes::MOVE_INTO_ADDR_LITERAL_FROM_ADDR_LITERAL, 1)),
            ]),
        ])),

        "mov2" => Some(ArgTable::Two(vec![
            // Register
            Some(vec![
                // Register
                None, 
                // Address in register
                Some((ByteCodes::MOVE_INTO_REG_FROM_ADDR_IN_REG, 2)), 
                // Number
                Some((ByteCodes::MOVE_INTO_REG_FROM_CONST, 2)), 
                // Address literal
                Some((ByteCodes::MOVE_INTO_REG_FROM_ADDR_LITERAL, 2)), 
                // Label
                Some((ByteCodes::MOVE_INTO_REG_FROM_CONST, 2)),
                // Address at label
                Some((ByteCodes::MOVE_INTO_REG_FROM_ADDR_LITERAL, 2)),
            ]),
            // Address in register
            Some(vec![
                // Register
                Some((ByteCodes::MOVE_INTO_ADDR_IN_REG_FROM_REG, 2)),
                // Address in register
                Some((ByteCodes::MOVE_INTO_ADDR_IN_REG_FROM_ADDR_IN_REG, 2)), 
                // Number
                Some((ByteCodes::MOVE_INTO_ADDR_IN_REG_FROM_CONST, 2)), 
                // Address literal
                Some((ByteCodes::MOVE_INTO_ADDR_IN_REG_FROM_ADDR_LITERAL, 2)), 
                // Label
                Some((ByteCodes::MOVE_INTO_ADDR_IN_REG_FROM_CONST, 2)),
                // Address at label
                Some((ByteCodes::MOVE_INTO_ADDR_IN_REG_FROM_ADDR_LITERAL, 2)),
            ]),
            // Number
            None,
            // Address literal
            Some(vec![
                // Register
                Some((ByteCodes::MOVE_INTO_ADDR_LITERAL_FROM_REG, 2)),
                // Address in register
                Some((ByteCodes::MOVE_INTO_ADDR_LITERAL_FROM_ADDR_IN_REG, 2)),
                // Number
                Some((ByteCodes::MOVE_INTO_ADDR_LITERAL_FROM_CONST, 2)), 
                // Address literal
                Some((ByteCodes::MOVE_INTO_ADDR_LITERAL_FROM_ADDR_LITERAL, 2)), 
                // Label
                Some((ByteCodes::MOVE_INTO_ADDR_LITERAL_FROM_CONST, 2)),
                // Address at label
                Some((ByteCodes::MOVE_INTO_ADDR_LITERAL_FROM_ADDR_LITERAL, 2)),
            ]),
            // Label
            None,
            // Address at label
            Some(vec![
                // Register
                Some((ByteCodes::MOVE_INTO_ADDR_LITERAL_FROM_REG, 2)),
                // Address in register
                Some((ByteCodes::MOVE_INTO_ADDR_LITERAL_FROM_ADDR_IN_REG, 2)),
                // Number
                Some((ByteCodes::MOVE_INTO_ADDR_LITERAL_FROM_CONST, 2)), 
                // Address literal
                Some((ByteCodes::MOVE_INTO_ADDR_LITERAL_FROM_ADDR_LITERAL, 2)), 
                // Label
                Some((ByteCodes::MOVE_INTO_ADDR_LITERAL_FROM_CONST, 2)),
                // Address at label
                Some((ByteCodes::MOVE_INTO_ADDR_LITERAL_FROM_ADDR_LITERAL, 2)),
            ]),
        ])),
        
        "mov4" => Some(ArgTable::Two(vec![
            // Register
            Some(vec![
                // Register
                None, 
                // Address in register
                Some((ByteCodes::MOVE_INTO_REG_FROM_ADDR_IN_REG, 4)), 
                // Number
                Some((ByteCodes::MOVE_INTO_REG_FROM_CONST, 4)), 
                // Address literal
                Some((ByteCodes::MOVE_INTO_REG_FROM_ADDR_LITERAL, 4)), 
                // Label
                Some((ByteCodes::MOVE_INTO_REG_FROM_CONST, 4)),
                // Address at label
                Some((ByteCodes::MOVE_INTO_REG_FROM_ADDR_LITERAL, 4)),
            ]),
            // Address in register
            Some(vec![
                // Register
                Some((ByteCodes::MOVE_INTO_ADDR_IN_REG_FROM_REG, 4)), 
                // Address in register
                Some((ByteCodes::MOVE_INTO_ADDR_IN_REG_FROM_ADDR_IN_REG, 4)), 
                // Number
                Some((ByteCodes::MOVE_INTO_ADDR_IN_REG_FROM_CONST, 4)),
                // Address literal
                Some((ByteCodes::MOVE_INTO_ADDR_IN_REG_FROM_ADDR_LITERAL, 4)), 
                // Label
                Some((ByteCodes::MOVE_INTO_ADDR_IN_REG_FROM_CONST, 4)),
                // Address at label
                Some((ByteCodes::MOVE_INTO_ADDR_IN_REG_FROM_ADDR_LITERAL, 4)),
            ]),
            // Number
            None, 
            // Address literal
            Some(vec![
                // Register
                Some((ByteCodes::MOVE_INTO_ADDR_LITERAL_FROM_REG, 4)),
                // Address in register
                Some((ByteCodes::MOVE_INTO_ADDR_LITERAL_FROM_ADDR_IN_REG, 4)), 
                // Number
                Some((ByteCodes::MOVE_INTO_ADDR_LITERAL_FROM_CONST, 4)), 
                // Address literal
                Some((ByteCodes::MOVE_INTO_ADDR_LITERAL_FROM_ADDR_LITERAL, 4)), 
                // Label
                Some((ByteCodes::MOVE_INTO_ADDR_LITERAL_FROM_CONST, 4)),
                // Address at label
                Some((ByteCodes::MOVE_INTO_ADDR_LITERAL_FROM_ADDR_LITERAL, 4)),
            ]),
            // Label
            None,
            // Address at label
            Some(vec![
                // Register
                Some((ByteCodes::MOVE_INTO_ADDR_LITERAL_FROM_REG, 4)),
                // Address in register
                Some((ByteCodes::MOVE_INTO_ADDR_LITERAL_FROM_ADDR_IN_REG, 4)), 
                // Number
                Some((ByteCodes::MOVE_INTO_ADDR_LITERAL_FROM_CONST, 4)), 
                // Address literal
                Some((ByteCodes::MOVE_INTO_ADDR_LITERAL_FROM_ADDR_LITERAL, 4)), 
                // Label
                Some((ByteCodes::MOVE_INTO_ADDR_LITERAL_FROM_CONST, 4)),
                // Address at label
                Some((ByteCodes::MOVE_INTO_ADDR_LITERAL_FROM_ADDR_LITERAL, 4)),
            ]),
        ])),

        "mov8" => Some(ArgTable::Two(vec![
            // Register
            Some(vec![
                // Register
                None, 
                // Address in register
                Some((ByteCodes::MOVE_INTO_REG_FROM_ADDR_IN_REG, 8)), 
                // Number
                Some((ByteCodes::MOVE_INTO_REG_FROM_CONST, 8)), 
                // Address literal
                Some((ByteCodes::MOVE_INTO_REG_FROM_ADDR_LITERAL, 8)), 
                // Label
                Some((ByteCodes::MOVE_INTO_REG_FROM_CONST, 8)),
                // Address at label
                Some((ByteCodes::MOVE_INTO_REG_FROM_ADDR_LITERAL, 8)),
            ]),
            // Address in register
            Some(vec![
                // Register
                Some((ByteCodes::MOVE_INTO_ADDR_IN_REG_FROM_REG, 8)),
                // Address in register
                Some((ByteCodes::MOVE_INTO_ADDR_IN_REG_FROM_ADDR_IN_REG, 8)), 
                // Number
                Some((ByteCodes::MOVE_INTO_ADDR_IN_REG_FROM_CONST, 8)), 
                // Address literal
                Some((ByteCodes::MOVE_INTO_ADDR_IN_REG_FROM_ADDR_LITERAL, 8)),
                // Label
                Some((ByteCodes::MOVE_INTO_ADDR_IN_REG_FROM_CONST, 8)),
                // Address at label
                Some((ByteCodes::MOVE_INTO_ADDR_IN_REG_FROM_ADDR_LITERAL, 8)),
            ]),
            // Number
            None,
            // Address literal
            Some(vec![
                // Register
                Some((ByteCodes::MOVE_INTO_ADDR_LITERAL_FROM_REG, 8)), 
                // Address in register
                Some((ByteCodes::MOVE_INTO_ADDR_LITERAL_FROM_ADDR_IN_REG, 8)), 
                // Number
                Some((ByteCodes::MOVE_INTO_ADDR_LITERAL_FROM_CONST, 8)), 
                // Address literal
                Some((ByteCodes::MOVE_INTO_ADDR_LITERAL_FROM_ADDR_LITERAL, 8)),
                // Label
                Some((ByteCodes::MOVE_INTO_ADDR_LITERAL_FROM_CONST, 8)),
                // Address at label
                Some((ByteCodes::MOVE_INTO_ADDR_LITERAL_FROM_ADDR_LITERAL, 8)),
            ]),
            // Label
            None,
            // Address at label
            Some(vec![
                // Register
                Some((ByteCodes::MOVE_INTO_ADDR_LITERAL_FROM_REG, 8)), 
                // Address in register
                Some((ByteCodes::MOVE_INTO_ADDR_LITERAL_FROM_ADDR_IN_REG, 8)), 
                // Number
                Some((ByteCodes::MOVE_INTO_ADDR_LITERAL_FROM_CONST, 8)), 
                // Address literal
                Some((ByteCodes::MOVE_INTO_ADDR_LITERAL_FROM_ADDR_LITERAL, 8)),
                // Label
                Some((ByteCodes::MOVE_INTO_ADDR_LITERAL_FROM_CONST, 8)),
                // Address at label
                Some((ByteCodes::MOVE_INTO_ADDR_LITERAL_FROM_ADDR_LITERAL, 8)),
            ]),
        ])),
        
        "push" => Some(ArgTable::One(vec![
            Some((ByteCodes::PUSH_FROM_REG, 0)), // Register
        ])),

        "push1" => Some(ArgTable::One(vec![
            // Register
            None, 
            // Address in register
            Some((ByteCodes::PUSH_FROM_ADDR_IN_REG, 1)), 
            // Number
            Some((ByteCodes::PUSH_FROM_CONST, 1)),
            // Address literal
            Some((ByteCodes::PUSH_FROM_ADDR_LITERAL, 1)), 
            // Label
            Some((ByteCodes::PUSH_FROM_CONST, 1)),
            // Address at label
            Some((ByteCodes::PUSH_FROM_ADDR_LITERAL, 1)),
        ])),

        "push2" => Some(ArgTable::One(vec![
            // Register
            None, 
            // Address in register
            Some((ByteCodes::PUSH_FROM_ADDR_IN_REG, 2)), 
            // Number
            Some((ByteCodes::PUSH_FROM_CONST, 2)),
            // Address literal
            Some((ByteCodes::PUSH_FROM_ADDR_LITERAL, 2)), 
            // Label
            Some((ByteCodes::PUSH_FROM_CONST, 2)),
            // Address at label
            Some((ByteCodes::PUSH_FROM_ADDR_LITERAL, 2)),
        ])),

        "push4" => Some(ArgTable::One(vec![
            // Register
            None, 
            // Address in register
            Some((ByteCodes::PUSH_FROM_ADDR_IN_REG, 4)), 
            // Number
            Some((ByteCodes::PUSH_FROM_CONST, 4)), 
            // Address literal
            Some((ByteCodes::PUSH_FROM_ADDR_LITERAL, 4)),
            // Label
            Some((ByteCodes::PUSH_FROM_CONST, 4)),
            // Address at label
            Some((ByteCodes::PUSH_FROM_ADDR_LITERAL, 4)),
        ])),

        "push8" => Some(ArgTable::One(vec![
            // Register
            None, 
            // Address in register
            Some((ByteCodes::PUSH_FROM_ADDR_IN_REG, 8)), 
            // Number
            Some((ByteCodes::PUSH_FROM_CONST, 8)), 
            // Address literal
            Some((ByteCodes::PUSH_FROM_ADDR_LITERAL, 8)), 
            // Label
            Some((ByteCodes::PUSH_FROM_CONST, 8)),
            // Address at label
            Some((ByteCodes::PUSH_FROM_ADDR_LITERAL, 8)),
        ])),
        
        "pop1" => Some(ArgTable::One(vec![
            // Register
            Some((ByteCodes::POP_INTO_REG, 1)), 
            // Address in register
            Some((ByteCodes::POP_INTO_ADDR_IN_REG, 1)), 
            // Number
            None, 
            // Address literal
            Some((ByteCodes::POP_INTO_ADDR_LITERAL, 1)), 
            // Label
            None,
            // Address at label
            Some((ByteCodes::POP_INTO_ADDR_LITERAL, 1)),
        ])),
        
        "pop2" => Some(ArgTable::One(vec![
            // Register
            Some((ByteCodes::POP_INTO_REG, 2)),
            // Address in register
            Some((ByteCodes::POP_INTO_ADDR_IN_REG, 2)), 
            // Number
            None, 
            // Address literal
            Some((ByteCodes::POP_INTO_ADDR_LITERAL, 2)), 
            // Label
            None,
            // Address at label
            Some((ByteCodes::POP_INTO_ADDR_LITERAL, 2)),
        ])),
        
        "pop4" => Some(ArgTable::One(vec![
            // Register
            Some((ByteCodes::POP_INTO_REG, 4)), 
            // Address in register
            Some((ByteCodes::POP_INTO_ADDR_IN_REG, 4)),
            // Number
            None, 
            // Address literal
            Some((ByteCodes::POP_INTO_ADDR_LITERAL, 4)), 
            // Label
            None,
            // Address at label
            Some((ByteCodes::POP_INTO_ADDR_LITERAL, 4)),
        ])),
        
        "pop8" => Some(ArgTable::One(vec![
            // Register
            Some((ByteCodes::POP_INTO_REG, 8)), 
            // Address in register
            Some((ByteCodes::POP_INTO_ADDR_IN_REG, 8)), 
            // Number
            None, 
            // Address literal
            Some((ByteCodes::POP_INTO_ADDR_LITERAL, 8)), 
            // Label
            None,
            // Address at label
            Some((ByteCodes::POP_INTO_ADDR_LITERAL, 8)),
        ])),
        
        // Control flow

        "jmp" => Some(ArgTable::One(vec![
            // Register
            Some((ByteCodes::JUMP_TO_REG, 0)), 
            // Address in register
            Some((ByteCodes::JUMP_TO_ADDR_IN_REG, 0)),
            // Number
            Some((ByteCodes::JUMP_TO_CONST, 0)),
            // Address literal
            Some((ByteCodes::JUMP_TO_ADDR_LITERAL, 0)),
            // Label
            Some((ByteCodes::JUMP_TO_CONST, 0)),
            // Address at label
            Some((ByteCodes::JUMP_TO_ADDR_LITERAL, 0)),
        ])),

        "jmpnz" => Some(ArgTable::Two(vec![
            // Register
            Some(vec![
                // Register
                Some((ByteCodes::JUMP_IF_NOT_ZERO_REG_TO_REG, 0)),
            ]), 
            // Address in register
            Some(vec![  
                // Register
                Some((ByteCodes::JUMP_IF_NOT_ZERO_REG_TO_ADDR_IN_REG, 0)),
            ]), 
            // Number
            Some(vec![
                // Register
                Some((ByteCodes::JUMP_IF_NOT_ZERO_REG_TO_CONST, 0)), 
            ]),
            // Address literal
            Some(vec![
                // Register
                Some((ByteCodes::JUMP_IF_NOT_ZERO_REG_TO_ADDR_LITERAL, 0)),
            ]),
            // Label
            Some(vec![
                // Register
                Some((ByteCodes::JUMP_IF_NOT_ZERO_REG_TO_CONST, 0)),
            ]),
            // Address at label
            Some(vec![
                // Register
                Some((ByteCodes::JUMP_IF_NOT_ZERO_REG_TO_ADDR_LITERAL, 0)),
            ]),
        ])),

        "jmpz" => Some(ArgTable::Two(vec![
            // Register
            Some(vec![
                // Register
                Some((ByteCodes::JUMP_IF_ZERO_REG_TO_REG, 0)),
            ]),
            // Address in register
            Some(vec![
                // Register
                Some((ByteCodes::JUMP_IF_ZERO_REG_TO_ADDR_IN_REG, 0)),
            ]),
            // Number
            Some(vec![
                // Register
                Some((ByteCodes::JUMP_IF_ZERO_REG_TO_CONST, 0)), 
            ]),
            // Address literal
            Some(vec![
                // Register
                Some((ByteCodes::JUMP_IF_ZERO_REG_TO_ADDR_LITERAL, 0)),
            ]),
            // Label
            Some(vec![
                // Register
                Some((ByteCodes::JUMP_IF_ZERO_REG_TO_CONST, 0)),
            ]),
            // Address at label
            Some(vec![
                // Register
                Some((ByteCodes::JUMP_IF_ZERO_REG_TO_ADDR_LITERAL, 0)),
            ]),
        ])),

        // Comparison

        "cmp" => Some(ArgTable::Two(vec![
            // Register
            Some(vec![
                // Register
                Some((ByteCodes::COMPARE_REG_REG, 0)),
            ])
        ])),

        "cmp1" => Some(ArgTable::Two(vec![
            // Register
            Some(vec![
                // Register
                None, 
                // Address in register
                Some((ByteCodes::COMPARE_REG_ADDR_IN_REG, 1)), 
                // Number
                Some((ByteCodes::COMPARE_REG_CONST, 1)), 
                // Address literal
                Some((ByteCodes::COMPARE_REG_ADDR_LITERAL, 1)),
                // Label
                Some((ByteCodes::COMPARE_REG_CONST, 1)),
                // Address at label
                Some((ByteCodes::COMPARE_REG_ADDR_LITERAL, 1)),
            ]),
            // Address in register
            Some(vec![
                // Register
                Some((ByteCodes::COMPARE_ADDR_IN_REG_REG, 1)), 
                // Address in register
                Some((ByteCodes::COMPARE_ADDR_IN_REG_ADDR_IN_REG, 1)), 
                // Number
                Some((ByteCodes::COMPARE_ADDR_IN_REG_CONST, 1)), 
                // Address literal
                Some((ByteCodes::COMPARE_ADDR_IN_REG_ADDR_LITERAL, 1)),
                // Label
                Some((ByteCodes::COMPARE_ADDR_IN_REG_CONST, 1)),
                // Address at label
                Some((ByteCodes::COMPARE_ADDR_IN_REG_ADDR_LITERAL, 1)),
            ]),
            // Number
            Some(vec![
                // Register
                Some((ByteCodes::COMPARE_CONST_REG, 1)),
                // Address in register
                Some((ByteCodes::COMPARE_CONST_ADDR_IN_REG, 1)), 
                // Number
                Some((ByteCodes::COMPARE_CONST_CONST, 1)),
                // Address literal
                Some((ByteCodes::COMPARE_CONST_ADDR_LITERAL, 1)),
                // Label
                Some((ByteCodes::COMPARE_CONST_CONST, 1)),
                // Address at label
                Some((ByteCodes::COMPARE_CONST_ADDR_LITERAL, 1)),
            ]),
            // Address literal
            Some(vec![
                // Register
                Some((ByteCodes::COMPARE_ADDR_LITERAL_REG, 1)),
                // Address in register
                Some((ByteCodes::COMPARE_ADDR_LITERAL_ADDR_IN_REG, 1)),
                // Number
                Some((ByteCodes::COMPARE_ADDR_LITERAL_CONST, 1)),
                // Address literal
                Some((ByteCodes::COMPARE_ADDR_LITERAL_ADDR_LITERAL, 1)),
                // Label
                Some((ByteCodes::COMPARE_ADDR_LITERAL_CONST, 1)),
                // Address at label
                Some((ByteCodes::COMPARE_ADDR_LITERAL_ADDR_LITERAL, 1)),
            ]),
            // Label
            Some(vec![
                // Register
                Some((ByteCodes::COMPARE_CONST_REG, 1)),
                // Address in register
                Some((ByteCodes::COMPARE_CONST_ADDR_IN_REG, 1)),
                // Number
                Some((ByteCodes::COMPARE_CONST_CONST, 1)),
                // Address literal
                Some((ByteCodes::COMPARE_CONST_ADDR_LITERAL, 1)),
                // Label
                Some((ByteCodes::COMPARE_CONST_CONST, 1)),
                // Address at label
                Some((ByteCodes::COMPARE_CONST_ADDR_LITERAL, 1)),
            ]),
            // Address at label
            Some(vec![
                // Register
                Some((ByteCodes::COMPARE_ADDR_LITERAL_REG, 1)),
                // Address in register
                Some((ByteCodes::COMPARE_ADDR_LITERAL_ADDR_IN_REG, 1)),
                // Number
                Some((ByteCodes::COMPARE_ADDR_LITERAL_CONST, 1)),
                // Address literal
                Some((ByteCodes::COMPARE_ADDR_LITERAL_ADDR_LITERAL, 1)),
                // Label
                Some((ByteCodes::COMPARE_ADDR_LITERAL_CONST, 1)),
                // Address at label
                Some((ByteCodes::COMPARE_ADDR_LITERAL_ADDR_LITERAL, 1)),
            ]),
        ])),

        "cmp2" => Some(ArgTable::Two(vec![
            // Register
            Some(vec![
                // Register
                None, 
                // Address in register
                Some((ByteCodes::COMPARE_REG_ADDR_IN_REG, 2)),
                // Number
                Some((ByteCodes::COMPARE_REG_CONST, 2)),
                // Address literal
                Some((ByteCodes::COMPARE_REG_ADDR_LITERAL, 2)),
                // Label
                Some((ByteCodes::COMPARE_REG_CONST, 2)),
                // Address at label
                Some((ByteCodes::COMPARE_REG_ADDR_LITERAL, 2)),
            ]),
            // Address in register
            Some(vec![
                // Register
                Some((ByteCodes::COMPARE_ADDR_IN_REG_REG, 2)), 
                // Address in register
                Some((ByteCodes::COMPARE_ADDR_IN_REG_ADDR_IN_REG, 2)), 
                // Number
                Some((ByteCodes::COMPARE_ADDR_IN_REG_CONST, 2)), 
                // Address literal
                Some((ByteCodes::COMPARE_ADDR_IN_REG_ADDR_LITERAL, 2)),
                // Label
                Some((ByteCodes::COMPARE_ADDR_IN_REG_CONST, 2)),
                // Address at label
                Some((ByteCodes::COMPARE_ADDR_IN_REG_ADDR_LITERAL, 2)),
            ]),
            // Number
            Some(vec![
                // Register
                Some((ByteCodes::COMPARE_CONST_REG, 2)), 
                // Address in register
                Some((ByteCodes::COMPARE_CONST_ADDR_IN_REG, 2)),
                // Number
                Some((ByteCodes::COMPARE_CONST_CONST, 2)), 
                // Address literal
                Some((ByteCodes::COMPARE_CONST_ADDR_LITERAL, 2)),
                // Label
                Some((ByteCodes::COMPARE_CONST_CONST, 2)),
                // Address at label
                Some((ByteCodes::COMPARE_CONST_ADDR_LITERAL, 2)),
            ]),
            // Address literal
            Some(vec![
                // Register
                Some((ByteCodes::COMPARE_ADDR_LITERAL_REG, 2)), 
                // Address in register
                Some((ByteCodes::COMPARE_ADDR_LITERAL_ADDR_IN_REG, 2)),
                // Number
                Some((ByteCodes::COMPARE_ADDR_LITERAL_CONST, 2)), 
                // Address literal
                Some((ByteCodes::COMPARE_ADDR_LITERAL_ADDR_LITERAL, 2)),
                // Label
                Some((ByteCodes::COMPARE_ADDR_LITERAL_CONST, 2)),
                // Address at label
                Some((ByteCodes::COMPARE_ADDR_LITERAL_ADDR_LITERAL, 2)),
            ]),
            // Label
            Some(vec![
                // Register
                Some((ByteCodes::COMPARE_CONST_REG, 2)), 
                // Address in register
                Some((ByteCodes::COMPARE_CONST_ADDR_IN_REG, 2)),
                // Number
                Some((ByteCodes::COMPARE_CONST_CONST, 2)), 
                // Address literal
                Some((ByteCodes::COMPARE_CONST_ADDR_LITERAL, 2)),
                // Label
                Some((ByteCodes::COMPARE_CONST_CONST, 2)),
                // Address at label
                Some((ByteCodes::COMPARE_CONST_ADDR_LITERAL, 2)),
            ]),
            // Address at label
            Some(vec![
                // Register
                Some((ByteCodes::COMPARE_ADDR_LITERAL_REG, 2)), 
                // Address in register
                Some((ByteCodes::COMPARE_ADDR_LITERAL_ADDR_IN_REG, 2)),
                // Number
                Some((ByteCodes::COMPARE_ADDR_LITERAL_CONST, 2)), 
                // Address literal
                Some((ByteCodes::COMPARE_ADDR_LITERAL_ADDR_LITERAL, 2)),
                // Label
                Some((ByteCodes::COMPARE_ADDR_LITERAL_CONST, 2)),
                // Address at label
                Some((ByteCodes::COMPARE_ADDR_LITERAL_ADDR_LITERAL, 2)),
            ]),
        ])),

        "cmp4" => Some(ArgTable::Two(vec![
            // Register
            Some(vec![
                // Register
                None,
                // Address in register
                Some((ByteCodes::COMPARE_REG_ADDR_IN_REG, 4)),
                // Number
                Some((ByteCodes::COMPARE_REG_CONST, 4)), 
                // Address literal
                Some((ByteCodes::COMPARE_REG_ADDR_LITERAL, 4)),
                // Label
                Some((ByteCodes::COMPARE_REG_CONST, 4)),
                // Address at label
                Some((ByteCodes::COMPARE_REG_ADDR_LITERAL, 4)),
            ]),
            // Address in register
            Some(vec![
                // Register
                Some((ByteCodes::COMPARE_ADDR_IN_REG_REG, 4)), 
                // Address in register
                Some((ByteCodes::COMPARE_ADDR_IN_REG_ADDR_IN_REG, 4)), 
                // Number
                Some((ByteCodes::COMPARE_ADDR_IN_REG_CONST, 4)), 
                // Address literal
                Some((ByteCodes::COMPARE_ADDR_IN_REG_ADDR_LITERAL, 4)),
                // Label
                Some((ByteCodes::COMPARE_ADDR_IN_REG_CONST, 4)),
                // Address at label
                Some((ByteCodes::COMPARE_ADDR_IN_REG_ADDR_LITERAL, 4)),
            ]),
            // Number
            Some(vec![
                // Register
                Some((ByteCodes::COMPARE_CONST_REG, 4)), 
                // Address in register
                Some((ByteCodes::COMPARE_CONST_ADDR_IN_REG, 4)),
                // Number
                Some((ByteCodes::COMPARE_CONST_CONST, 4)), 
                // Address literal
                Some((ByteCodes::COMPARE_CONST_ADDR_LITERAL, 4)),
                // Label
                Some((ByteCodes::COMPARE_CONST_CONST, 4)),
                // Address at label
                Some((ByteCodes::COMPARE_CONST_ADDR_LITERAL, 4)),
            ]),
            // Address literal
            Some(vec![
                // Register
                Some((ByteCodes::COMPARE_ADDR_LITERAL_REG, 4)), 
                // Address in register
                Some((ByteCodes::COMPARE_ADDR_LITERAL_ADDR_IN_REG, 4)),
                // Number
                Some((ByteCodes::COMPARE_ADDR_LITERAL_CONST, 4)), 
                // Address literal
                Some((ByteCodes::COMPARE_ADDR_LITERAL_ADDR_LITERAL, 4)),
                // Label
                Some((ByteCodes::COMPARE_ADDR_LITERAL_CONST, 4)),
                // Address at label
                Some((ByteCodes::COMPARE_ADDR_LITERAL_ADDR_LITERAL, 4)),
            ]),
            // Label
            Some(vec![
                // Register
                Some((ByteCodes::COMPARE_CONST_REG, 4)), 
                // Address in register
                Some((ByteCodes::COMPARE_CONST_ADDR_IN_REG, 4)),
                // Number
                Some((ByteCodes::COMPARE_CONST_CONST, 4)), 
                // Address literal
                Some((ByteCodes::COMPARE_CONST_ADDR_LITERAL, 4)),
                // Label
                Some((ByteCodes::COMPARE_CONST_CONST, 4)),
                // Address at label
                Some((ByteCodes::COMPARE_CONST_ADDR_LITERAL, 4)),
            ]),
            // Address at label
            Some(vec![
                // Register
                Some((ByteCodes::COMPARE_ADDR_LITERAL_REG, 4)), 
                // Address in register
                Some((ByteCodes::COMPARE_ADDR_LITERAL_ADDR_IN_REG, 4)),
                // Number
                Some((ByteCodes::COMPARE_ADDR_LITERAL_CONST, 4)), 
                // Address literal
                Some((ByteCodes::COMPARE_ADDR_LITERAL_ADDR_LITERAL, 4)),
                // Label
                Some((ByteCodes::COMPARE_ADDR_LITERAL_CONST, 4)),
                // Address at label
                Some((ByteCodes::COMPARE_ADDR_LITERAL_ADDR_LITERAL, 4)),
            ]),
        ])),

        "cmp8" => Some(ArgTable::Two(vec![
            // Register
            Some(vec![
                // Register
                None, 
                // Address in register
                Some((ByteCodes::COMPARE_REG_ADDR_IN_REG, 8)),
                // Number
                Some((ByteCodes::COMPARE_REG_CONST, 8)), 
                // Address literal
                Some((ByteCodes::COMPARE_REG_ADDR_LITERAL, 8)),
                // Label
                Some((ByteCodes::COMPARE_REG_CONST, 8)),
                // Address at label
                Some((ByteCodes::COMPARE_REG_ADDR_LITERAL, 8)),
            ]),
            // Address in register
            Some(vec![
                // Register
                Some((ByteCodes::COMPARE_ADDR_IN_REG_REG, 8)), 
                // Address in register
                Some((ByteCodes::COMPARE_ADDR_IN_REG_ADDR_IN_REG, 8)), 
                // Number
                Some((ByteCodes::COMPARE_ADDR_IN_REG_CONST, 8)), 
                // Address literal
                Some((ByteCodes::COMPARE_ADDR_IN_REG_ADDR_LITERAL, 8)),
                // Label
                Some((ByteCodes::COMPARE_ADDR_IN_REG_CONST, 8)),
                // Address at label
                Some((ByteCodes::COMPARE_ADDR_IN_REG_ADDR_LITERAL, 8)),
            ]),
            // Number
            Some(vec![
                // Register
                Some((ByteCodes::COMPARE_CONST_REG, 8)),
                // Address in register
                Some((ByteCodes::COMPARE_CONST_ADDR_IN_REG, 8)),
                // Number
                Some((ByteCodes::COMPARE_CONST_CONST, 8)), 
                // Address literal
                Some((ByteCodes::COMPARE_CONST_ADDR_LITERAL, 8)),
                // Label
                Some((ByteCodes::COMPARE_CONST_CONST, 8)),
                // Address at label
                Some((ByteCodes::COMPARE_CONST_ADDR_LITERAL, 8)),
            ]),
            // Address literal
            Some(vec![
                // Register
                Some((ByteCodes::COMPARE_ADDR_LITERAL_REG, 8)),
                // Address in register
                Some((ByteCodes::COMPARE_ADDR_LITERAL_ADDR_IN_REG, 8)),
                // Number
                Some((ByteCodes::COMPARE_ADDR_LITERAL_CONST, 8)), 
                // Address literal
                Some((ByteCodes::COMPARE_ADDR_LITERAL_ADDR_LITERAL, 8)),
                // Label
                Some((ByteCodes::COMPARE_ADDR_LITERAL_CONST, 8)),
                // Address at label
                Some((ByteCodes::COMPARE_ADDR_LITERAL_ADDR_LITERAL, 8)),
            ]),
            // Label
            Some(vec![
                // Register
                Some((ByteCodes::COMPARE_CONST_REG, 8)),
                // Address in register
                Some((ByteCodes::COMPARE_CONST_ADDR_IN_REG, 8)),
                // Number
                Some((ByteCodes::COMPARE_CONST_CONST, 8)), 
                // Address literal
                Some((ByteCodes::COMPARE_CONST_ADDR_LITERAL, 8)),
                // Label
                Some((ByteCodes::COMPARE_CONST_CONST, 8)),
                // Address at label
                Some((ByteCodes::COMPARE_CONST_ADDR_LITERAL, 8)),
            ]),
            // Address at label
            Some(vec![
                // Register
                Some((ByteCodes::COMPARE_ADDR_LITERAL_REG, 8)),
                // Address in register
                Some((ByteCodes::COMPARE_ADDR_LITERAL_ADDR_IN_REG, 8)),
                // Number
                Some((ByteCodes::COMPARE_ADDR_LITERAL_CONST, 8)), 
                // Address literal
                Some((ByteCodes::COMPARE_ADDR_LITERAL_ADDR_LITERAL, 8)),
                // Label
                Some((ByteCodes::COMPARE_ADDR_LITERAL_CONST, 8)),
                // Address at label
                Some((ByteCodes::COMPARE_ADDR_LITERAL_ADDR_LITERAL, 8)),
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

