use std::fmt::Display;
use std::path::Path;

use crate::data_types::DataType;
use crate::token::{TokenValue, LiteralValue, Number, Token, Priority};
use crate::error;
use crate::operations::Ops;
use crate::token::Value;

use regex::Regex;
use lazy_static::lazy_static;

use rust_vm_lib::ir::IRCode;


lazy_static! {

    static ref TOKEN_REGEX: Regex = Regex::new(r#"
        (?m)
        ((?:'|").*(?:'|"))
        |\w+
        |[+-]?\d+[.]\d*|[+-]?[.]\d+
        |->
        |==
        |<=
        |>=
        |!=
        |&&
        |\|\|
        |[-+*/%\[\](){}=:#<>!^&|~]
        |\S
    "#).unwrap();

}


struct StringToken<'a> {

    pub string: &'a str,
    pub line: usize,
    pub start: usize,

}


impl Display for StringToken<'_> {

    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.string)
    }

}


impl StringToken<'_> {

    pub fn new(string: &str, line: usize, start: usize) -> StringToken<'_> {
        StringToken {
            string,
            line,
            start
        }
    }

}


fn escape_string(string: &str) -> String {
    let mut s = String::with_capacity(string.len());

    let mut escape = false;

    for c in string.chars() {
        if escape {
            escape = false;
            s.push(match c {
                'n' => '\n',
                'r' => '\r',
                '0' => '\0',
                't' => '\t',
                '\\' => '\\',
                c => c
            })
        } else if c == '\\' {
            escape = true;
        }
    }

    s
}


#[inline]
fn is_numeric(c: char) -> bool {
    c.is_numeric() || c == '-' || c == '+' || c == '.'
}


#[inline]
fn may_be_value(token: Option<&Token>) -> bool {
    token.map(|t| matches!(t.value,
        TokenValue::Symbol(_) |
        TokenValue::ParClose |
        TokenValue::SquareClose
    )).unwrap_or(false)
}


pub fn tokenize<'a>(source: &'a IRCode, unit_path: &'a Path) -> Vec<Token<'a>> {

    // Divide the source code lines into string tokens 

    let matches: Vec<StringToken> = source.iter().enumerate().flat_map(
        |(line_number, line)| {
            let mut matches = Vec::new();
            for mat in TOKEN_REGEX.find_iter(line) {
                // Stop on comments
                if mat.as_str() == "#" {
                    break;
                }
                matches.push(
                    StringToken::new(mat.as_str(), line_number, mat.start())
                );
            }
            matches
        }
    ).collect();

    // Transform the string tokens into syntax tokens

    let mut tokens: Vec<Token> = Vec::with_capacity(matches.len());

    let mut parenthesis_depth: usize = 1;
    let mut square_depth: usize = 1;
    let mut curly_depth: usize = 1;
    let mut base_priority: usize = 0;

    for mat in matches {
        
        let token = match mat.string {

            "->" => TokenValue::Arrow,
            "{" => {
                curly_depth += 1;
                base_priority += Priority::Max as usize;
                TokenValue::ScopeOpen
            },
            "}" => {
                curly_depth -= 1;
                if curly_depth == 0 {
                    error::unmatched_delimiter(unit_path, '}', mat.line, mat.start, &source[mat.line], "Unexpected closing delimiter. Did you forget a '{'?")
                }
                base_priority -= Priority::Max as usize;
                TokenValue::ScopeClose
            },
            "(" => {
                parenthesis_depth += 1;
                base_priority += Priority::Max as usize;
                if may_be_value(tokens.last()) { TokenValue::Op(Ops::Call) } else { TokenValue::ParOpen }
            },
            ")" => {
                parenthesis_depth -= 1;
                if parenthesis_depth == 0 {
                    error::unmatched_delimiter(unit_path, ')', mat.line, mat.start, &source[mat.line], "Unexpected closing delimiter. Did you forget a '('?")
                }
                base_priority -= Priority::Max as usize;
                TokenValue::ParClose
            },
            "[" => {
                square_depth += 1;
                base_priority += Priority::Max as usize;
                TokenValue::SquareOpen
            },
            "]" => {
                square_depth -= 1;
                if square_depth == 0 {
                    error::unmatched_delimiter(unit_path, ']', mat.line, mat.start, &source[mat.line], "Unexpected closing delimiter. Did you forget a '['?")
                }
                base_priority -= Priority::Max as usize;
                TokenValue::SquareClose
            },
            ":" => TokenValue::Semicolon,
            "+" => TokenValue::Op(Ops::Add),
            "-" => TokenValue::Op(Ops::Sub),
            "*" => if may_be_value(tokens.last()) { TokenValue::Op(Ops::Mul) } else { TokenValue::Op(Ops::Deref) },
            "/" => TokenValue::Op(Ops::Div),
            "%" => TokenValue::Op(Ops::Mod),
            "=" => TokenValue::Op(Ops::Assign),
            "==" => TokenValue::Op(Ops::Equal),
            "!=" => TokenValue::Op(Ops::NotEqual),
            "<" => TokenValue::Op(Ops::Less),
            ">" => TokenValue::Op(Ops::Greater),
            "<=" => TokenValue::Op(Ops::LessEqual),
            ">=" => TokenValue::Op(Ops::GreaterEqual),
            "&" => if may_be_value(tokens.last()) { TokenValue::Op(Ops::BitwiseAnd) } else { TokenValue::Op(Ops::Ref) },
            "^" => TokenValue::Op(Ops::BitwiseXor),
            "<<" => TokenValue::Op(Ops::BitShiftLeft),
            ">>" => TokenValue::Op(Ops::BitShiftRight),
            "~" => TokenValue::Op(Ops::BitwiseNot),
            "!" => TokenValue::Op(Ops::LogicalNot),
            "&&" => TokenValue::Op(Ops::LogicalAnd),
            "||" => TokenValue::Op(Ops::LogicalOr),
            "|" => TokenValue::Op(Ops::BitwiseOr),

            string => {
                
                // Numbers
                if string.starts_with(is_numeric) {

                    if string.contains('.') {
                        TokenValue::Value(Value::Literal { value: LiteralValue::Numeric(Number::Float(string.parse::<f64>().unwrap_or_else(
                            |e| error::invalid_number(unit_path, string, mat.line, mat.start, &source[mat.line], e.to_string().as_str())
                        ))) })
                    } else if string.starts_with('-') {
                        TokenValue::Value(Value::Literal { value: LiteralValue::Numeric(Number::Int(string.parse::<i64>().unwrap_or_else(
                            |e| error::invalid_number(unit_path, string, mat.line, mat.start, &source[mat.line], e.to_string().as_str())
                        ))) })
                    } else {
                        TokenValue::Value(Value::Literal { value: LiteralValue::Numeric(Number::Uint(string.parse::<u64>().unwrap_or_else(
                            |e| error::invalid_number(unit_path, string, mat.line, mat.start, &source[mat.line], e.to_string().as_str())
                        ))) 
                    })
                    }

                // Strings
                } else if string.starts_with('"') {

                    TokenValue::Value(Value::Literal { 
                        value: LiteralValue::String(escape_string(string))
                    })
                
                } else if string.starts_with('\'') {
                    
                    let s = escape_string(string);
                    if s.len() != 1 {
                        error::invalid_char_literal(unit_path, &s, mat.line, mat.start, &source[mat.line], "Character literals can only be one character long")
                    }

                    TokenValue::Value(Value::Literal { 
                        value: LiteralValue::Char(s.chars().next().unwrap())
                    })

                } else {
                    
                    match string {

                        "fn" => TokenValue::Fn,
                        "return" => TokenValue::Op(Ops::Return),
                        "jmp" => TokenValue::Op(Ops::Jump),
                        "let" => TokenValue::Let,

                        "i8" => TokenValue::DataType(DataType::I8),
                        "i16" => TokenValue::DataType(DataType::I16),
                        "i32" => TokenValue::DataType(DataType::I32),
                        "i64" => TokenValue::DataType(DataType::I64),
                        "u8" => TokenValue::DataType(DataType::U8),
                        "u16" => TokenValue::DataType(DataType::U16),
                        "u32" => TokenValue::DataType(DataType::U32),
                        "u64" => TokenValue::DataType(DataType::U64),
                        "f32" => TokenValue::DataType(DataType::F32),
                        "f64" => TokenValue::DataType(DataType::F64),
                        "char" => TokenValue::DataType(DataType::Char),
                        "str" => TokenValue::DataType(DataType::String),

                        string => {
                            TokenValue::Symbol(string)
                        }

                    }

                }

            }
        };


        tokens.push(
            Token::new(token, mat.line, mat.start, unit_path, base_priority)
        );
    }

    tokens
}

