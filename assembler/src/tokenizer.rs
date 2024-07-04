use std::borrow::Cow;
use std::collections::VecDeque;
use std::rc::Rc;

use lazy_static::lazy_static;

use regex::Regex;
use rusty_vm_lib::registers::Registers;

use crate::error;
use crate::lang::{AsmInstruction, Number, PseudoInstructions};
use crate::module_manager::UnitPath;


lazy_static! {

    static ref TOKEN_REGEX: Regex = Regex::new(
        r#"(?m)#.*$|'(?:\\'|[^'])*'|"(?:\\"|[^"])*"|0x[a-fA-F\d]+|-?\d+[.]\d*|-?[.]?\d+|%%-|%-|@@|%%|[_a-zA-Z]\w*|\S"#
    ).expect("Failed to compile regex");

    static ref IDENTIFIER_REGEX: Regex = Regex::new(
        r#"[_a-zA-Z]\w*"#
    ).expect("Failed to compile regex");

}


#[derive(Debug, Clone)]
pub enum TokenValue<'a> {

    FunctionMacroDef { export: bool },
    InlineMacroDef { export: bool },
    Endmacro,
    Bang,
    Equals,
    Colon,
    LabelDef { export: bool },
    Dot,
    Comma,
    SquareOpen,
    SquareClose,
    CurlyOpen,
    CurlyClose,
    CurrentPosition,
    Register (Registers),
    Number (Number),
    Identifier (&'a str),
    StringLiteral (Cow<'a, str>),
    Instruction (AsmInstruction),
    PseudoInstruction (PseudoInstructions)

}


#[derive(Debug)]
pub struct SourceToken<'a> {

    pub string: &'a str,
    pub line_index: usize,
    pub column: usize,
    pub unit_path: UnitPath<'a>

}

impl SourceToken<'_> {

    #[inline]
    pub fn line_number(&self) -> usize {
        self.line_index + 1
    }

}


#[derive(Debug, Clone)]
pub struct Token<'a> {

    pub value: TokenValue<'a>,
    pub source: Rc<SourceToken<'a>>,
    
}


pub type SourceCode<'a> = &'a [&'a str];


fn escape_string_copy(string: &str, checked_until: usize, token: &SourceToken, source: SourceCode) -> String {
    // use -1 because the escape character won't be copied
    let mut s = String::with_capacity(string.len() - 1);
    
    // Copy the part of the string before the escape character
    s.push_str(&string[..checked_until]);

    let mut escape = true;

    for (i, c) in string[checked_until + 1..].chars().enumerate() {
        if escape {
            escape = false;
            s.push(match c { // Characters that are part of an escape sequence
                'n' => '\n',
                'r' => '\r',
                '0' => '\0',
                't' => '\t',
                '\\' => '\\',
                '\'' => '\'',
                '"' => '"',
                c => error::invalid_escape_sequence(token, c, token.column + checked_until + i + 2, source)
            })
        } else if c == '\\' {
            escape = true;
        } else {
            s.push(c);
        }
    }

    s
}


fn escape_string<'a>(string: &'a str, token: &SourceToken, source: SourceCode<'a>) -> Cow<'a, str> {
    // Ignore the enclosing quote characters
    let string = &string[1..string.len() - 1];
    
    for (i, c) in string.chars().enumerate() {
        if c == '\\' {
            let copied_string = escape_string_copy(string, i, token, source);
            return Cow::Owned(copied_string);
        }
    }

    Cow::Borrowed(string)
}


#[inline]
fn is_decimal_numeric(c: char) -> bool {
    c.is_numeric() || c == '-' || c == '+' || c == '.'
}


pub type TokenList<'a> = VecDeque<Token<'a>>;
pub type TokenLines<'a> = VecDeque<TokenList<'a>>;


pub fn tokenize<'a>(source: SourceCode<'a>, unit_path: UnitPath<'a>) -> TokenLines<'a> {

    let mut last_unique_symbol: usize = 0;
    macro_rules! generate_unique_symbol {
        () => {{
            last_unique_symbol += 1;
            Box::new(
                format!("__unique_symbol_{}", last_unique_symbol)
            ).leak()
        }};
    }

    
    source.iter().enumerate().filter_map(
        |(line_index, raw_line)| {

            if raw_line.trim().is_empty() {
                return None;
            }

            let mut current_line = TokenList::new();

            for mat in TOKEN_REGEX.find_iter(raw_line) {

                let string = mat.as_str();

                if string.starts_with('#') {
                    break;
                }

                let token_rc = Rc::new(SourceToken {
                    string,
                    unit_path,
                    line_index,
                    column: mat.start() + 1
                });

                let token = token_rc.as_ref();
    
                let token_value = match token.string {

                    "=" => TokenValue::Equals,

                    ":" => TokenValue::Colon,
    
                    "&" => TokenValue::Identifier(generate_unique_symbol!()),

                    "!" => TokenValue::Bang,

                    "." => TokenValue::Dot,
    
                    "$" => TokenValue::CurrentPosition,
    
                    "%-" => TokenValue::InlineMacroDef { export: false },
                    "%%-" => TokenValue::InlineMacroDef { export: true },
    
                    "@" => TokenValue::LabelDef { export: false },
                    "@@" => TokenValue::LabelDef { export: true },
    
                    "%" => TokenValue::FunctionMacroDef { export: false },
                    "%%" => TokenValue::FunctionMacroDef { export: true },
    
                    "[" => TokenValue::SquareOpen,
                    "]" => TokenValue::SquareClose,
                    "{" => TokenValue::CurlyOpen,
                    "}" => TokenValue::CurlyClose,
    
                    "," => TokenValue::Comma,
    
                    string if string.starts_with("0x") => {
                        TokenValue::Number(Number::UnsignedInt(
                            u64::from_str_radix(&string[2..], 16)
                                .unwrap_or_else(|err| error::invalid_number_format(&token, source, err.to_string().as_str()))
                        ))
                    },

                    string if string.starts_with("0b") => {
                        TokenValue::Number(Number::UnsignedInt(
                            u64::from_str_radix(&string[2..], 2)
                                .unwrap_or_else(|err| error::invalid_number_format(&token, source, err.to_string().as_str()))
                        ))
                    },
    
                    string if string.starts_with(is_decimal_numeric) => {
                        TokenValue::Number(if string.contains('.') {
                            Number::Float(string.parse::<f64>().unwrap_or_else(|err| error::invalid_number_format(&token, source, err.to_string().as_str())))
                        } else if string.starts_with('-') {
                            Number::SignedInt(string.parse::<i64>().unwrap_or_else(|err| error::invalid_number_format(&token, source, err.to_string().as_str())))
                        } else {
                            Number::UnsignedInt(string.parse::<u64>().unwrap_or_else(|err| error::invalid_number_format(&token, source, err.to_string().as_str())))
                        })
                    },
    
                    string if string.starts_with('"') => {
    
                        if !string.ends_with('"') {
                            error::tokenizer_error(&token, source, "Unterminated string literal.");
                        }
    
                        let string = escape_string(string, &token, source);
    
                        TokenValue::StringLiteral(string)
                    },
            
                    string if string.starts_with('\'') => {
    
                        if !string.ends_with('\'') {
                            error::tokenizer_error(&token, source, "Unterminated character literal.");
                        }
    
                        let escaped_string = escape_string(string, &token, source);
    
                        if escaped_string.len() != 1 {
                            error::tokenizer_error(&token, source, format!("Invalid character literal. A character literal can only contain one character, but {} were found.", escaped_string.len()).as_str());
                        }
    
                        let c = escaped_string.chars().next().unwrap();
    
                        TokenValue::Number(Number::UnsignedInt(c as u64))
                    },
    
                    string => {
                        
                        if let Some(instruction) = AsmInstruction::from_name(string) {
                            TokenValue::Instruction(instruction)
    
                        } else if let Some(register) = Registers::from_name(string) {
                            TokenValue::Register(register)
                        
                        } else if let Some(pseudo_instruction) = PseudoInstructions::from_name(string) {
                            TokenValue::PseudoInstruction(pseudo_instruction)

                        } else if string == "endmacro" {
    
                            if let Some(last_token) = current_line.pop_back() {
                                if !matches!(last_token.value, TokenValue::FunctionMacroDef {..}) {
                                    error::tokenizer_error(&token, source, "Expected the macro modifier `%` before the 'endmacro' keyword.")
                                }
                            } else {
                                error::tokenizer_error(&token, source, "Expected the macro modifier `%` before the 'endmacro' keyword.")
                            }
    
                            TokenValue::Endmacro
    
                        } else if IDENTIFIER_REGEX.is_match(string) {
                            TokenValue::Identifier(string)
    
                        } else {
                            error::tokenizer_error(&token, source, "Invalid token.")
                        }
    
                    }
                };
    
                current_line.push_back(Token {
                    source: token_rc,
                    value: token_value,
                });
                
            }

            Some(current_line)
        }
    ).collect::<TokenLines>()
}

