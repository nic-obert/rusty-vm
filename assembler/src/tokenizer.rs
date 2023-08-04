use rust_vm_lib::registers::get_register;
use rust_vm_lib::token::{Token, TokenValue};

use crate::error;
use crate::assembler::LabelMap;


pub fn is_name_character(c: char) -> bool {
    c.is_alphabetic() || c == '_'
}


pub fn is_label_name(name: &str) -> bool {
    for c in name.chars() {
        if !is_name_character(c) {
            return false;
        }
    }
    true
}


/// Returns the evaluated escape character
fn match_escape_char(c: char, line_number: usize, char_index: usize, line: &str, unit_path: &str) -> char {
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
pub fn evaluate_string(string: &str, delimiter: char, line_number: usize, line: &str, unit_path: &str) -> String {
    
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



pub fn tokenize_operands(mut operands: String, line_number: usize, line: &str, label_map: &LabelMap, unit_path: &str) -> Vec<Token> {

    let mut tokens: Vec<Token> = Vec::new();

    // Add a semicolon at the end in order to make the loop iterate one more time for simplicity
    operands.push('#');

    let mut current_token: Option<Token> = None;

    let mut escape_char = false;
    let mut string_length: usize = 0;

    for (char_index, c) in operands.chars().enumerate() {
        
        if let Some(token) = &mut current_token {

            match &mut token.value {

                TokenValue::Char(value) => {

                    if string_length > 1 {
                        error::invalid_character(unit_path, c, line_number, char_index, line, "Expected a closing single quote: Character literals can only be one character long.");
                    }

                    // Handle escape characters
                   
                    string_length += 1;

                    if escape_char {
                        *value = match_escape_char(c, line_number, char_index, line, unit_path);
                        escape_char = false;
                    } else if c == '\'' {
                        // The character literal is complete
                        if string_length == 0 {
                            error::invalid_character(unit_path, c, line_number, char_index, line, "Character literals cannot be empty.");
                        }
                        string_length = 0;
                        // Convert the character to a byte
                        tokens.push(Token::new(TokenValue::Number(*value as i64)));
                        current_token = None;
                    } else {
                        *value = c;
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
                            Token::new(TokenValue::AddressInRegisterIncomplete(c.to_string()))
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

                TokenValue::AddressInRegisterIncomplete(value) => {
                    if is_name_character(c) {
                        value.push(c);
                        continue;
                    }

                    if c != ']' {
                        error::invalid_character(unit_path, c, line_number, char_index, line, format!("Expected a closing address ']', got '{}' instead.", c).as_str());
                    }

                    if let Some(register) = get_register(value) {
                        tokens.push(Token::new(TokenValue::AddressInRegister(register)));
                        current_token = None;
                    } else {
                        error::invalid_register_name(unit_path, value, line_number, line);
                    }

                    continue;
                },

                TokenValue::Name(value) => {
                    if is_name_character(c) {
                        value.push(c);
                        continue;
                    }

                    // Check if the name is a special name (labels, registers, reserved names...)

                    if let Some(register) = get_register(value) {
                        tokens.push(Token::new(TokenValue::Register(register)));
                        current_token = None;
                    }
                    else if let Some(label) = label_map.get(value) {
                        // Don't mind the primitive type conversion. The underlying bytes are the same.
                        tokens.push(Token::new(TokenValue::Number(*label as i64)));
                    } else {
                        tokens.push(current_token.take().unwrap());
                    }
                }
                   
                TokenValue::Number(value) => {
                    if c.is_digit(10) {
                        *value = value.checked_mul(10).unwrap_or_else(
                            || error::number_out_of_range(unit_path, *value, line_number, line)
                        ).checked_add(c.to_digit(10).unwrap() as i64).unwrap_or_else(
                            || error::number_out_of_range(unit_path, *value, line_number, line)
                        );
                        continue;
                    }

                    tokens.push(current_token.take().unwrap());
                },
                
                _ => { }
                
            }
        }


        if is_name_character(c) {
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

