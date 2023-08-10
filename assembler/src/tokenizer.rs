use std::mem;
use std::path::Path;

use rust_vm_lib::registers;
use rust_vm_lib::token::{Token, TokenValue};

use crate::error;
use crate::assembler::is_reserved_name;


/// Return whether the given character is a valid identifier character.
/// 
/// Identifiers can only contain letters, numbers, and underscores.
/// The first character cannot be a number.
pub fn is_identifier_char(c: char, is_first_char: bool) -> bool {
    if is_first_char {
        c.is_alphabetic() || c == '_'
    } else {
        c.is_alphanumeric() || c == '_'
    }
}


/// Returns whether the given string is a valid label name.
/// 
/// Label names can only contain letters, numbers, and underscores.
/// The first character cannot be a number.
pub fn is_label_name(name: &str) -> bool {

    if name.is_empty() {
        return false;
    }

    let mut it = name.chars();

    if !is_identifier_char(it.next().unwrap(), true) {
        return false;
    }

    for c in it {
        if !is_identifier_char(c, false) {
            return false;
        }
    }

    true
}


/// Returns the evaluated escape character
fn match_escape_char(c: char, line_number: usize, char_index: usize, line: &str, unit_path: &Path) -> char {
    match c {
        '\\' => '\\',
        'n' => '\n',
        't' => '\t',
        'r' => '\r',
        '0' => '\0',
        '\'' => '\'',
        _ => error::invalid_character(unit_path, c, line_number, char_index, line, "Invalid escape character.")
    }
}


/// Evaluates escape characters in a string literal
pub fn evaluate_string(string: &str, delimiter: char, line_number: usize, line: &str, unit_path: &Path) -> String {
    
    let mut evaluated_string = String::with_capacity(string.len());

    let mut escape_char = false;

    for (char_index, c) in string.chars().enumerate() {

        if escape_char {
            evaluated_string.push(
                match_escape_char(c, line_number, char_index, line, unit_path)
            );
            escape_char = false;
            continue;
        }

        if c == '\\' {
            escape_char = true;
            continue;
        }

        if c == delimiter {
            break;
        }

        evaluated_string.push(c);
    }

    evaluated_string
}


/// Tokenizes the operands of an instruction and returns a vector of tokens.
/// 
/// The tokenizer handles eventual labels and converts them to their address.
pub fn tokenize_operands(operands: &str, line_number: usize, line: &str, unit_path: &Path) -> Vec<Token> {

    let mut tokens: Vec<Token> = Vec::new();

    let mut current_token: Option<Token> = None;

    let mut escape_char = false;
    let mut string_length: usize = 0;

    let mut chars_iter = operands.chars();

    // Iterate one additional time to handle the end of the string
    for char_index in 0..=operands.len() {

        // Get the next character
        // chars_iter.next() will fail only at the end of the string
        let c = chars_iter.next().unwrap_or('#');
        
        if let Some(token) = &mut current_token {

            match &mut token.value {

                TokenValue::Char(value) => {

                    if string_length > 1 {
                        error::invalid_character(unit_path, c, line_number, char_index, line, "Expected a closing single quote: Character literals can only be one character long.");
                    }

                    if escape_char {
                        *value = match_escape_char(c, line_number, char_index, line, unit_path);
                        escape_char = false;
                        string_length += 1;
                        continue;
                    } 
                    
                    match c {

                        '\'' => {
                            // The character literal is complete
                            if string_length == 0 {
                                error::invalid_character(unit_path, c, line_number, char_index, line, "Character literals cannot be empty.");
                            }
                            string_length = 0;
                            // Convert the character to a byte
                            tokens.push(Token::new(TokenValue::Number(*value as i64)));
                            current_token = None;
                        },

                        '\\' => escape_char = true,

                        _ => {
                            *value = c;
                            string_length += 1;
                        }

                    }

                    continue;
                },

                TokenValue::AddressGeneric(_value) => {
                    if c.is_digit(10) {
                        current_token = Some(
                            Token::new(TokenValue::AddressLiteral(c.to_digit(10).unwrap() as usize))
                        );
                    }
                    else if c.is_alphabetic() {
                        current_token = Some(
                            Token::new(TokenValue::AddressAtIdentifier(c.to_string()))
                        );
                    } else if c == ']' {
                        error::invalid_character(unit_path, c, line_number, char_index, line, "Addresses cannot be empty.");
                    } else {
                        error::invalid_character(unit_path, c, line_number, char_index, line, "Addresses can only be numeric or register names.");
                    }

                    continue;
                },
                
                TokenValue::AddressLiteral(value) => {
                    if c.is_digit(10) {
                        *value = value.checked_mul(10).unwrap_or_else(
                            || error::invalid_address(unit_path, *value, line_number, line, "Address literal is too large.")
                        ).checked_add(c.to_digit(10).unwrap() as usize).unwrap_or_else(
                            || error::invalid_address(unit_path, *value, line_number, line, "Address literal is too large.")
                        );
                    } else if c == ']' {
                        tokens.push(current_token.take().unwrap());                    
                    } else {
                        error::invalid_character(unit_path, c, line_number, char_index, line, "Address literals can only be numeric.");
                    }

                    continue;
                },

                TokenValue::AddressAtIdentifier(value) => {
                    if is_identifier_char(c, false) {
                        value.push(c);
                        continue;
                    }

                    if c != ']' {
                        error::invalid_character(unit_path, c, line_number, char_index, line, format!("Expected a closing address ']', got '{}' instead.", c).as_str());
                    }

                    // Determine what kind of identifier it is
                    if let Some(register) = registers::get_register(value) {
                        tokens.push(Token::new(TokenValue::AddressInRegister(register)));
                        current_token = None;

                    } else if is_reserved_name(value) {
                        // Check if the name is a reserved keyword
                        error::invalid_address_identifier(unit_path, &value, line_number, line);
                    } else {
                        // The name is probably a label
                        tokens.push(Token::new(TokenValue::Label(mem::take(value))));
                    }

                    continue;
                },

                TokenValue::Name(value) => {
                    if is_identifier_char(c, false) {
                        value.push(c);
                        continue;
                    }

                    // Check if the name is a special register name 

                    if let Some(register) = registers::get_register(value) {
                        tokens.push(Token::new(TokenValue::Register(register)));
                    } else {
                        tokens.push(Token::new(TokenValue::Label(mem::take(value))));
                    }

                    current_token = None;
                }
                   
                TokenValue::Number(value) => {
                    if c.is_digit(10) {
                        *value = value.checked_mul(10).unwrap_or_else(
                            || error::number_out_of_range(unit_path, *value, mem::size_of::<i64>() as u8, line_number, line)
                        ).checked_add(c.to_digit(10).unwrap() as i64).unwrap_or_else(
                            || error::number_out_of_range(unit_path, *value, mem::size_of::<i64>() as u8, line_number, line)
                        );
                        continue;
                    }

                    tokens.push(current_token.take().unwrap());
                },
                
                _ => { }
                
            }
        }


        if is_identifier_char(c, true) {
            current_token = Some(Token::new(TokenValue::Name(c.to_string())));
            continue;
        }

        if c.is_digit(10) {
            current_token = Some(Token::new(TokenValue::Number(c.to_digit(10).unwrap() as i64)));
            continue;
        }

        match c {
            ' ' | '\t' => continue,

            '\'' => {
                // Use a null character to represent an empty char literal that will be filled later
                current_token = Some(Token::new(TokenValue::Char('\0')));
                continue;
            }

            '#' => break,

            '[' => {
                current_token = Some(Token::new(TokenValue::AddressGeneric(0)));
                continue;
            },

            _ => error::invalid_character(unit_path, c, line_number, char_index, line, "The given character wans't expected in this context.")
        }

    }   

    tokens
}

