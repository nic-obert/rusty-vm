use std::fmt::Display;
use std::path::Path;

use crate::data_types::DataType;
use crate::token::{Token, LiteralValue, Number};
use crate::error;
use crate::operations::Ops;
use crate::token::Value;

use regex::Regex;
use lazy_static::lazy_static;

use rust_vm_lib::ir::IRCode;


lazy_static! {

    // TODO: add support for strings, char literals, escape chars in strings
    static ref TOKEN_REGEX: Regex = Regex::new(r#"(?m)\w+|[+-]?\d+[.]\d*|[+-]?[.]\d+|->|==|<=|>=|!=|[-+*/%\[\](){}=:#<>]|\S"#).unwrap();

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


fn is_numeric(c: char) -> bool {
    c.is_numeric() || c == '-' || c == '+' || c == '.'
}


pub fn tokenize<'a>(source: &'a IRCode, unit_path: &Path) -> Vec<Token<'a>> {

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
    for mat in matches {
        
        let token = match mat.string {

            "->" => Token::Arrow,
            "{" => Token::ScopeOpen,
            "}" => Token::ScopeClose,
            "(" => Token::ParOpen,
            ")" => Token::ParClose,
            "[" => Token::GroupOpen,
            "]" => Token::GroupClose,
            ":" => Token::Semicolon,
            "+" => Token::Op(Ops::Add),
            "-" => Token::Op(Ops::Sub),
            "*" => if tokens.last().map(|t| !matches!(t, Token::Symbol(_))).unwrap_or(true) { Token::Op(Ops::Deref) } else { Token::Op(Ops::Mul) },
            "/" => Token::Op(Ops::Div),
            "%" => Token::Op(Ops::Mod),
            "=" => Token::Op(Ops::Assign),
            "==" => Token::Op(Ops::Equal),
            "!=" => Token::Op(Ops::NotEqual),
            "<" => Token::Op(Ops::Less),
            ">" => Token::Op(Ops::Greater),
            "<=" => Token::Op(Ops::LessEqual),
            ">=" => Token::Op(Ops::GreaterEqual),
            "&" => Token::Op(Ops::Ref),

            string => {

                if string.starts_with(is_numeric) {
                    if string.contains('.') {
                        Token::Value(Value::Literal { value: LiteralValue::Numeric(Number::Float(string.parse::<f64>().unwrap_or_else(
                            |e| error::invalid_number(unit_path, string, mat.line, mat.start, &source[mat.line], e.to_string().as_str())
                        ))) })
                    } else if string.starts_with('-') {
                        Token::Value(Value::Literal { value: LiteralValue::Numeric(Number::Int(string.parse::<i64>().unwrap_or_else(
                            |e| error::invalid_number(unit_path, string, mat.line, mat.start, &source[mat.line], e.to_string().as_str())
                        ))) })
                    } else {
                        Token::Value(Value::Literal { value: LiteralValue::Numeric(Number::Uint(string.parse::<u64>().unwrap_or_else(
                            |e| error::invalid_number(unit_path, string, mat.line, mat.start, &source[mat.line], e.to_string().as_str())
                        ))) })
                    }
                } else {
                    
                    match string {

                        "fn" => Token::Fn,
                        "return" => Token::Op(Ops::Return),
                        "jmp" => Token::Op(Ops::Jump),

                        "i8" => Token::DataType(DataType::I8),
                        "i16" => Token::DataType(DataType::I16),
                        "i32" => Token::DataType(DataType::I32),
                        "i64" => Token::DataType(DataType::I64),
                        "u8" => Token::DataType(DataType::U8),
                        "u16" => Token::DataType(DataType::U16),
                        "u32" => Token::DataType(DataType::U32),
                        "u64" => Token::DataType(DataType::U64),
                        "f32" => Token::DataType(DataType::F32),
                        "f64" => Token::DataType(DataType::F64),
                        "char" => Token::DataType(DataType::Char),
                        "str" => Token::DataType(DataType::String),

                        string => {
                            Token::Symbol(string)
                        }

                    }

                }

            }
        };

        tokens.push(token);
    }

    tokens
}

