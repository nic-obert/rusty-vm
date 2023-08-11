use std::path::Path;
use std::u8;

use rust_vm_lib::byte_code::ByteCodes;
use rust_vm_lib::registers::REGISTER_ID_SIZE;
use rust_vm_lib::registers::REGISTER_SIZE;
use rust_vm_lib::token::TokenTypes;
use rust_vm_lib::token::Token;
use rust_vm_lib::vm::ADDRESS_SIZE;

use crate::error;


pub struct Operation {
    pub instruction: ByteCodes,
    /// Variable amount of bytes handled by the instruction
    pub handled_size: u8,
    /// Size of the instruction arguments in bytes
    pub total_arg_size: u8,
}


impl Operation {

    pub const fn new(instruction: ByteCodes, handled_size: u8, arg_size: usize) -> Self {
        debug_assert!(handled_size <= REGISTER_SIZE as u8);
        debug_assert!(arg_size <= u8::MAX as usize);

        Self {
            instruction,
            handled_size,
            // If handled_size is 0, the instruction doesn't need to specify the handled size in its arguments
            total_arg_size: arg_size as u8 + if handled_size == 0 { 0 } else { 1 },
        }
    }

}


type OneArgument = [Option<Operation>; 6];
type TwoArguments = [Option<OneArgument>; 6];


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
    pub fn get_operation(&self, operator_name: &str, operands: &[Token], unit_path: &Path, line_number: usize, line: &str) -> &Operation {

        match self {
            
            ArgTable::Zero(op) => {

                // The operator requires zero arguments
                if !operands.is_empty() {
                    error::invalid_arg_number(unit_path, operands.len(), 0, line_number, line, operator_name);
                }

                op
            },

            ArgTable::One(required_arg) => {

                // The operator requires one argument
                if operands.len() != 1 {
                    error::invalid_arg_number(unit_path, operands.len(), 1, line_number, line, operator_name);
                }

                // Return the instruction and handled size for the given operand
                required_arg.get(operands[0].value.to_ordinal() as usize).unwrap_or_else(
                    || error::invalid_token_argument(unit_path, operator_name, &operands[0], line_number, line, &self.get_possible_combinations())
                ).as_ref().unwrap_or_else(
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
                ).as_ref().unwrap_or_else(
                    || error::invalid_token_argument(unit_path, operator_name, &operands[1], line_number, line, &self.get_possible_combinations())
                )
            
            },

        }

    }

}


// Defining argument tables for all the assembly operators

const ADD_ARGS: ArgTable = ArgTable::Zero(Operation::new(ByteCodes::ADD, 0, 0));

const SUB_ARGS: ArgTable = ArgTable::Zero(Operation::new(ByteCodes::SUB, 0, 0));

const MUL_ARGS: ArgTable = ArgTable::Zero(Operation::new(ByteCodes::MUL, 0, 0));

const DIV_ARGS: ArgTable = ArgTable::Zero(Operation::new(ByteCodes::DIV, 0, 0));

const MOD_ARGS: ArgTable = ArgTable::Zero(Operation::new(ByteCodes::MOD, 0, 0));

const INC_ARGS: ArgTable = ArgTable::One([ 
    // Register
    Some(Operation::new(ByteCodes::INC_REG, 0, REGISTER_ID_SIZE)),
    // Address in register
    None,
    // Number
    None,
    // Address literal
    None,
    // Label
    None,
    // Address at label
    None,
]);

const INC1_ARGS: ArgTable = ArgTable::One([
    // Register
    None,
    // Address in register
    Some(Operation::new(ByteCodes::INC_ADDR_IN_REG, 1, REGISTER_ID_SIZE)),
    // Number
    None,
    // Address literal
    Some(Operation::new(ByteCodes::INC_ADDR_LITERAL, 1, ADDRESS_SIZE)),
    // Label
    None,
    // Address at label
    Some(Operation::new(ByteCodes::INC_ADDR_LITERAL, 1, ADDRESS_SIZE)),
]);

const INC2_ARGS: ArgTable = ArgTable::One([
    // Register
    None,
    // Address in register
    Some(Operation::new(ByteCodes::INC_ADDR_IN_REG, 2, REGISTER_ID_SIZE)),
    // Number
    None,
    // Address literal
    Some(Operation::new(ByteCodes::INC_ADDR_LITERAL, 2, ADDRESS_SIZE)),
    // Label
    None,
    // Address at label
    Some(Operation::new(ByteCodes::INC_ADDR_LITERAL, 2, ADDRESS_SIZE)),
]);

const INC4_ARGS: ArgTable = ArgTable::One([
    // Register
    None,
    // Address in register
    Some(Operation::new(ByteCodes::INC_ADDR_IN_REG, 4, REGISTER_ID_SIZE)),
    // Number
    None,
    // Address literal
    Some(Operation::new(ByteCodes::INC_ADDR_LITERAL, 4, ADDRESS_SIZE)),
    // Label
    None,
    // Address at label
    Some(Operation::new(ByteCodes::INC_ADDR_LITERAL, 4, ADDRESS_SIZE)),
]);

const INC8_ARGS: ArgTable = ArgTable::One([
    // Register
    None,
    // Address in register
    Some(Operation::new(ByteCodes::INC_ADDR_IN_REG, 8, REGISTER_ID_SIZE)),
    // Number
    None,
    // Address literal
    Some(Operation::new(ByteCodes::INC_ADDR_LITERAL, 8, ADDRESS_SIZE)),
    // Label
    None,
    // Address at label
    Some(Operation::new(ByteCodes::INC_ADDR_LITERAL, 8, ADDRESS_SIZE)),
]);

const DEC_ARGS: ArgTable = ArgTable::One([
    // Register
    Some(Operation::new(ByteCodes::DEC_REG, 0, REGISTER_ID_SIZE)),
    // Address in register
    None,
    // Number
    None,
    // Address literal
    None,
    // Label
    None,
    // Address at label
    None,
]);

const DEC1_ARGS: ArgTable = ArgTable::One([
    // Register
    None,
    // Address in register
    Some(Operation::new(ByteCodes::DEC_ADDR_IN_REG, 1, REGISTER_ID_SIZE)),
    // Number
    None,
    // Address literal
    Some(Operation::new(ByteCodes::DEC_ADDR_LITERAL, 1, ADDRESS_SIZE)),
    // Label
    None,
    // Address at label
    Some(Operation::new(ByteCodes::DEC_ADDR_LITERAL, 1, ADDRESS_SIZE)),
]);

const DEC2_ARGS: ArgTable = ArgTable::One([
    // Register
    None,
    // Address in register
    Some(Operation::new(ByteCodes::DEC_ADDR_IN_REG, 2, REGISTER_ID_SIZE)),
    // Number
    None,
    // Address literal
    Some(Operation::new(ByteCodes::DEC_ADDR_LITERAL, 2, ADDRESS_SIZE)),
    // Label
    None,
    // Address at label
    Some(Operation::new(ByteCodes::DEC_ADDR_LITERAL, 2, ADDRESS_SIZE)),
]);

const DEC4_ARGS: ArgTable = ArgTable::One([
    // Register
    None, 
    // Address in register
    Some(Operation::new(ByteCodes::DEC_ADDR_IN_REG, 4, REGISTER_ID_SIZE)),
    // Number
    None, 
    // Address literal
    Some(Operation::new(ByteCodes::DEC_ADDR_LITERAL, 4, ADDRESS_SIZE)),
    // Label
    None,
    // Address at label
    Some(Operation::new(ByteCodes::DEC_ADDR_LITERAL, 4, ADDRESS_SIZE)),
]);

const DEC8_ARGS: ArgTable = ArgTable::One([
    // Register
    None,
    // Address in register
    Some(Operation::new(ByteCodes::DEC_ADDR_IN_REG, 8, REGISTER_ID_SIZE)),
    // Number
    None, 
    // Address literal
    Some(Operation::new(ByteCodes::DEC_ADDR_LITERAL, 8, ADDRESS_SIZE)),
    // Label
    None,
    // Address at label
    Some(Operation::new(ByteCodes::DEC_ADDR_LITERAL, 8, ADDRESS_SIZE)),
]);

const NOP_ARGS: ArgTable = ArgTable::Zero(Operation::new(ByteCodes::NO_OPERATION, 0, 0));

const MOV_ARGS: ArgTable = ArgTable::Two([
    // Register
    Some([
        // Register
        Some(Operation::new(ByteCodes::MOVE_INTO_REG_FROM_REG, 0, REGISTER_ID_SIZE + REGISTER_ID_SIZE)),
        // Address in register
        None,
        // Number
        None,
        // Address literal
        None,
        // Label
        None,
        // Address at label
        None,
    ]),
    // Address in register
    None,
    // Number
    None,
    // Address literal
    None,
    // Label
    None,
    // Address at label
    None,
]);

const MOV1_ARGS: ArgTable = ArgTable::Two([
    // Register
    Some([
        // Register
        None,
        // Address in register
        Some(Operation::new(ByteCodes::MOVE_INTO_REG_FROM_ADDR_IN_REG, 1, REGISTER_ID_SIZE + REGISTER_ID_SIZE)),
        // Number
        Some(Operation::new(ByteCodes::MOVE_INTO_REG_FROM_CONST, 1, REGISTER_ID_SIZE + 1)),
        // Address literal
        Some(Operation::new(ByteCodes::MOVE_INTO_REG_FROM_ADDR_LITERAL, 1, REGISTER_ID_SIZE + ADDRESS_SIZE)),
        // Label
        None,
        // Address at label
        Some(Operation::new(ByteCodes::MOVE_INTO_REG_FROM_ADDR_LITERAL, 1, REGISTER_ID_SIZE + ADDRESS_SIZE)),
    ]),
    // Address in register
    Some([
        // Register
        Some(Operation::new(ByteCodes::MOVE_INTO_ADDR_IN_REG_FROM_REG, 1, REGISTER_ID_SIZE + REGISTER_ID_SIZE)),
        // Address in register
        Some(Operation::new(ByteCodes::MOVE_INTO_ADDR_IN_REG_FROM_ADDR_IN_REG, 1, REGISTER_ID_SIZE + REGISTER_ID_SIZE)),
        // Number
        Some(Operation::new(ByteCodes::MOVE_INTO_ADDR_IN_REG_FROM_CONST, 1, REGISTER_ID_SIZE + 1)), 
        // Address literal
        Some(Operation::new(ByteCodes::MOVE_INTO_ADDR_IN_REG_FROM_ADDR_LITERAL, 1, REGISTER_ID_SIZE + ADDRESS_SIZE)),
        // Label
        None,
        // Address at label
        Some(Operation::new(ByteCodes::MOVE_INTO_ADDR_IN_REG_FROM_ADDR_LITERAL, 1, REGISTER_ID_SIZE + ADDRESS_SIZE)),
    ]),
    // Number
    None,
    // Address literal
    Some([
        // Register
        Some(Operation::new(ByteCodes::MOVE_INTO_ADDR_LITERAL_FROM_REG, 1, ADDRESS_SIZE + REGISTER_ID_SIZE)),
        // Address in register
        Some(Operation::new(ByteCodes::MOVE_INTO_ADDR_LITERAL_FROM_ADDR_IN_REG, 1, ADDRESS_SIZE + REGISTER_ID_SIZE)),
        // Number
        Some(Operation::new(ByteCodes::MOVE_INTO_ADDR_LITERAL_FROM_CONST, 1, ADDRESS_SIZE + 1)),
        // Address literal
        Some(Operation::new(ByteCodes::MOVE_INTO_ADDR_LITERAL_FROM_ADDR_LITERAL, 1, ADDRESS_SIZE + ADDRESS_SIZE)),
        // Label
        None,
        // Address at label
        Some(Operation::new(ByteCodes::MOVE_INTO_ADDR_LITERAL_FROM_ADDR_LITERAL, 1, ADDRESS_SIZE + ADDRESS_SIZE)),
    ]),
    // Label
    None,
    // Address at label
    Some([
        // Register
        Some(Operation::new(ByteCodes::MOVE_INTO_ADDR_LITERAL_FROM_REG, 1, ADDRESS_SIZE + REGISTER_ID_SIZE)),
        // Address in register
        Some(Operation::new(ByteCodes::MOVE_INTO_ADDR_LITERAL_FROM_ADDR_IN_REG, 1, ADDRESS_SIZE + REGISTER_ID_SIZE)),
        // Number
        Some(Operation::new(ByteCodes::MOVE_INTO_ADDR_LITERAL_FROM_CONST, 1, ADDRESS_SIZE + 1)),
        // Address literal
        Some(Operation::new(ByteCodes::MOVE_INTO_ADDR_LITERAL_FROM_ADDR_LITERAL, 1, ADDRESS_SIZE + ADDRESS_SIZE)),
        // Label
        None,
        // Address at label
        Some(Operation::new(ByteCodes::MOVE_INTO_ADDR_LITERAL_FROM_ADDR_LITERAL, 1, ADDRESS_SIZE + ADDRESS_SIZE)),
    ]),
]);

const MOV2_ARGS: ArgTable = ArgTable::Two([
    // Register
    Some([
        // Register
        None, 
        // Address in register
        Some(Operation::new(ByteCodes::MOVE_INTO_REG_FROM_ADDR_IN_REG, 2, REGISTER_ID_SIZE + REGISTER_ID_SIZE)),
        // Number
        Some(Operation::new(ByteCodes::MOVE_INTO_REG_FROM_CONST, 2, REGISTER_ID_SIZE + 2)), 
        // Address literal
        Some(Operation::new(ByteCodes::MOVE_INTO_REG_FROM_ADDR_LITERAL, 2, REGISTER_ID_SIZE + ADDRESS_SIZE)), 
        // Label
        None,
        // Address at label
        Some(Operation::new(ByteCodes::MOVE_INTO_REG_FROM_ADDR_LITERAL, 2, REGISTER_ID_SIZE + ADDRESS_SIZE)),
    ]),
    // Address in register
    Some([
        // Register
        Some(Operation::new(ByteCodes::MOVE_INTO_ADDR_IN_REG_FROM_REG, 2, REGISTER_ID_SIZE + REGISTER_ID_SIZE)),
        // Address in register
        Some(Operation::new(ByteCodes::MOVE_INTO_ADDR_IN_REG_FROM_ADDR_IN_REG, 2, REGISTER_ID_SIZE + REGISTER_ID_SIZE)), 
        // Number
        Some(Operation::new(ByteCodes::MOVE_INTO_ADDR_IN_REG_FROM_CONST, 2, REGISTER_ID_SIZE + 2)), 
        // Address literal
        Some(Operation::new(ByteCodes::MOVE_INTO_ADDR_IN_REG_FROM_ADDR_LITERAL, 2, REGISTER_ID_SIZE + ADDRESS_SIZE)), 
        // Label
        None,
        // Address at label
        Some(Operation::new(ByteCodes::MOVE_INTO_ADDR_IN_REG_FROM_ADDR_LITERAL, 2, REGISTER_ID_SIZE + ADDRESS_SIZE)),
    ]),
    // Number
    None,
    // Address literal
    Some([
        // Register
        Some(Operation::new(ByteCodes::MOVE_INTO_ADDR_LITERAL_FROM_REG, 2, ADDRESS_SIZE + REGISTER_ID_SIZE)),
        // Address in register
        Some(Operation::new(ByteCodes::MOVE_INTO_ADDR_LITERAL_FROM_ADDR_IN_REG, 2, ADDRESS_SIZE + REGISTER_ID_SIZE)),
        // Number
        Some(Operation::new(ByteCodes::MOVE_INTO_ADDR_LITERAL_FROM_CONST, 2, ADDRESS_SIZE + 2)), 
        // Address literal
        Some(Operation::new(ByteCodes::MOVE_INTO_ADDR_LITERAL_FROM_ADDR_LITERAL, 2, ADDRESS_SIZE + ADDRESS_SIZE)), 
        // Label
        None,
        // Address at label
        Some(Operation::new(ByteCodes::MOVE_INTO_ADDR_LITERAL_FROM_ADDR_LITERAL, 2, ADDRESS_SIZE + ADDRESS_SIZE)),
    ]),
    // Label
    None,
    // Address at label
    Some([
        // Register
        Some(Operation::new(ByteCodes::MOVE_INTO_ADDR_LITERAL_FROM_REG, 2, ADDRESS_SIZE + REGISTER_ID_SIZE)),
        // Address in register
        Some(Operation::new(ByteCodes::MOVE_INTO_ADDR_LITERAL_FROM_ADDR_IN_REG, 2, ADDRESS_SIZE + REGISTER_ID_SIZE)),
        // Number
        Some(Operation::new(ByteCodes::MOVE_INTO_ADDR_LITERAL_FROM_CONST, 2, ADDRESS_SIZE + 2)), 
        // Address literal
        Some(Operation::new(ByteCodes::MOVE_INTO_ADDR_LITERAL_FROM_ADDR_LITERAL, 2, ADDRESS_SIZE + ADDRESS_SIZE)), 
        // Label
        None,
        // Address at label
        Some(Operation::new(ByteCodes::MOVE_INTO_ADDR_LITERAL_FROM_ADDR_LITERAL, 2, ADDRESS_SIZE + ADDRESS_SIZE)),
    ]),
]);

const MOV4_ARGS: ArgTable = ArgTable::Two([
    // Register
    Some([
        // Register
        None, 
        // Address in register
        Some(Operation::new(ByteCodes::MOVE_INTO_REG_FROM_ADDR_IN_REG, 4, REGISTER_ID_SIZE + REGISTER_ID_SIZE)), 
        // Number
        Some(Operation::new(ByteCodes::MOVE_INTO_REG_FROM_CONST, 4, REGISTER_ID_SIZE + 4)), 
        // Address literal
        Some(Operation::new(ByteCodes::MOVE_INTO_REG_FROM_ADDR_LITERAL, 4, REGISTER_ID_SIZE + ADDRESS_SIZE)), 
        // Label
        None,
        // Address at label
        Some(Operation::new(ByteCodes::MOVE_INTO_REG_FROM_ADDR_LITERAL, 4, REGISTER_ID_SIZE + ADDRESS_SIZE)),
    ]),
    // Address in register
    Some([
        // Register
        Some(Operation::new(ByteCodes::MOVE_INTO_ADDR_IN_REG_FROM_REG, 4, REGISTER_ID_SIZE + REGISTER_ID_SIZE)), 
        // Address in register
        Some(Operation::new(ByteCodes::MOVE_INTO_ADDR_IN_REG_FROM_ADDR_IN_REG, 4, REGISTER_ID_SIZE + REGISTER_ID_SIZE)), 
        // Number
        Some(Operation::new(ByteCodes::MOVE_INTO_ADDR_IN_REG_FROM_CONST, 4, REGISTER_ID_SIZE + 4)),
        // Address literal
        Some(Operation::new(ByteCodes::MOVE_INTO_ADDR_IN_REG_FROM_ADDR_LITERAL, 4, REGISTER_ID_SIZE + ADDRESS_SIZE)), 
        // Label
        None,
        // Address at label
        Some(Operation::new(ByteCodes::MOVE_INTO_ADDR_IN_REG_FROM_ADDR_LITERAL, 4, REGISTER_ID_SIZE + ADDRESS_SIZE)),
    ]),
    // Number
    None, 
    // Address literal
    Some([
        // Register
        Some(Operation::new(ByteCodes::MOVE_INTO_ADDR_LITERAL_FROM_REG, 4, ADDRESS_SIZE + REGISTER_ID_SIZE)),
        // Address in register
        Some(Operation::new(ByteCodes::MOVE_INTO_ADDR_LITERAL_FROM_ADDR_IN_REG, 4, ADDRESS_SIZE + REGISTER_ID_SIZE)), 
        // Number
        Some(Operation::new(ByteCodes::MOVE_INTO_ADDR_LITERAL_FROM_CONST, 4, ADDRESS_SIZE + 4)), 
        // Address literal
        Some(Operation::new(ByteCodes::MOVE_INTO_ADDR_LITERAL_FROM_ADDR_LITERAL, 4, ADDRESS_SIZE + ADDRESS_SIZE)), 
        // Label
        None,
        // Address at label
        Some(Operation::new(ByteCodes::MOVE_INTO_ADDR_LITERAL_FROM_ADDR_LITERAL, 4, ADDRESS_SIZE + ADDRESS_SIZE)),
    ]),
    // Label
    None,
    // Address at label
    Some([
        // Register
        Some(Operation::new(ByteCodes::MOVE_INTO_ADDR_LITERAL_FROM_REG, 4, ADDRESS_SIZE + REGISTER_ID_SIZE)),
        // Address in register
        Some(Operation::new(ByteCodes::MOVE_INTO_ADDR_LITERAL_FROM_ADDR_IN_REG, 4, ADDRESS_SIZE + REGISTER_ID_SIZE)), 
        // Number
        Some(Operation::new(ByteCodes::MOVE_INTO_ADDR_LITERAL_FROM_CONST, 4, ADDRESS_SIZE + 4)), 
        // Address literal
        Some(Operation::new(ByteCodes::MOVE_INTO_ADDR_LITERAL_FROM_ADDR_LITERAL, 4, ADDRESS_SIZE + ADDRESS_SIZE)), 
        // Label
        None,
        // Address at label
        Some(Operation::new(ByteCodes::MOVE_INTO_ADDR_LITERAL_FROM_ADDR_LITERAL, 4, ADDRESS_SIZE + ADDRESS_SIZE)),
    ]),
]);

const MOV8_ARGS: ArgTable = ArgTable::Two([
    // Register
    Some([
        // Register
        None, 
        // Address in register
        Some(Operation::new(ByteCodes::MOVE_INTO_REG_FROM_ADDR_IN_REG, 8, REGISTER_ID_SIZE + REGISTER_ID_SIZE)), 
        // Number
        Some(Operation::new(ByteCodes::MOVE_INTO_REG_FROM_CONST, 8, REGISTER_ID_SIZE + 8)), 
        // Address literal
        Some(Operation::new(ByteCodes::MOVE_INTO_REG_FROM_ADDR_LITERAL, 8, REGISTER_ID_SIZE + ADDRESS_SIZE)), 
        // Label
        Some(Operation::new(ByteCodes::MOVE_INTO_REG_FROM_CONST, 8, REGISTER_ID_SIZE + ADDRESS_SIZE)),
        // Address at label
        Some(Operation::new(ByteCodes::MOVE_INTO_REG_FROM_ADDR_LITERAL, 8, REGISTER_ID_SIZE + ADDRESS_SIZE)),
    ]),
    // Address in register
    Some([
        // Register
        Some(Operation::new(ByteCodes::MOVE_INTO_ADDR_IN_REG_FROM_REG, 8, REGISTER_ID_SIZE + REGISTER_ID_SIZE)),
        // Address in register
        Some(Operation::new(ByteCodes::MOVE_INTO_ADDR_IN_REG_FROM_ADDR_IN_REG, 8, REGISTER_ID_SIZE + REGISTER_ID_SIZE)), 
        // Number
        Some(Operation::new(ByteCodes::MOVE_INTO_ADDR_IN_REG_FROM_CONST, 8, REGISTER_ID_SIZE + 8)), 
        // Address literal
        Some(Operation::new(ByteCodes::MOVE_INTO_ADDR_IN_REG_FROM_ADDR_LITERAL, 8, REGISTER_ID_SIZE + ADDRESS_SIZE)),
        // Label
        Some(Operation::new(ByteCodes::MOVE_INTO_ADDR_IN_REG_FROM_CONST, 8, REGISTER_ID_SIZE + ADDRESS_SIZE)),
        // Address at label
        Some(Operation::new(ByteCodes::MOVE_INTO_ADDR_IN_REG_FROM_ADDR_LITERAL, 8, REGISTER_ID_SIZE + ADDRESS_SIZE)),
    ]),
    // Number
    None,
    // Address literal
    Some([
        // Register
        Some(Operation::new(ByteCodes::MOVE_INTO_ADDR_LITERAL_FROM_REG, 8, ADDRESS_SIZE + REGISTER_ID_SIZE)), 
        // Address in register
        Some(Operation::new(ByteCodes::MOVE_INTO_ADDR_LITERAL_FROM_ADDR_IN_REG, 8, ADDRESS_SIZE + REGISTER_ID_SIZE)), 
        // Number
        Some(Operation::new(ByteCodes::MOVE_INTO_ADDR_LITERAL_FROM_CONST, 8, ADDRESS_SIZE + 8)), 
        // Address literal
        Some(Operation::new(ByteCodes::MOVE_INTO_ADDR_LITERAL_FROM_ADDR_LITERAL, 8, ADDRESS_SIZE + ADDRESS_SIZE)),
        // Label
        Some(Operation::new(ByteCodes::MOVE_INTO_ADDR_LITERAL_FROM_CONST, 8, ADDRESS_SIZE + ADDRESS_SIZE)),
        // Address at label
        Some(Operation::new(ByteCodes::MOVE_INTO_ADDR_LITERAL_FROM_ADDR_LITERAL, 8, ADDRESS_SIZE + ADDRESS_SIZE)),
    ]),
    // Label
    None,
    // Address at label
    Some([
        // Register
        Some(Operation::new(ByteCodes::MOVE_INTO_ADDR_LITERAL_FROM_REG, 8, ADDRESS_SIZE + REGISTER_ID_SIZE)), 
        // Address in register
        Some(Operation::new(ByteCodes::MOVE_INTO_ADDR_LITERAL_FROM_ADDR_IN_REG, 8, ADDRESS_SIZE + REGISTER_ID_SIZE)), 
        // Number
        Some(Operation::new(ByteCodes::MOVE_INTO_ADDR_LITERAL_FROM_CONST, 8, ADDRESS_SIZE + 8)), 
        // Address literal
        Some(Operation::new(ByteCodes::MOVE_INTO_ADDR_LITERAL_FROM_ADDR_LITERAL, 8, ADDRESS_SIZE + ADDRESS_SIZE)),
        // Label
        Some(Operation::new(ByteCodes::MOVE_INTO_ADDR_LITERAL_FROM_CONST, 8, ADDRESS_SIZE + ADDRESS_SIZE)),
        // Address at label
        Some(Operation::new(ByteCodes::MOVE_INTO_ADDR_LITERAL_FROM_ADDR_LITERAL, 8, ADDRESS_SIZE + ADDRESS_SIZE)),
    ]),
]);

const PUSH_ARGS: ArgTable = ArgTable::One([
    // Register
    Some(Operation::new(ByteCodes::PUSH_FROM_REG, 0, REGISTER_ID_SIZE)),
    // Address in register
    None,
    // Number
    None,
    // Address literal
    None,
    // Label
    None,
    // Address at label
    None,       
]);

const PUSH1_ARGS: ArgTable = ArgTable::One([
    // Register
    None, 
    // Address in register
    Some(Operation::new(ByteCodes::PUSH_FROM_ADDR_IN_REG, 1, REGISTER_ID_SIZE)), 
    // Number
    Some(Operation::new(ByteCodes::PUSH_FROM_CONST, 1, 1)),
    // Address literal
    Some(Operation::new(ByteCodes::PUSH_FROM_ADDR_LITERAL, 1, ADDRESS_SIZE)), 
    // Label
    None,
    // Address at label
    Some(Operation::new(ByteCodes::PUSH_FROM_ADDR_LITERAL, 1, ADDRESS_SIZE)),
]);

const PUSH2_ARGS: ArgTable = ArgTable::One([
    // Register
    None, 
    // Address in register
    Some(Operation::new(ByteCodes::PUSH_FROM_ADDR_IN_REG, 2, REGISTER_ID_SIZE)), 
    // Number
    Some(Operation::new(ByteCodes::PUSH_FROM_CONST, 2, 2)),
    // Address literal
    Some(Operation::new(ByteCodes::PUSH_FROM_ADDR_LITERAL, 2, ADDRESS_SIZE)), 
    // Label
    None,
    // Address at label
    Some(Operation::new(ByteCodes::PUSH_FROM_ADDR_LITERAL, 2, ADDRESS_SIZE)),
]);

const PUSH4_ARGS: ArgTable = ArgTable::One([
    // Register
    None, 
    // Address in register
    Some(Operation::new(ByteCodes::PUSH_FROM_ADDR_IN_REG, 4, REGISTER_ID_SIZE)), 
    // Number
    Some(Operation::new(ByteCodes::PUSH_FROM_CONST, 4, 4)), 
    // Address literal
    Some(Operation::new(ByteCodes::PUSH_FROM_ADDR_LITERAL, 4, ADDRESS_SIZE)),
    // Label
    None,
    // Address at label
    Some(Operation::new(ByteCodes::PUSH_FROM_ADDR_LITERAL, 4, ADDRESS_SIZE)),
]);

const PUSH8_ARGS: ArgTable = ArgTable::One([
    // Register
    None, 
    // Address in register
    Some(Operation::new(ByteCodes::PUSH_FROM_ADDR_IN_REG, 8, REGISTER_ID_SIZE)), 
    // Number
    Some(Operation::new(ByteCodes::PUSH_FROM_CONST, 8, 8)), 
    // Address literal
    Some(Operation::new(ByteCodes::PUSH_FROM_ADDR_LITERAL, 8, ADDRESS_SIZE)), 
    // Label
    Some(Operation::new(ByteCodes::PUSH_FROM_CONST, 8, ADDRESS_SIZE)),
    // Address at label
    Some(Operation::new(ByteCodes::PUSH_FROM_ADDR_LITERAL, 8, ADDRESS_SIZE)),
]);

const POP1_ARGS: ArgTable = ArgTable::One([
    // Register
    Some(Operation::new(ByteCodes::POP_INTO_REG, 1, REGISTER_ID_SIZE)), 
    // Address in register
    Some(Operation::new(ByteCodes::POP_INTO_ADDR_IN_REG, 1, REGISTER_ID_SIZE)), 
    // Number
    None, 
    // Address literal
    Some(Operation::new(ByteCodes::POP_INTO_ADDR_LITERAL, 1, ADDRESS_SIZE)), 
    // Label
    None,
    // Address at label
    Some(Operation::new(ByteCodes::POP_INTO_ADDR_LITERAL, 1, ADDRESS_SIZE)),
]);

const POP2_ARGS: ArgTable = ArgTable::One([
    // Register
    Some(Operation::new(ByteCodes::POP_INTO_REG, 2, REGISTER_ID_SIZE)),
    // Address in register
    Some(Operation::new(ByteCodes::POP_INTO_ADDR_IN_REG, 2, REGISTER_ID_SIZE)), 
    // Number
    None, 
    // Address literal
    Some(Operation::new(ByteCodes::POP_INTO_ADDR_LITERAL, 2, ADDRESS_SIZE)), 
    // Label
    None,
    // Address at label
    Some(Operation::new(ByteCodes::POP_INTO_ADDR_LITERAL, 2, ADDRESS_SIZE)),
]);

const POP4_ARGS: ArgTable = ArgTable::One([
    // Register
    Some(Operation::new(ByteCodes::POP_INTO_REG, 4, REGISTER_ID_SIZE)), 
    // Address in register
    Some(Operation::new(ByteCodes::POP_INTO_ADDR_IN_REG, 4, REGISTER_ID_SIZE)),
    // Number
    None, 
    // Address literal
    Some(Operation::new(ByteCodes::POP_INTO_ADDR_LITERAL, 4, ADDRESS_SIZE)), 
    // Label
    None,
    // Address at label
    Some(Operation::new(ByteCodes::POP_INTO_ADDR_LITERAL, 4, ADDRESS_SIZE)),
]);

const POP8_ARGS: ArgTable = ArgTable::One([
    // Register
    Some(Operation::new(ByteCodes::POP_INTO_REG, 8, REGISTER_ID_SIZE)), 
    // Address in register
    Some(Operation::new(ByteCodes::POP_INTO_ADDR_IN_REG, 8, REGISTER_ID_SIZE)), 
    // Number
    None, 
    // Address literal
    Some(Operation::new(ByteCodes::POP_INTO_ADDR_LITERAL, 8, ADDRESS_SIZE)), 
    // Label
    None,
    // Address at label
    Some(Operation::new(ByteCodes::POP_INTO_ADDR_LITERAL, 8, ADDRESS_SIZE)),
]);

const JMP_ARGS: ArgTable = ArgTable::One([
    // Register
    None,
    // Address in register
    None,
    // Number
    None,
    // Address literal
    None,
    // Label
    Some(Operation::new(ByteCodes::JUMP, 0, ADDRESS_SIZE)),
    // Address at label
    None
]);

const JMPNZ_ARGS: ArgTable = ArgTable::One([
    // Register
    None, 
    // Address in register
    None, 
    // Number
    None,
    // Address literal
    None,
    // Label
    Some(Operation::new(ByteCodes::JUMP_NOT_ZERO, 0, ADDRESS_SIZE)),
    // Address at label
    None,
]);

const JMPZ_ARGS: ArgTable = ArgTable::One([
    // Register
    None,
    // Address in register
    None,
    // Number
    None,
    // Address literal
    None,
    // Label
    Some(Operation::new(ByteCodes::JUMP_ZERO, 0, ADDRESS_SIZE)),
    // Address at label
    None,
]);

const JMPGR_ARGS: ArgTable = ArgTable::One([
    // Register
    None,
    // Address in register
    None,
    // Number
    None,
    // Address literal
    None,
    // Label
    Some(Operation::new(ByteCodes::JUMP_GREATER, 0, ADDRESS_SIZE)),
    // Address at label
    None,
]);

const JMPGE_ARGS: ArgTable = ArgTable::One([
    // Register
    None,
    // Address in register
    None,
    // Number
    None,
    // Address literal
    None,
    // Label
    Some(Operation::new(ByteCodes::JUMP_GREATER_OR_EQUAL, 0, ADDRESS_SIZE)),
    // Address at label
    None,
]);

const JMPLT_ARGS: ArgTable = ArgTable::One([
    // Register
    None,
    // Address in register
    None,
    // Number
    None,
    // Address literal
    None,
    // Label
    Some(Operation::new(ByteCodes::JUMP_LESS, 0, ADDRESS_SIZE)),
    // Address at label
    None,
]);

const JMPLE_ARGS: ArgTable = ArgTable::One([
    // Register
    None,
    // Address in register
    None,
    // Number
    None,
    // Address literal
    None,
    // Label
    Some(Operation::new(ByteCodes::JUMP_LESS_OR_EQUAL, 0, ADDRESS_SIZE)),
    // Address at label
    None,
]);

const JMPOF: ArgTable = ArgTable::One([
    // Register
    None,
    // Address in register
    None,
    // Number
    None,
    // Address literal
    None,
    // Label
    Some(Operation::new(ByteCodes::JUMP_OVERFLOW, 0, ADDRESS_SIZE)),
    // Address at label
    None,
]);

const JMPNOF: ArgTable = ArgTable::One([
    // Register
    None,
    // Address in register
    None,
    // Number
    None,
    // Address literal
    None,
    // Label
    Some(Operation::new(ByteCodes::JUMP_NOT_OVERFLOW, 0, ADDRESS_SIZE)),
    // Address at label
    None,
]);

const JMPCR_ARGS: ArgTable = ArgTable::One([
    // Register
    None,
    // Address in register
    None,
    // Number
    None,
    // Address literal
    None,
    // Label
    Some(Operation::new(ByteCodes::JUMP_CARRY, 0, ADDRESS_SIZE)),
    // Address at label
    None,
]);

const JMPNCR_ARGS: ArgTable = ArgTable::One([
    // Register
    None,
    // Address in register
    None,
    // Number
    None,
    // Address literal
    None,
    // Label
    Some(Operation::new(ByteCodes::JUMP_NOT_CARRY, 0, ADDRESS_SIZE)),
    // Address at label
    None,
]);

const JMPSN_ARGS: ArgTable = ArgTable::One([
    // Register
    None,
    // Address in register
    None,
    // Number
    None,
    // Address literal
    None,
    // Label
    Some(Operation::new(ByteCodes::JUMP_SIGN, 0, ADDRESS_SIZE)),
    // Address at label
    None,
]);

const JMPNSN_ARGS: ArgTable = ArgTable::One([
    // Register
    None,
    // Address in register
    None,
    // Number
    None,
    // Address literal
    None,
    // Label
    Some(Operation::new(ByteCodes::JUMP_NOT_SIGN, 0, ADDRESS_SIZE)),
    // Address at label
    None,
]);

const CALL_ARGS: ArgTable = ArgTable::One([
    // Register
    None,
    // Address in register
    None,
    // Number
    None,
    // Address literal
    None,
    // Label
    Some(Operation::new(ByteCodes::CALL, 0, ADDRESS_SIZE)),
    // Address at label
    None,
]);

const RET_ARGS: ArgTable = ArgTable::Zero(Operation::new(ByteCodes::RETURN, 0, 0));

const CMP_ARGS: ArgTable = ArgTable::Two([
    // Register
    Some([
        // Register
        Some(Operation::new(ByteCodes::COMPARE_REG_REG, 0, REGISTER_ID_SIZE + REGISTER_ID_SIZE)),
        // Address in register
        None,
        // Number
        None,
        // Address literal
        None,
        // Label
        None,
        // Address at label
        None,
    ]),
    // Address in register
    None,
    // Number
    None,
    // Address literal
    None,
    // Label
    None,
    // Address at label
    None,
]);

const CMP1_ARGS: ArgTable = ArgTable::Two([
    // Register
    Some([
        // Register
        None, 
        // Address in register
        Some(Operation::new(ByteCodes::COMPARE_REG_ADDR_IN_REG, 1, REGISTER_ID_SIZE + REGISTER_ID_SIZE)), 
        // Number
        Some(Operation::new(ByteCodes::COMPARE_REG_CONST, 1, REGISTER_ID_SIZE + 1)), 
        // Address literal
        Some(Operation::new(ByteCodes::COMPARE_REG_ADDR_LITERAL, 1, REGISTER_ID_SIZE + ADDRESS_SIZE)),
        // Label
        None,
        // Address at label
        Some(Operation::new(ByteCodes::COMPARE_REG_ADDR_LITERAL, 1, REGISTER_ID_SIZE + ADDRESS_SIZE)),
    ]),
    // Address in register
    Some([
        // Register
        Some(Operation::new(ByteCodes::COMPARE_ADDR_IN_REG_REG, 1, REGISTER_ID_SIZE + REGISTER_ID_SIZE)), 
        // Address in register
        Some(Operation::new(ByteCodes::COMPARE_ADDR_IN_REG_ADDR_IN_REG, 1, REGISTER_ID_SIZE + REGISTER_ID_SIZE)), 
        // Number
        Some(Operation::new(ByteCodes::COMPARE_ADDR_IN_REG_CONST, 1, REGISTER_ID_SIZE + 1)), 
        // Address literal
        Some(Operation::new(ByteCodes::COMPARE_ADDR_IN_REG_ADDR_LITERAL, 1, REGISTER_ID_SIZE + ADDRESS_SIZE)),
        // Label
        None,
        // Address at label
        Some(Operation::new(ByteCodes::COMPARE_ADDR_IN_REG_ADDR_LITERAL, 1, REGISTER_ID_SIZE + ADDRESS_SIZE)),
    ]),
    // Number
    Some([
        // Register
        Some(Operation::new(ByteCodes::COMPARE_CONST_REG, 1, 1 + REGISTER_ID_SIZE)),
        // Address in register
        Some(Operation::new(ByteCodes::COMPARE_CONST_ADDR_IN_REG, 1, 1 + REGISTER_ID_SIZE)), 
        // Number
        Some(Operation::new(ByteCodes::COMPARE_CONST_CONST, 1, 1 + 1)),
        // Address literal
        Some(Operation::new(ByteCodes::COMPARE_CONST_ADDR_LITERAL, 1, 1 + ADDRESS_SIZE)),
        // Label
        None,
        // Address at label
        Some(Operation::new(ByteCodes::COMPARE_CONST_ADDR_LITERAL, 1, 1 + ADDRESS_SIZE)),
    ]),
    // Address literal
    Some([
        // Register
        Some(Operation::new(ByteCodes::COMPARE_ADDR_LITERAL_REG, 1, ADDRESS_SIZE + REGISTER_ID_SIZE)),
        // Address in register
        Some(Operation::new(ByteCodes::COMPARE_ADDR_LITERAL_ADDR_IN_REG, 1, ADDRESS_SIZE + REGISTER_ID_SIZE)),
        // Number
        Some(Operation::new(ByteCodes::COMPARE_ADDR_LITERAL_CONST, 1, ADDRESS_SIZE + 1)),
        // Address literal
        Some(Operation::new(ByteCodes::COMPARE_ADDR_LITERAL_ADDR_LITERAL, 1, ADDRESS_SIZE + ADDRESS_SIZE)),
        // Label
        None,
        // Address at label
        Some(Operation::new(ByteCodes::COMPARE_ADDR_LITERAL_ADDR_LITERAL, 1, ADDRESS_SIZE + ADDRESS_SIZE)),
    ]),
    // Label
    None,
    // Address at label
    Some([
        // Register
        Some(Operation::new(ByteCodes::COMPARE_ADDR_LITERAL_REG, 1, ADDRESS_SIZE + REGISTER_ID_SIZE)),
        // Address in register
        Some(Operation::new(ByteCodes::COMPARE_ADDR_LITERAL_ADDR_IN_REG, 1, ADDRESS_SIZE + REGISTER_ID_SIZE)),
        // Number
        Some(Operation::new(ByteCodes::COMPARE_ADDR_LITERAL_CONST, 1, ADDRESS_SIZE + 1)),
        // Address literal
        Some(Operation::new(ByteCodes::COMPARE_ADDR_LITERAL_ADDR_LITERAL, 1, ADDRESS_SIZE + ADDRESS_SIZE)),
        // Label
        None,
        // Address at label
        Some(Operation::new(ByteCodes::COMPARE_ADDR_LITERAL_ADDR_LITERAL, 1, ADDRESS_SIZE + ADDRESS_SIZE)),
    ]),
]);

const CMP2_ARGS: ArgTable = ArgTable::Two([
    // Register
    Some([
        // Register
        None, 
        // Address in register
        Some(Operation::new(ByteCodes::COMPARE_REG_ADDR_IN_REG, 2, REGISTER_ID_SIZE + REGISTER_ID_SIZE)),
        // Number
        Some(Operation::new(ByteCodes::COMPARE_REG_CONST, 2, REGISTER_ID_SIZE + 2)),
        // Address literal
        Some(Operation::new(ByteCodes::COMPARE_REG_ADDR_LITERAL, 2, REGISTER_ID_SIZE + ADDRESS_SIZE)),
        // Label
        None,
        // Address at label
        Some(Operation::new(ByteCodes::COMPARE_REG_ADDR_LITERAL, 2, REGISTER_ID_SIZE + ADDRESS_SIZE)),
    ]),
    // Address in register
    Some([
        // Register
        Some(Operation::new(ByteCodes::COMPARE_ADDR_IN_REG_REG, 2, REGISTER_ID_SIZE + REGISTER_ID_SIZE)), 
        // Address in register
        Some(Operation::new(ByteCodes::COMPARE_ADDR_IN_REG_ADDR_IN_REG, 2, REGISTER_ID_SIZE + REGISTER_ID_SIZE)), 
        // Number
        Some(Operation::new(ByteCodes::COMPARE_ADDR_IN_REG_CONST, 2, REGISTER_ID_SIZE + 2)), 
        // Address literal
        Some(Operation::new(ByteCodes::COMPARE_ADDR_IN_REG_ADDR_LITERAL, 2, REGISTER_ID_SIZE + ADDRESS_SIZE)),
        // Label
        None,
        // Address at label
        Some(Operation::new(ByteCodes::COMPARE_ADDR_IN_REG_ADDR_LITERAL, 2, REGISTER_ID_SIZE + ADDRESS_SIZE)),
    ]),
    // Number
    Some([
        // Register
        Some(Operation::new(ByteCodes::COMPARE_CONST_REG, 2, 2 + REGISTER_ID_SIZE)), 
        // Address in register
        Some(Operation::new(ByteCodes::COMPARE_CONST_ADDR_IN_REG, 2, 2 + REGISTER_ID_SIZE)),
        // Number
        Some(Operation::new(ByteCodes::COMPARE_CONST_CONST, 2, 2 + 2)), 
        // Address literal
        Some(Operation::new(ByteCodes::COMPARE_CONST_ADDR_LITERAL, 2, 2 + ADDRESS_SIZE)),
        // Label
        None,
        // Address at label
        Some(Operation::new(ByteCodes::COMPARE_CONST_ADDR_LITERAL, 2, 2 + ADDRESS_SIZE)),
    ]),
    // Address literal
    Some([
        // Register
        Some(Operation::new(ByteCodes::COMPARE_ADDR_LITERAL_REG, 2, ADDRESS_SIZE + REGISTER_ID_SIZE)), 
        // Address in register
        Some(Operation::new(ByteCodes::COMPARE_ADDR_LITERAL_ADDR_IN_REG, 2, ADDRESS_SIZE + REGISTER_ID_SIZE)),
        // Number
        Some(Operation::new(ByteCodes::COMPARE_ADDR_LITERAL_CONST, 2, ADDRESS_SIZE + 2)), 
        // Address literal
        Some(Operation::new(ByteCodes::COMPARE_ADDR_LITERAL_ADDR_LITERAL, 2, ADDRESS_SIZE + ADDRESS_SIZE)),
        // Label
        None,
        // Address at label
        Some(Operation::new(ByteCodes::COMPARE_ADDR_LITERAL_ADDR_LITERAL, 2, ADDRESS_SIZE + ADDRESS_SIZE)),
    ]),
    // Label
    None,
    // Address at label
    Some([
        // Register
        Some(Operation::new(ByteCodes::COMPARE_ADDR_LITERAL_REG, 2, ADDRESS_SIZE + REGISTER_ID_SIZE)), 
        // Address in register
        Some(Operation::new(ByteCodes::COMPARE_ADDR_LITERAL_ADDR_IN_REG, 2, ADDRESS_SIZE + REGISTER_ID_SIZE)),
        // Number
        Some(Operation::new(ByteCodes::COMPARE_ADDR_LITERAL_CONST, 2, ADDRESS_SIZE + 2)), 
        // Address literal
        Some(Operation::new(ByteCodes::COMPARE_ADDR_LITERAL_ADDR_LITERAL, 2, ADDRESS_SIZE + ADDRESS_SIZE)),
        // Label
        None,
        // Address at label
        Some(Operation::new(ByteCodes::COMPARE_ADDR_LITERAL_ADDR_LITERAL, 2, ADDRESS_SIZE + ADDRESS_SIZE)),
    ]),
]);

const CMP4_ARGS: ArgTable = ArgTable::Two([
    // Register
    Some([
        // Register
        None,
        // Address in register
        Some(Operation::new(ByteCodes::COMPARE_REG_ADDR_IN_REG, 4, REGISTER_ID_SIZE + REGISTER_ID_SIZE)),
        // Number
        Some(Operation::new(ByteCodes::COMPARE_REG_CONST, 4, REGISTER_ID_SIZE + 4)), 
        // Address literal
        Some(Operation::new(ByteCodes::COMPARE_REG_ADDR_LITERAL, 4, REGISTER_ID_SIZE + ADDRESS_SIZE)),
        // Label
        None,
        // Address at label
        Some(Operation::new(ByteCodes::COMPARE_REG_ADDR_LITERAL, 4, REGISTER_ID_SIZE + ADDRESS_SIZE)),
    ]),
    // Address in register
    Some([
        // Register
        Some(Operation::new(ByteCodes::COMPARE_ADDR_IN_REG_REG, 4, REGISTER_ID_SIZE + REGISTER_ID_SIZE)), 
        // Address in register
        Some(Operation::new(ByteCodes::COMPARE_ADDR_IN_REG_ADDR_IN_REG, 4, REGISTER_ID_SIZE + REGISTER_ID_SIZE)), 
        // Number
        Some(Operation::new(ByteCodes::COMPARE_ADDR_IN_REG_CONST, 4, REGISTER_ID_SIZE + 4)), 
        // Address literal
        Some(Operation::new(ByteCodes::COMPARE_ADDR_IN_REG_ADDR_LITERAL, 4, REGISTER_ID_SIZE + ADDRESS_SIZE)),
        // Label
        None,
        // Address at label
        Some(Operation::new(ByteCodes::COMPARE_ADDR_IN_REG_ADDR_LITERAL, 4, REGISTER_ID_SIZE + ADDRESS_SIZE)),
    ]),
    // Number
    Some([
        // Register
        Some(Operation::new(ByteCodes::COMPARE_CONST_REG, 4, 4 + REGISTER_ID_SIZE)), 
        // Address in register
        Some(Operation::new(ByteCodes::COMPARE_CONST_ADDR_IN_REG, 4, 4 + REGISTER_ID_SIZE)),
        // Number
        Some(Operation::new(ByteCodes::COMPARE_CONST_CONST, 4, 4 + 4)), 
        // Address literal
        Some(Operation::new(ByteCodes::COMPARE_CONST_ADDR_LITERAL, 4, 4 + ADDRESS_SIZE)),
        // Label
        None,
        // Address at label
        Some(Operation::new(ByteCodes::COMPARE_CONST_ADDR_LITERAL, 4, 4 + ADDRESS_SIZE)),
    ]),
    // Address literal
    Some([
        // Register
        Some(Operation::new(ByteCodes::COMPARE_ADDR_LITERAL_REG, 4, ADDRESS_SIZE + REGISTER_ID_SIZE)), 
        // Address in register
        Some(Operation::new(ByteCodes::COMPARE_ADDR_LITERAL_ADDR_IN_REG, 4, ADDRESS_SIZE + REGISTER_ID_SIZE)),
        // Number
        Some(Operation::new(ByteCodes::COMPARE_ADDR_LITERAL_CONST, 4, ADDRESS_SIZE + 4)), 
        // Address literal
        Some(Operation::new(ByteCodes::COMPARE_ADDR_LITERAL_ADDR_LITERAL, 4, ADDRESS_SIZE + ADDRESS_SIZE)),
        // Label
        None,
        // Address at label
        Some(Operation::new(ByteCodes::COMPARE_ADDR_LITERAL_ADDR_LITERAL, 4, ADDRESS_SIZE + ADDRESS_SIZE)),
    ]),
    // Label
    None,
    // Address at label
    Some([
        // Register
        Some(Operation::new(ByteCodes::COMPARE_ADDR_LITERAL_REG, 4, ADDRESS_SIZE + REGISTER_ID_SIZE)), 
        // Address in register
        Some(Operation::new(ByteCodes::COMPARE_ADDR_LITERAL_ADDR_IN_REG, 4, ADDRESS_SIZE + REGISTER_ID_SIZE)),
        // Number
        Some(Operation::new(ByteCodes::COMPARE_ADDR_LITERAL_CONST, 4, ADDRESS_SIZE + 4)), 
        // Address literal
        Some(Operation::new(ByteCodes::COMPARE_ADDR_LITERAL_ADDR_LITERAL, 4, ADDRESS_SIZE + ADDRESS_SIZE)),
        // Label
        None,
        // Address at label
        Some(Operation::new(ByteCodes::COMPARE_ADDR_LITERAL_ADDR_LITERAL, 4, ADDRESS_SIZE + ADDRESS_SIZE)),
    ]),
]);

const CMP8_ARGS: ArgTable = ArgTable::Two([
    // Register
    Some([
        // Register
        None, 
        // Address in register
        Some(Operation::new(ByteCodes::COMPARE_REG_ADDR_IN_REG, 8, REGISTER_ID_SIZE + REGISTER_ID_SIZE)),
        // Number
        Some(Operation::new(ByteCodes::COMPARE_REG_CONST, 8, REGISTER_ID_SIZE + 8)), 
        // Address literal
        Some(Operation::new(ByteCodes::COMPARE_REG_ADDR_LITERAL, 8, REGISTER_ID_SIZE + ADDRESS_SIZE)),
        // Label
        Some(Operation::new(ByteCodes::COMPARE_REG_CONST, 8, REGISTER_ID_SIZE + ADDRESS_SIZE)),
        // Address at label
        Some(Operation::new(ByteCodes::COMPARE_REG_ADDR_LITERAL, 8, REGISTER_ID_SIZE + ADDRESS_SIZE)),
    ]),
    // Address in register
    Some([
        // Register
        Some(Operation::new(ByteCodes::COMPARE_ADDR_IN_REG_REG, 8, REGISTER_ID_SIZE + REGISTER_ID_SIZE)), 
        // Address in register
        Some(Operation::new(ByteCodes::COMPARE_ADDR_IN_REG_ADDR_IN_REG, 8, REGISTER_ID_SIZE + REGISTER_ID_SIZE)), 
        // Number
        Some(Operation::new(ByteCodes::COMPARE_ADDR_IN_REG_CONST, 8, REGISTER_ID_SIZE + 8)), 
        // Address literal
        Some(Operation::new(ByteCodes::COMPARE_ADDR_IN_REG_ADDR_LITERAL, 8, REGISTER_ID_SIZE + ADDRESS_SIZE)),
        // Label
        Some(Operation::new(ByteCodes::COMPARE_ADDR_IN_REG_CONST, 8, REGISTER_ID_SIZE + ADDRESS_SIZE)),
        // Address at label
        Some(Operation::new(ByteCodes::COMPARE_ADDR_IN_REG_ADDR_LITERAL, 8, REGISTER_ID_SIZE + ADDRESS_SIZE)),
    ]),
    // Number
    Some([
        // Register
        Some(Operation::new(ByteCodes::COMPARE_CONST_REG, 8, 8 + REGISTER_ID_SIZE)),
        // Address in register
        Some(Operation::new(ByteCodes::COMPARE_CONST_ADDR_IN_REG, 8, 8 + REGISTER_ID_SIZE)),
        // Number
        Some(Operation::new(ByteCodes::COMPARE_CONST_CONST, 8, 8 + 8)), 
        // Address literal
        Some(Operation::new(ByteCodes::COMPARE_CONST_ADDR_LITERAL, 8, 8 + ADDRESS_SIZE)),
        // Label
        Some(Operation::new(ByteCodes::COMPARE_CONST_CONST, 8, 8 + ADDRESS_SIZE)),
        // Address at label
        Some(Operation::new(ByteCodes::COMPARE_CONST_ADDR_LITERAL, 8, 8 + ADDRESS_SIZE)),
    ]),
    // Address literal
    Some([
        // Register
        Some(Operation::new(ByteCodes::COMPARE_ADDR_LITERAL_REG, 8, ADDRESS_SIZE + REGISTER_ID_SIZE)),
        // Address in register
        Some(Operation::new(ByteCodes::COMPARE_ADDR_LITERAL_ADDR_IN_REG, 8, ADDRESS_SIZE + REGISTER_ID_SIZE)),
        // Number
        Some(Operation::new(ByteCodes::COMPARE_ADDR_LITERAL_CONST, 8, ADDRESS_SIZE + 8)), 
        // Address literal
        Some(Operation::new(ByteCodes::COMPARE_ADDR_LITERAL_ADDR_LITERAL, 8, ADDRESS_SIZE + ADDRESS_SIZE)),
        // Label
        Some(Operation::new(ByteCodes::COMPARE_ADDR_LITERAL_CONST, 8, ADDRESS_SIZE + ADDRESS_SIZE)),
        // Address at label
        Some(Operation::new(ByteCodes::COMPARE_ADDR_LITERAL_ADDR_LITERAL, 8, ADDRESS_SIZE + ADDRESS_SIZE)),
    ]),
    // Label
    Some([
        // Register
        Some(Operation::new(ByteCodes::COMPARE_CONST_REG, 8, ADDRESS_SIZE + REGISTER_ID_SIZE)),
        // Address in register
        Some(Operation::new(ByteCodes::COMPARE_CONST_ADDR_IN_REG, 8, ADDRESS_SIZE + REGISTER_ID_SIZE)),
        // Number
        Some(Operation::new(ByteCodes::COMPARE_CONST_CONST, 8, ADDRESS_SIZE + 8)), 
        // Address literal
        Some(Operation::new(ByteCodes::COMPARE_CONST_ADDR_LITERAL, 8, ADDRESS_SIZE + ADDRESS_SIZE)),
        // Label
        Some(Operation::new(ByteCodes::COMPARE_CONST_CONST, 8, ADDRESS_SIZE + ADDRESS_SIZE)),
        // Address at label
        Some(Operation::new(ByteCodes::COMPARE_CONST_ADDR_LITERAL, 8, ADDRESS_SIZE + ADDRESS_SIZE)),
    ]),
    // Address at label
    Some([
        // Register
        Some(Operation::new(ByteCodes::COMPARE_ADDR_LITERAL_REG, 8, ADDRESS_SIZE + REGISTER_ID_SIZE)),
        // Address in register
        Some(Operation::new(ByteCodes::COMPARE_ADDR_LITERAL_ADDR_IN_REG, 8, ADDRESS_SIZE + REGISTER_ID_SIZE)),
        // Number
        Some(Operation::new(ByteCodes::COMPARE_ADDR_LITERAL_CONST, 8, ADDRESS_SIZE + 8)), 
        // Address literal
        Some(Operation::new(ByteCodes::COMPARE_ADDR_LITERAL_ADDR_LITERAL, 8, ADDRESS_SIZE + ADDRESS_SIZE)),
        // Label
        Some(Operation::new(ByteCodes::COMPARE_ADDR_LITERAL_CONST, 8, ADDRESS_SIZE + ADDRESS_SIZE)),
        // Address at label
        Some(Operation::new(ByteCodes::COMPARE_ADDR_LITERAL_ADDR_LITERAL, 8, ADDRESS_SIZE + ADDRESS_SIZE)),
    ]),
]);

const PRINTI_ARGS: ArgTable = ArgTable::Zero(Operation::new(ByteCodes::PRINT_SIGNED, 0, 0));

const PRINTU_ARGS: ArgTable = ArgTable::Zero(Operation::new(ByteCodes::PRINT_UNSIGNED, 0, 0));

const PRINTC_ARGS: ArgTable = ArgTable::Zero(Operation::new(ByteCodes::PRINT_CHAR, 0, 0));

const PRINTSTR_ARGS: ArgTable = ArgTable::Zero(Operation::new(ByteCodes::PRINT_STRING, 0, 0));

const PRINTBYTES_ARGS: ArgTable = ArgTable::Zero(Operation::new(ByteCodes::PRINT_BYTES, 0, 0));

const INPUTINT_ARGS: ArgTable = ArgTable::Zero(Operation::new(ByteCodes::INPUT_INT, 0, 0));

const INPUTSTR_ARGS: ArgTable = ArgTable::Zero(Operation::new(ByteCodes::INPUT_STRING, 0, 0));

const EXIT_ARGS: ArgTable = ArgTable::Zero(Operation::new(ByteCodes::EXIT, 0, 0));


/// Return the arguments table for the given operator
pub fn get_arguments_table(operator_name: &str) -> Option<&'static ArgTable> {

    Some(match operator_name {

        // Arithmetic

        "add" => &ADD_ARGS,

        "sub" => &SUB_ARGS,

        "mul" => &MUL_ARGS,

        "div" => &DIV_ARGS,

        "mod" => &MOD_ARGS,

        "inc" => &INC_ARGS, 
    
        "inc1" => &INC1_ARGS,
        
        "inc2" => &INC2_ARGS,

        "inc4" => &INC4_ARGS,
    
        "inc8" => &INC8_ARGS,

        "dec" => &DEC_ARGS,
        
        "dec1" => &DEC1_ARGS,
        
        "dec2" => &DEC2_ARGS,
        
        "dec4" => &DEC4_ARGS,
        
        "dec8" => &DEC8_ARGS,
    
        // No operation

        "nop" => &NOP_ARGS,

        // Memory

        "mov" => &MOV_ARGS,

        "mov1" => &MOV1_ARGS,

        "mov2" => &MOV2_ARGS,
        
        "mov4" => &MOV4_ARGS,

        "mov8" => &MOV8_ARGS,
        
        "push" => &PUSH_ARGS,

        "push1" => &PUSH1_ARGS,

        "push2" => &PUSH2_ARGS,

        "push4" => &PUSH4_ARGS,

        "push8" => &PUSH8_ARGS,
        
        "pop1" => &POP1_ARGS,
        
        "pop2" => &POP2_ARGS,
        
        "pop4" => &POP4_ARGS,
        
        "pop8" => &POP8_ARGS,
        
        // Control flow

        "jmp" => &JMP_ARGS,

        "jmpnz" => &JMPNZ_ARGS,

        "jmpz" => &JMPZ_ARGS,

        "jmpgr" => &JMPGR_ARGS,

        "jmpge" => &JMPGE_ARGS,

        "jmplt" => &JMPLT_ARGS,

        "jmple" => &JMPLE_ARGS,

        "jmpof" => &JMPOF,

        "jmpnof" => &JMPNOF,

        "jmpcr" => &JMPCR_ARGS,

        "jmpncr" => &JMPNCR_ARGS,

        "jmpsn" => &JMPSN_ARGS,

        "jmpnsn" => &JMPNSN_ARGS,

        "call" => &CALL_ARGS,

        "ret" => &RET_ARGS,

        // Comparison

        "cmp" => &CMP_ARGS,

        "cmp1" => &CMP1_ARGS,

        "cmp2" => &CMP2_ARGS,

        "cmp4" => &CMP4_ARGS,

        "cmp8" => &CMP8_ARGS,

        // Interrupts

        "printi" => &PRINTI_ARGS,

        "printu" => &PRINTU_ARGS,

        "printc" => &PRINTC_ARGS,

        "printstr" => &PRINTSTR_ARGS,

        "printbytes" => &PRINTBYTES_ARGS,

        "inputint" => &INPUTINT_ARGS,

        "inputstr" => &INPUTSTR_ARGS,

        "exit" => &EXIT_ARGS,

        _ => return None

    })

}

