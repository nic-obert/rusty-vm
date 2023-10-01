use std::fmt::Display;
use std::path::Path;

use crate::data_types::DataType;
use crate::token::{TokenKind, LiteralValue, Number, Token, Priority, TokenList};
use crate::error;
use crate::operations::Ops;
use crate::token::Value;

use regex::Regex;
use lazy_static::lazy_static;

use rust_vm_lib::ir::IRCode;


lazy_static! {

    static ref TOKEN_REGEX: Regex = Regex::new(r#"(?m)((?:'|").*(?:'|"))|\w+|[+-]?\d+[.]\d*|[+-]?[.]\d+|->|==|<=|>=|!=|&&|\|\||[-+*/%\[\](){}=:#<>!^&|~]|\S"#).unwrap();

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
        TokenKind::ParClose |
        TokenKind::SquareClose |
        TokenKind::Value(_)
    )).unwrap_or(false)
}


fn is_symbol_name(name: &str) -> bool {
    let mut chars = name.chars();

    if let Some(c) = chars.next() {
        if !(c.is_alphabetic() || c == '_') {
            return false;
        }
    }

    for c in chars {
        if !(c.is_alphanumeric() || c == '_') {
            return false;
        }
    }

    true
}


pub fn tokenize<'a>(source: &'a IRCode, unit_path: &'a Path) -> TokenList<'a> {

    // Divide the source code lines into string tokens 

    let matches: Vec<StringToken> = source.iter().enumerate().flat_map(
        |(line_number, line)| {
            if line.trim().is_empty() {
                return Vec::new();
            }
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

    let mut tokens = TokenList::new();

    let mut parenthesis_depth: usize = 1;
    let mut square_depth: usize = 1;
    let mut curly_depth: usize = 1;
    let mut base_priority: usize = 0;

    for mat in matches {
        
        let token = match mat.string {

            "->" => TokenKind::Arrow,
            "{" => {
                curly_depth += 1;
                base_priority += Priority::Max as usize;
                TokenKind::ScopeOpen
            },
            "}" => {
                curly_depth -= 1;
                if curly_depth == 0 {
                    error::unmatched_delimiter(unit_path, '}', mat.line, mat.start, &source[mat.line], "Unexpected closing delimiter. Did you forget a '{'?")
                }
                base_priority -= Priority::Max as usize;
                TokenKind::ScopeClose
            },
            "(" => {
                parenthesis_depth += 1;
                base_priority += Priority::Max as usize;
                if may_be_value(tokens.last()) { TokenKind::Op(Ops::Call) } else { TokenKind::ParOpen }
            },
            ")" => {
                parenthesis_depth -= 1;
                if parenthesis_depth == 0 {
                    error::unmatched_delimiter(unit_path, ')', mat.line, mat.start, &source[mat.line], "Unexpected closing delimiter. Did you forget a '('?")
                }
                base_priority -= Priority::Max as usize;
                TokenKind::ParClose
            },
            "[" => {
                square_depth += 1;
                base_priority += Priority::Max as usize;
                TokenKind::SquareOpen
            },
            "]" => {
                square_depth -= 1;
                if square_depth == 0 {
                    error::unmatched_delimiter(unit_path, ']', mat.line, mat.start, &source[mat.line], "Unexpected closing delimiter. Did you forget a '['?")
                }
                base_priority -= Priority::Max as usize;
                TokenKind::SquareClose
            },
            ":" => TokenKind::Colon,
            ";" => TokenKind::Semicolon,
            "+" => TokenKind::Op(Ops::Add),
            "-" => TokenKind::Op(Ops::Sub),
            "*" => if may_be_value(tokens.last()) { TokenKind::Op(Ops::Mul) } else { TokenKind::Op(Ops::Deref) },
            "/" => TokenKind::Op(Ops::Div),
            "%" => TokenKind::Op(Ops::Mod),
            "=" => TokenKind::Op(Ops::Assign),
            "==" => TokenKind::Op(Ops::Equal),
            "!=" => TokenKind::Op(Ops::NotEqual),
            "<" => TokenKind::Op(Ops::Less),
            ">" => TokenKind::Op(Ops::Greater),
            "<=" => TokenKind::Op(Ops::LessEqual),
            ">=" => TokenKind::Op(Ops::GreaterEqual),
            "&" => if may_be_value(tokens.last()) { TokenKind::Op(Ops::BitwiseAnd) } else { TokenKind::Op(Ops::Ref) },
            "^" => TokenKind::Op(Ops::BitwiseXor),
            "<<" => TokenKind::Op(Ops::BitShiftLeft),
            ">>" => TokenKind::Op(Ops::BitShiftRight),
            "~" => TokenKind::Op(Ops::BitwiseNot),
            "!" => TokenKind::Op(Ops::LogicalNot),
            "&&" => TokenKind::Op(Ops::LogicalAnd),
            "||" => TokenKind::Op(Ops::LogicalOr),
            "|" => TokenKind::Op(Ops::BitwiseOr),

            string => {
                
                // Numbers
                if string.starts_with(is_numeric) {

                    if string.contains('.') {
                        TokenKind::Value(Value::Literal { value: LiteralValue::Numeric(Number::Float(string.parse::<f64>().unwrap_or_else(
                            |e| error::invalid_number(unit_path, string, mat.line, mat.start, &source[mat.line], e.to_string().as_str())
                        ))) })
                    } else if string.starts_with('-') {
                        TokenKind::Value(Value::Literal { value: LiteralValue::Numeric(Number::Int(string.parse::<i64>().unwrap_or_else(
                            |e| error::invalid_number(unit_path, string, mat.line, mat.start, &source[mat.line], e.to_string().as_str())
                        ))) })
                    } else {
                        TokenKind::Value(Value::Literal { value: LiteralValue::Numeric(Number::Uint(string.parse::<u64>().unwrap_or_else(
                            |e| error::invalid_number(unit_path, string, mat.line, mat.start, &source[mat.line], e.to_string().as_str())
                        ))) 
                    })
                    }

                // Strings
                } else if string.starts_with('"') {

                    TokenKind::Value(Value::Literal { 
                        value: LiteralValue::String(escape_string(string))
                    })
                
                } else if string.starts_with('\'') {
                    
                    let s = escape_string(string);
                    if s.len() != 1 {
                        error::invalid_char_literal(unit_path, &s, mat.line, mat.start, &source[mat.line], "Character literals can only be one character long")
                    }

                    TokenKind::Value(Value::Literal { 
                        value: LiteralValue::Char(s.chars().next().unwrap())
                    })

                } else {
                    
                    match string {

                        "fn" => TokenKind::Fn,
                        "return" => TokenKind::Op(Ops::Return),
                        "jmp" => TokenKind::Op(Ops::Jump),
                        "let" => TokenKind::Let,

                        "i8" => TokenKind::DataType(DataType::I8),
                        "i16" => TokenKind::DataType(DataType::I16),
                        "i32" => TokenKind::DataType(DataType::I32),
                        "i64" => TokenKind::DataType(DataType::I64),
                        "u8" => TokenKind::DataType(DataType::U8),
                        "u16" => TokenKind::DataType(DataType::U16),
                        "u32" => TokenKind::DataType(DataType::U32),
                        "u64" => TokenKind::DataType(DataType::U64),
                        "f32" => TokenKind::DataType(DataType::F32),
                        "f64" => TokenKind::DataType(DataType::F64),
                        "char" => TokenKind::DataType(DataType::Char),
                        "str" => TokenKind::DataType(DataType::String),
                        "void" => TokenKind::DataType(DataType::Void),

                        string => {
                        
                            if !is_symbol_name(string) {
                                error::invalid_token(unit_path, string, mat.line, mat.start, &source[mat.line], "The token doesn't have meaning");
                            }

                            TokenKind::Value(Value::Symbol { id: string })
                        }

                    }

                }

            }
        };


        tokens.append(
            Token::new(token, mat.line, mat.start, unit_path, base_priority)
        );
    }

    tokens
}

