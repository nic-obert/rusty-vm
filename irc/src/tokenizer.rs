use std::path::Path;

use crate::data_types::DataType;
use crate::token::{TokenKind, LiteralValue, Number, Token, Priority, StringToken};
use crate::error;
use crate::operations::Ops;
use crate::token::Value;
use crate::token_tree::TokenTree;

use regex::Regex;
use lazy_static::lazy_static;

use rust_vm_lib::ir::IRCode;


lazy_static! {

    static ref TOKEN_REGEX: Regex = Regex::new(r#"(?m)((?:'|").*(?:'|"))|\w+|[+-]?\d+[.]\d*|[+-]?[.]\d+|->|==|<=|>=|!=|&&|\|\||[-+*/%\[\](){}=:#<>!^&|~]|\S"#).unwrap();

}


fn escape_string_copy(string: &str, checked_until: usize, unit_path: &Path, token: &StringToken, source: &IRCode) -> String {
    // use -1 because the escape character won't be copied
    let mut s = String::with_capacity(string.len() - 1);

    // Copy the part of the string before the escape character
    s.push_str(&string[..checked_until]);

    let mut escape = true;

    for (i, c) in string[checked_until + 1..].chars().enumerate() {
        if escape {
            escape = false;
            s.push(match c {
                'n' => '\n',
                'r' => '\r',
                '0' => '\0',
                't' => '\t',
                '\\' => '\\',
                c => error::invalid_escape_character(unit_path, c, token.line_number(), token.column + checked_until + i + 2, &source[token.line_index()], "Invalid escape character")
            })
        } else if c == '\\' {
            escape = true;
        } else {
            s.push(c);
        }
    }

    s
}


fn escape_string<'a>(string: &'a str, unit_path: &Path, token: &StringToken, source: &IRCode) -> &'a str {
    // Ignore the enclosing quote characters
    let string = &string[1..string.len() - 1];
    
    for (i, c) in string.chars().enumerate() {
        if c == '\\' {
            let copied_string = escape_string_copy(string, i, unit_path, token, source);
            return Box::leak(copied_string.into_boxed_str());
        }
    }

    string
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


/// Useful struct to keep track of the status of the tokenizer
struct TokenizerStatus {

    pub parenthesis_depth: usize,
    pub square_depth: usize,
    pub curly_depth: usize,
    pub base_priority: i32,
    priority_delta: i32,

}

impl TokenizerStatus {

    pub fn new() -> TokenizerStatus {
        TokenizerStatus {
            parenthesis_depth: 1,
            square_depth: 1,
            curly_depth: 1,
            base_priority: 0,
            priority_delta: 0,
        }
    }

    pub fn update(&mut self) {
        self.base_priority += self.priority_delta;
        self.priority_delta = 0;
    }

    pub fn enter_parenthesis(&mut self) {
        self.parenthesis_depth += 1;
        self.base_priority += Priority::Max as i32;
    }

    pub fn leave_parenthesis(&mut self) -> Result<(), ()> {
        self.parenthesis_depth -= 1;
        self.base_priority -= Priority::Max as i32;

        if self.parenthesis_depth == 0 {
            Err(())
        } else {
            Ok(())
        }
    }

    pub fn enter_square(&mut self) {
        self.square_depth += 1;
        self.base_priority += Priority::Max as i32;
    }

    pub fn leave_square(&mut self) -> Result<(), ()> {
        self.square_depth -= 1;
        self.base_priority -= Priority::Max as i32;

        if self.square_depth == 0 {
            Err(())
        } else {
            Ok(())
        }
    }

    pub fn enter_curly(&mut self) {
        self.curly_depth += 1;
        self.priority_delta = Priority::Max as i32;
    }

    pub fn leave_curly(&mut self) -> Result<(), ()> {
        self.curly_depth -= 1;
        self.priority_delta = - (Priority::Max as i32);

        if self.curly_depth == 0 {
            Err(())
        } else {
            Ok(())
        }
    }

}


/// Divide the source code into meaningful string tokens
fn lex(source: &IRCode) -> Vec<StringToken<'_>> {
    source.iter().enumerate().flat_map(
        |(line_index, line)| {
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
                    StringToken::new(mat.as_str(), line_index, mat.start() + 1)
                );
            }
            matches
        }
    ).collect()
}


pub fn tokenize<'a>(source: &'a IRCode, unit_path: &'a Path) -> TokenTree<'a> {

    let raw_tokens = lex(source);

    let mut tokens = TokenTree::new();

    let mut ts = TokenizerStatus::new();

    for token in raw_tokens {
        
        let token_kind = match token.string {

            "->" => TokenKind::Arrow,
            "{" => {
                ts.enter_curly();
                TokenKind::ScopeOpen
            },
            "}" => {
                ts.leave_curly().unwrap_or_else(
                    |_| error::unmatched_delimiter(unit_path, '}', token.line_number(), token.column, &source[token.line_index()], "Unexpected closing delimiter. Did you forget a '{'?")
                );
                TokenKind::ScopeClose
            },
            "(" => {
                ts.enter_parenthesis();

                // Get around the borrow checker not recognizing that last_token immutable reference is dropped before tokens is borrowed mutably when appending the token
                let last_node = tokens.last_node();
                
                if may_be_value(last_node.map(|node| &node.item)) {

                    // In this branch, last_node is Some because may_be_value returns false if last_node is None
                    let last_node = last_node.unwrap();

                    if matches!(last_node.item.value, TokenKind::Value(Value::Symbol { id: _ })) && last_node.left().map(|node| matches!(node.item.value, TokenKind::Fn)).unwrap_or(false) {
                        // Syntax: fn <symbol> (
                        TokenKind::FunctionParamsOpen
                    } else {
                        // Syntax: <value-like> (
                        TokenKind::Op(Ops::Call)
                    }
                } else {
                    // Syntax: <not-a-value> (
                    TokenKind::ParOpen
                }
            },
            ")" => {
                ts.leave_parenthesis().unwrap_or_else(
                    |_| error::unmatched_delimiter(unit_path, ')', token.line_number(), token.column, &source[token.line_index()], "Unexpected closing delimiter. Did you forget a '('?")
                );
                TokenKind::ParClose
            },
            "[" => {
                ts.enter_square();
                TokenKind::SquareOpen
            },
            "]" => {
                ts.leave_square().unwrap_or_else(
                    |_| error::unmatched_delimiter(unit_path, ']', token.line_number(), token.column, &source[token.line_index()], "Unexpected closing delimiter. Did you forget a '['?")
                );
                TokenKind::SquareClose
            },
            ":" => TokenKind::Colon,
            ";" => TokenKind::Semicolon,
            "," => TokenKind::Comma,
            "+" => TokenKind::Op(Ops::Add),
            "-" => TokenKind::Op(Ops::Sub),
            "*" => {
                let last_token = unsafe { (*(&tokens as *const TokenTree)).last_item() };
                if may_be_value(last_token) { TokenKind::Op(Ops::Mul) } else { TokenKind::Op(Ops::Deref) }
            },
            "/" => TokenKind::Op(Ops::Div),
            "%" => TokenKind::Op(Ops::Mod),
            "=" => TokenKind::Op(Ops::Assign),
            "==" => TokenKind::Op(Ops::Equal),
            "!=" => TokenKind::Op(Ops::NotEqual),
            "<" => TokenKind::Op(Ops::Less),
            ">" => TokenKind::Op(Ops::Greater),
            "<=" => TokenKind::Op(Ops::LessEqual),
            ">=" => TokenKind::Op(Ops::GreaterEqual),
            "&" => {
                let last_token = unsafe { (*(&tokens as *const TokenTree)).last_item() };
                if may_be_value(last_token) { TokenKind::Op(Ops::BitwiseAnd) } else { TokenKind::Op(Ops::Ref) }
            },
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
                            |e| error::invalid_number(unit_path, string, token.line_number(), token.column, &source[token.line_index()], e.to_string().as_str())
                        ))) })
                    } else if string.starts_with('-') {
                        TokenKind::Value(Value::Literal { value: LiteralValue::Numeric(Number::Int(string.parse::<i64>().unwrap_or_else(
                            |e| error::invalid_number(unit_path, string, token.line_number(), token.column, &source[token.line_index()], e.to_string().as_str())
                        ))) })
                    } else {
                        TokenKind::Value(Value::Literal { value: LiteralValue::Numeric(Number::Uint(string.parse::<u64>().unwrap_or_else(
                            |e| error::invalid_number(unit_path, string, token.line_number(), token.column, &source[token.line_index()], e.to_string().as_str())
                        ))) 
                    })
                    }

                // Strings
                } else if string.starts_with('"') {

                    if string.len() == 1 {
                        error::unmatched_delimiter(unit_path, '"', token.line_number(), token.column, &source[token.line_index()], "Unexpected closing delimiter. Did you forget a '\"'?")
                    }

                    TokenKind::Value(Value::Literal { 
                        value: LiteralValue::String(escape_string(string, unit_path, &token, source))
                    })
                
                } else if string.starts_with('\'') {

                    if string.len() == 1 {
                        error::unmatched_delimiter(unit_path, '\'', token.line_number(), token.column, &source[token.line_index()], "Unexpected closing delimiter. Did you forget a \"'?\"?")
                    }
                    
                    let s = escape_string(string, unit_path, &token, source);
                    if s.len() != 1 {
                        error::invalid_char_literal(unit_path, s, token.line_number(), token.column, &source[token.line_index()], "Character literals can only be one character long")
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
                        "mut" => TokenKind::Mut,

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
                                error::invalid_token(unit_path, string, token.line_number(), token.column, &source[token.line_index()], "The token doesn't have meaning");
                            }

                            TokenKind::Value(Value::Symbol { id: string })
                        }

                    }

                }

            }
        };

        tokens.append(
            Token::new(token_kind, token, unit_path, ts.base_priority)
        );

        ts.update();
    }

    tokens
}

