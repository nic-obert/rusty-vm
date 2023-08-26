use std::mem;
use std::path::Path;

use rust_vm_lib::registers::{Registers, REGISTER_SIZE};
use rust_vm_lib::token::{Token, TokenValue, NumberFormat, NumberSign};
use rust_vm_lib::vm::Address;

use crate::error;
use crate::argmuments_table;


/// Returns whether the name is a reserved name by the assembler
/// Reserved names are register names, and instruction names, error names
pub fn is_reserved_name(name: &str) -> bool {
    Registers::from_name(name).is_some() 
    || argmuments_table::get_arguments_table(name).is_some()
    || name == "endmacro"
    
}


/// Return whether the given character is a valid identifier character.
/// 
/// Identifiers can only contain letters, numbers, and underscores.
/// The first character cannot be a number.
pub fn is_identifier_char(c: char, is_first_char: bool) -> bool {
    c == '_'
    || c.is_alphabetic()
    || (!is_first_char && c.is_numeric())
}


/// Returns whether the given string is a valid label name.
/// 
/// Label names can only contain letters, numbers, and underscores.
/// The first character cannot be a number.
pub fn is_identifier_name(name: &str) -> bool {

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
                            tokens.push(Token::new(TokenValue::Number { value: *value as i64, sign: NumberSign::Positive, format: NumberFormat::Unknown }));
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

                TokenValue::AddressGeneric() => {

                    if c == '0' {
                        current_token = Some(Token::new(TokenValue::AddressLiteral { value: 0, format: NumberFormat::Unknown }));

                    } else if let Some(digit) = c.to_digit(10) {
                        current_token = Some(Token::new(TokenValue::AddressLiteral { value: digit as usize, format: NumberFormat::Decimal }));

                    } else if is_identifier_char(c, true) {
                        current_token = Some(Token::new(TokenValue::AddressAtIdentifier(c.to_string())));

                    } else if c == ']' {
                        error::invalid_character(unit_path, c, line_number, char_index, line, "Addresses cannot be empty.");
                    
                    } else {
                        error::invalid_character(unit_path, c, line_number, char_index, line, "Addresses can only be numeric or names.");
                    }

                    continue;
                },
                
                TokenValue::AddressLiteral { value, format } => {

                    match format {

                        NumberFormat::Decimal => {
                            if let Some(digit) = c.to_digit(10) {
                                *value = value.checked_mul(10).unwrap_or_else(
                                    || error::number_out_of_range::<Address>(unit_path, format!("{}{}", value, digit).as_str(), 10, REGISTER_SIZE as u8, line_number, line)
                                ).checked_add(digit as usize).unwrap_or_else(
                                    || error::number_out_of_range::<Address>(unit_path, format!("{}{}", value, digit).as_str(), 10, REGISTER_SIZE as u8, line_number, line)
                                );
        
                                continue;
                            }
                        },

                        NumberFormat::Hexadecimal => {
                            if let Some(digit) = c.to_digit(16) {
                                *value = value.checked_mul(16).unwrap_or_else(
                                    || error::number_out_of_range::<Address>(unit_path, format!("{:#x}{}", value, digit).as_str(), 16, REGISTER_SIZE as u8, line_number, line)
                                ).checked_add(digit as usize).unwrap_or_else(
                                    || error::number_out_of_range::<Address>(unit_path, format!("{:#x}{}", value, digit).as_str(), 16, REGISTER_SIZE as u8, line_number, line)
                                );
        
                                continue;
                            }
                        },

                        NumberFormat::Binary => {
                            if let Some(digit) = c.to_digit(2) {
                                *value = value.checked_mul(2).unwrap_or_else(
                                    || error::number_out_of_range::<Address>(unit_path, format!("{:#b}{}", value, digit).as_str(), 2, REGISTER_SIZE as u8, line_number, line)
                                ).checked_add(digit as usize).unwrap_or_else(
                                    || error::number_out_of_range::<Address>(unit_path, format!("{:#b}{}", value, digit).as_str(), 2, REGISTER_SIZE as u8, line_number, line)
                                );
        
                                continue;
                            }
                        },

                        NumberFormat::Unknown => {
                            match c {
                                'x' => *format = NumberFormat::Hexadecimal,
                                'b' => *format = NumberFormat::Binary,
                                'd' => *format = NumberFormat::Decimal,
                                _ => error::invalid_character(unit_path, c, line_number, char_index, line, "Invalid number format.")
                            }

                            continue;
                        },

                        NumberFormat::Float { .. } => unreachable!("Cannot have a float address literal. This is a bug."),

                    }  

                    if c == ']' {
                        tokens.push(current_token.take().unwrap());                    
                    } else {
                        error::invalid_character(unit_path, c, line_number, char_index, line, "Address literals can only be numeric.");
                    }

                    continue;
                },

                TokenValue::AddressAtIdentifier(identifier) => {
                    if is_identifier_char(c, false) {
                        identifier.push(c);
                        continue;
                    }

                    if c != ']' {
                        error::invalid_character(unit_path, c, line_number, char_index, line, format!("Expected a closing address ']', got '{}' instead.", c).as_str());
                    }

                    // Determine what kind of identifier it is
                    if let Some(register) = Registers::from_name(identifier) {
                        tokens.push(Token::new(TokenValue::AddressInRegister(register)));

                    } else if is_reserved_name(identifier) {
                        // Check if the name is a reserved keyword
                        error::invalid_address_identifier(unit_path, identifier, line_number, line);

                    } else {
                        // The name is not reserved, then it's a label
                        tokens.push(Token::new(TokenValue::AddressAtLabel(mem::take(identifier))));
                    }

                    current_token = None;

                    continue;
                },

                TokenValue::Name(name) => {
                    if is_identifier_char(c, false) {
                        name.push(c);
                        continue;
                    }

                    // Check if the name is a special reserved name 

                    if let Some(register) = Registers::from_name(name) {
                        tokens.push(Token::new(TokenValue::Register(register)));

                    } else {
                        // The name is not special, then it's a label
                        tokens.push(Token::new(TokenValue::Label(mem::take(name))));
                    }
                    
                    current_token = None;
                }
                   
                TokenValue::Number { value, sign, format } => {

                    match format {

                        NumberFormat::Decimal => {
                            if let Some(digit) = c.to_digit(10) {

                                if matches!(sign, NumberSign::Negative) {

                                    *value = value.checked_mul(10).unwrap_or_else(
                                        || error::number_out_of_range::<i64>(unit_path, format!("{}{}", value, digit).as_str(), 10, REGISTER_SIZE as u8, line_number, line)
                                    ).checked_add(digit as i64).unwrap_or_else(
                                        || error::number_out_of_range::<i64>(unit_path, format!("{}{}", value, digit).as_str(), 10, REGISTER_SIZE as u8, line_number, line)
                                    );

                                } else {

                                    *value = (*value as u64).checked_mul(10).unwrap_or_else(
                                        || error::number_out_of_range::<u64>(unit_path, format!("{}{}", *value as u64, digit).as_str(), 10, REGISTER_SIZE as u8, line_number, line)
                                    ).checked_add(digit as u64).unwrap_or_else(
                                        || error::number_out_of_range::<u64>(unit_path, format!("{}{}", *value as u64, digit).as_str(), 10, REGISTER_SIZE as u8, line_number, line)
                                    ) as i64;

                                }

                                continue;

                            } else if c == '.' {
                                *format = NumberFormat::Float { decimal: String::new() };
                                continue;
                            }
        
                            // After the decimal number is constructed, evaluate its sign
                            if matches!(sign, NumberSign::Negative) {
                                *value = -*value;
                            }
                        },

                        NumberFormat::Hexadecimal => {
                            if let Some(digit) = c.to_digit(16) {
                                *value = value.checked_mul(16).unwrap_or_else(
                                    || error::number_out_of_range::<i64>(unit_path, format!("{:#x}{}", value, digit).as_str(), 16, REGISTER_SIZE as u8, line_number, line)
                                ).checked_add(digit as i64).unwrap_or_else(
                                    || error::number_out_of_range::<i64>(unit_path, format!("{:#x}{}", value, digit).as_str(), 16, REGISTER_SIZE as u8, line_number, line)
                                );
        
                                continue;
                            }
                        },

                        NumberFormat::Binary => {
                            if let Some(digit) = c.to_digit(2) {
                                *value = value.checked_mul(2).unwrap_or_else(
                                    || error::number_out_of_range::<i64>(unit_path, format!("{:#b}{}", value, digit).as_str(), 2, REGISTER_SIZE as u8, line_number, line)
                                ).checked_add(digit as i64).unwrap_or_else(
                                    || error::number_out_of_range::<i64>(unit_path, format!("{:#b}{}", value, digit).as_str(), 2, REGISTER_SIZE as u8, line_number, line)
                                );
        
                                continue;
                            }
                        },

                        NumberFormat::Unknown => {
                            match c {
                                'x' => *format = NumberFormat::Hexadecimal,
                                'b' => *format = NumberFormat::Binary,
                                'd' => *format = NumberFormat::Decimal,

                                _ => {

                                    if let Some(digit) = c.to_digit(10) {
                                        *format = NumberFormat::Decimal;

                                        *value = value.checked_mul(10).unwrap_or_else(
                                            || error::number_out_of_range::<i64>(unit_path, format!("{}{}", value, digit).as_str(), 10, REGISTER_SIZE as u8, line_number, line)
                                        ).checked_add(digit as i64).unwrap_or_else(
                                            || error::number_out_of_range::<i64>(unit_path, format!("{}{}", value, digit).as_str(), 10, REGISTER_SIZE as u8, line_number, line)
                                        );

                                    } else {
                                        tokens.push(Token::new(TokenValue::Number { value: 0, sign: NumberSign::Positive, format: NumberFormat::Decimal }));
                                        current_token = None;
                                    }
                                }
                            }

                            continue;
                        },
                        
                        NumberFormat::Float { decimal } => {
                            if c.is_ascii_digit() {
                                decimal.push(c);
                                continue;
                            }

                            if c == '.' {
                                error::invalid_character(unit_path, c, line_number, char_index, line, "Floats can only have one decimal point.");
                            }

                            let decimal = decimal.parse::<f64>().unwrap_or_else(
                                |e| error::invalid_float_number(unit_path, format!("{}.{}", value, decimal).as_str(), line_number, line, e.to_string().as_str())
                            );

                            // After the decimal number is constructed, evaluate its sign
                            if matches!(sign, NumberSign::Negative) {
                                *value = (*value as f64 - decimal) as i64;
                            } else {
                                *value = (*value as f64 + decimal) as i64;
                            }

                        },

                    }                    

                    tokens.push(current_token.take().unwrap());
                },
                
                _ => unreachable!("Unhandled token value type: {:?}. This is a bug.", token.value)
                
            }
        }


        if is_identifier_char(c, true) {
            current_token = Some(Token::new(TokenValue::Name(c.to_string())));
            continue;
        }

        if c == '0' {
            current_token = Some(Token::new(TokenValue::Number { value: 0, sign: NumberSign::Positive, format: NumberFormat::Unknown }));
            continue;
        }

        if let Some(digit) = c.to_digit(10) {
            current_token = Some(Token::new(TokenValue::Number { value: digit as i64, sign: NumberSign::Positive, format: NumberFormat::Decimal }));
            continue;
        }

        match c {
            ' ' | '\t' => continue,

            '.' => {
                current_token = Some(Token::new(TokenValue::Number { value: 0, sign: NumberSign::Positive, format: NumberFormat::Float { decimal: String::new() } }));
                continue;
            }

            '+' => {
                current_token = Some(Token::new(TokenValue::Number { value: 0, sign: NumberSign::Positive, format: NumberFormat::Decimal }));
                continue;
            }

            '-' => {
                current_token = Some(Token::new(TokenValue::Number { value: 0, sign: NumberSign::Negative, format: NumberFormat::Decimal }));
                continue;
            }

            '\'' => {
                // Use a null character to represent an empty char literal that will be filled later
                current_token = Some(Token::new(TokenValue::Char('\0')));
                continue;
            }

            '#' => break,

            '[' => {
                current_token = Some(Token::new(TokenValue::AddressGeneric()));
                continue;
            },

            '=' => error::invalid_character(unit_path, c, line_number, char_index, line, "The given character wans't expected in this context. Maybe you forgot to declare a const macro?"),

            _ => error::invalid_character(unit_path, c, line_number, char_index, line, "The given character wans't expected in this context.")
        }

    }   

    tokens
}

