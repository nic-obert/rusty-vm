use shared::registers::get_register;
use shared::token::{Token, TokenValue};


fn is_name_character(c: char) -> bool {
    c.is_alphabetic() || c == '_'
}


pub fn tokenize_operands(operands: &String) -> Vec<Token> {

    let tokens: Vec<Token> = Vec::new();

    // Add a semicolon at the end in order to make the loop iterate one more time for simplicity
    operands.push(';');

    let mut current_token: Option<Token> = None;

    for c in operands.chars() {

        if let Some(token) = current_token {

            let token: &mut Token = current_token.as_mut().unwrap();

            match token.value {
                TokenValue::AddressGeneric(value) => {
                    if c.is_digit(10) {
                        current_token = Some(
                            Token::new(TokenValue::AddressLiteral(c.to_digit(10).unwrap() as u64))
                        );
                    }
                    else if is_name_character(c) {
                        current_token = Some(
                            Token::new(TokenValue::AddressInRegister(c))
                        );
                    }

                    continue;
                },
                
                TokenValue::AddressLiteral(value) => {
                    if c.is_digit(10) {
                        *value = *value * 10 + c.to_digit(10).unwrap() as u64;
                        continue;
                    }

                    tokens.push(current_token.take().unwrap());                    
                },

                TokenValue::AddressInRegisterIncomplete(value) => {
                    if is_name_character(c) {
                        value.push(c);
                        continue;
                    }

                    if c == ' ' {
                        continue;
                    }
                    if char != ']' {
                        panic!(format!("Expected ']' after address in argument list \"{}\", but '{}' was provided", operands, c));
                    }

                    if let Some(register) = get_register(value) {
                        tokens.push(Token::new(TokenValue::AddressInRegister(register)));
                        current_token = None;
                    }
                    else {
                        panic!(format!("Unknown register \"{}\" in argument list \"{}\"", value, operands));
                    }

                    continue;
                },

                TokenValue::Name(value) => {
                    if is_name_character(c) {
                        value.push(c);
                        continue;
                    }
                    if c == ':' {
                        tokens.push(Token::new(TokenValue::Label(value)));
                        current_token = None;
                        continue;
                    }

                    if let Some(register) = get_register(value) {
                        tokens.push(Token::new(TokenValue::Register(register)));
                        current_token = None;
                    }
                    else {
                        tokens.push(current_token.take().unwrap());
                    }
                }
                   
                TokenValue::Number(value) => {
                    if c.is_digit(10) {
                        *value = *value * 10 + c.to_digit(10).unwrap() as u64;
                        continue;
                    }

                    tokens.push(current_token.take().unwrap());
                },
                
                _ => { }
                
            }
        }


        if c == '[' {
            current_token = Some(Token::new(TokenType::AddressGeneric(0)));
            continue;
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
            ' ' |
            '\t' \
            ' ' 
            => continue,

            ';' => break,

            '[' => {
                current_token = Some(Token::new(TokenType::AddressGeneric(0)));
                continue;
            },

            _ => panic!(format!("Unhaldled character '{}' in operands \"{}\"", c, operands))
        }
    }   

    tokens
}


