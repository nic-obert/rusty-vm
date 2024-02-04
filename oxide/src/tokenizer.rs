use std::borrow::Cow;
use std::path::Path;

use crate::data_types::{DataType, LiteralValue, Number};
use crate::symbol_table::{ScopeDiscriminant, SymbolTable};
use crate::token::{TokenKind, Token, Priority, StringToken};
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


fn escape_string_copy(string: &str, checked_until: usize, token: &StringToken, source: &IRCode) -> String {
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
                c => error::invalid_escape_character(token.unit_path, c, token.column + checked_until + i + 2, token.line_index(), source, "Invalid escape character")
            })
        } else if c == '\\' {
            escape = true;
        } else {
            s.push(c);
        }
    }

    s
}


fn escape_string<'a>(string: &'a str, token: &StringToken, source: &IRCode) -> Cow<'a, str> {
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
fn is_numeric(c: char) -> bool {
    c.is_numeric() || c == '-' || c == '+' || c == '.'
}


#[inline]
fn may_be_expression(token: Option<&Token>) -> bool {
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


#[inline]
const fn is_data_type_precursor(token: &Token) -> bool {
    matches!(token.value, TokenKind::DataType(_) | TokenKind::ArrayTypeOpen | TokenKind::Colon | TokenKind::Arrow | TokenKind::RefType)
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
        self.priority_delta += Priority::Delimiter as i32;
    }

    pub fn leave_parenthesis(&mut self) -> Result<(), ()> {
        self.parenthesis_depth -= 1;
        self.priority_delta -= Priority::Delimiter as i32;

        if self.parenthesis_depth == 0 {
            Err(())
        } else {
            Ok(())
        }
    }

    pub fn enter_square(&mut self) {
        self.square_depth += 1;
        self.priority_delta += Priority::Delimiter as i32;
    }

    pub fn leave_square(&mut self) -> Result<(), ()> {
        self.square_depth -= 1;
        self.priority_delta -= Priority::Delimiter as i32;

        if self.square_depth == 0 {
            Err(())
        } else {
            Ok(())
        }
    }

    pub fn enter_curly(&mut self) {
        self.curly_depth += 1;
        self.priority_delta = Priority::Delimiter as i32;
    }

    pub fn leave_curly(&mut self) -> Result<(), ()> {
        self.curly_depth -= 1;
        self.priority_delta = - (Priority::Delimiter as i32);

        if self.curly_depth == 0 {
            Err(())
        } else {
            Ok(())
        }
    }

}


/// Divide the source code into meaningful string tokens
fn lex<'a>(source: &'a IRCode, unit_path: &'a Path) -> impl Iterator<Item = StringToken<'a>> {
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
                    StringToken {
                        string: mat.as_str(),
                        unit_path,
                        line_index,
                        column: mat.start() + 1
                    }
                );
            }
            matches
        }
    )
}


/// Divide the source code into syntax tokens
pub fn tokenize<'a>(source: &'a IRCode, unit_path: &'a Path, symbol_table: &mut SymbolTable<'a>) -> TokenTree<'a> {

    let raw_tokens = lex(source, unit_path);

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
                    |_| error::unmatched_delimiter('}', &token, source, "Unexpected closing delimiter. Did you forget a '{'?")
                );
                TokenKind::ScopeClose
            },
            "(" => {
                ts.enter_parenthesis();

                let last_node = tokens.last_node();
                
                if may_be_expression(last_node.map(|node| &node.item)) {

                    // Unwrap is safe because `last_node` is an expression
                    let last_node = last_node.unwrap();

                    if matches!(last_node.item.value, TokenKind::Value(Value::Symbol { .. })) && last_node.left().map(|node| matches!(node.item.value, TokenKind::Fn)).unwrap_or(false) {
                        // Syntax: fn <symbol> (
                        TokenKind::FunctionParamsOpen
                    } else {
                        // Syntax: <value-like> (
                        TokenKind::Op(Ops::FunctionCallOpen)
                    }
                } else {
                    // Syntax: <not-a-value> (
                    TokenKind::ParOpen
                }
            },
            ")" => {
                ts.leave_parenthesis().unwrap_or_else(
                    |_| error::unmatched_delimiter(')', &token, source, "Unexpected closing delimiter. Did you forget a '('?")
                );
                TokenKind::ParClose
            },
            "[" => {
                ts.enter_square();

                let last_token = tokens.last_node().map(|node| &node.item);
                last_token.map(|last_token| if is_data_type_precursor(last_token) {
                    // Syntax: <data-type-precursor> [
                    TokenKind::ArrayTypeOpen
                } else if may_be_expression(Some(last_token)) {
                    // Syntax: <possible-array-type> [
                    TokenKind::Op(Ops::ArrayIndexOpen)
                } else {
                    // Syntax: <not-a-data-type-precursor> [
                    TokenKind::ArrayOpen
                }).unwrap_or(
                    // Syntax: <nothing> [
                    TokenKind::ArrayOpen
                )
            },
            "]" => {
                ts.leave_square().unwrap_or_else(
                    |_| error::unmatched_delimiter(']', &token, source, "Unexpected closing delimiter. Did you forget a '['?")
                );
                TokenKind::SquareClose
            },
            ":" => TokenKind::Colon,
            ";" => TokenKind::Semicolon,
            "," => TokenKind::Comma,
            "+" => TokenKind::Op(Ops::Add),
            "-" => TokenKind::Op(Ops::Sub),
            "*" => {
                let last_token = tokens.last_node().map(|node| &node.item);
                if may_be_expression(last_token) { TokenKind::Op(Ops::Mul) } else { TokenKind::Op(Ops::Deref { mutable: false }) }
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
                let last_token = tokens.last_node().map(|node| &node.item);
                if may_be_expression(last_token) {
                    // Syntax: <value-like> &
                    TokenKind::Op(Ops::BitwiseAnd)
                } else if let Some(last_token) = last_token {
                    if is_data_type_precursor(last_token) {
                        // Syntax: <data-type-precursor> &
                        TokenKind::RefType
                    } else {
                        // Syntax: <not-a-data-type-precursor> &
                        TokenKind::Op(Ops::Ref { mutable: false })
                    }
                } else {
                    // Syntax: <nothing> &
                    TokenKind::Op(Ops::Ref { mutable: false })
                }
            },
            "^" => TokenKind::Op(Ops::BitwiseXor),
            "<<" => TokenKind::Op(Ops::BitShiftLeft),
            ">>" => TokenKind::Op(Ops::BitShiftRight),
            "~" => TokenKind::Op(Ops::BitwiseNot),
            "!" => TokenKind::Op(Ops::LogicalNot),
            "&&" => TokenKind::Op(Ops::LogicalAnd),
            "||" => TokenKind::Op(Ops::LogicalOr),
            "|" => TokenKind::Op(Ops::BitwiseOr),

            // Numbers
            string if string.starts_with(is_numeric) => {
                if string.contains('.') {
                    TokenKind::Value(Value::Literal { value: LiteralValue::Numeric(Number::Float(string.parse::<f64>().unwrap_or_else(
                        |e| error::invalid_number(&token, source, e.to_string().as_str())
                    ))) })
                } else if string.starts_with('-') {
                    TokenKind::Value(Value::Literal { value: LiteralValue::Numeric(Number::Int(string.parse::<i64>().unwrap_or_else(
                        |e| error::invalid_number(&token, source, e.to_string().as_str())
                    ))) })
                } else {
                    TokenKind::Value(Value::Literal { value: LiteralValue::Numeric(Number::Uint(string.parse::<u64>().unwrap_or_else(
                        |e| error::invalid_number(&token, source, e.to_string().as_str())
                    ))) }) 
                }
            },

            // Strings within ""
            string if string.starts_with('"') => {
                if string.len() == 1 {
                    error::unmatched_delimiter('"', &token, source, "Unexpected closing delimiter. Did you forget a '\"'?")
                }

                let string = escape_string(string, &token, source);

                // Add the string to the static string table, store the id in the token
                let static_id = symbol_table.add_static_string(string);

                TokenKind::Value(Value::Literal { 
                    value: LiteralValue::StaticString(static_id)
                })
            },

            // Characters within ''
            string if string.starts_with('\'') => {
                if string.len() == 1 {
                    error::unmatched_delimiter('\'', &token, source, "Unexpected closing delimiter. Did you forget a \"'?\"?")
                }
                
                let s = escape_string(string, &token, source);
                if s.len() != 1 {
                    error::invalid_char_literal(&s, &token, source, "Character literals can only be one character long")
                }

                TokenKind::Value(Value::Literal { 
                    value: LiteralValue::Char(s.chars().next().unwrap())
                })
            },

            // Reserved keywords
            "fn" => TokenKind::Fn,
            "return" => TokenKind::Op(Ops::Return),
            "let" => TokenKind::Let,
            "mut" => TokenKind::Mut,
            "as" => TokenKind::As,
            "if" => TokenKind::If,
            "else" => TokenKind::Else,
            "while" => TokenKind::While,
            "loop" => TokenKind::Loop,
            "true" => TokenKind::Value(Value::Literal { value: LiteralValue::Bool(true) }),
            "false" => TokenKind::Value(Value::Literal { value: LiteralValue::Bool(false) }),
            "const" => TokenKind::Const,
            "break" => TokenKind::Op(Ops::Break),
            "continue" => TokenKind::Op(Ops::Continue),
            "typedef" => TokenKind::TypeDef,
            
            // Data types
            "i8" => TokenKind::DataType(DataType::I8.into()),
            "i16" => TokenKind::DataType(DataType::I16.into()),
            "i32" => TokenKind::DataType(DataType::I32.into()),
            "i64" => TokenKind::DataType(DataType::I64.into()),
            "u8" => TokenKind::DataType(DataType::U8.into()),
            "u16" => TokenKind::DataType(DataType::U16.into()),
            "u32" => TokenKind::DataType(DataType::U32.into()),
            "u64" => TokenKind::DataType(DataType::U64.into()),
            "f32" => TokenKind::DataType(DataType::F32.into()),
            "f64" => TokenKind::DataType(DataType::F64.into()),
            "char" => TokenKind::DataType(DataType::Char.into()),
            "str" => TokenKind::DataType(DataType::RawString { length: 0 }.into()),
            "String" => TokenKind::DataType(DataType::String.into()),
            "void" => TokenKind::DataType(DataType::Void.into()),
            "bool" => TokenKind::DataType(DataType::Bool.into()),
            "usize" => TokenKind::DataType(DataType::Usize.into()),
            "isize" => TokenKind::DataType(DataType::Isize.into()),
            
            // Variable names
            string => {
                if !is_symbol_name(string) {
                    error::invalid_token(&token, source, "Not a valid symbol name.")
                }

                TokenKind::Value(Value::Symbol { name: string, scope_discriminant: ScopeDiscriminant::default() })
            }
        };

        tokens.append(
            Token::new(token_kind, token, ts.base_priority)
        );

        ts.update();
    }

    tokens
}

