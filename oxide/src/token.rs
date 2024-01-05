use std::fmt::Display;
use std::path::Path;

use crate::operations::Ops;
use crate::data_types::DataType;


#[derive(Debug)]
pub struct StringToken<'a> {

    pub string: &'a str,
    line_index: usize,
    pub column: usize,

}

impl Display for StringToken<'_> {

    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.string)
    }

}

impl StringToken<'_> {

    pub fn new(string: &str, line_index: usize, start: usize) -> StringToken<'_> {
        StringToken {
            string,
            line_index,
            column: start
        }
    }

    /// Returns the line number of the token, starting from 1
    /// 
    /// This is used to display the line number in the error message.
    #[inline]
    pub fn line_number(&self) -> usize {
        self.line_index + 1
    }

    /// Returns the line index of the token, starting from 0. 
    /// 
    /// This is used to index the line in the source code.
    #[inline]
    pub fn line_index(&self) -> usize {
        self.line_index
    }

}


#[derive(Debug, PartialEq)]
pub enum Value<'a> {

    Literal { value: LiteralValue<'a> },
    Symbol { id: &'a str }

}


impl Display for Value<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Literal { value } => write!(f, "Literal({})", value),
            Value::Symbol { id } => write!(f, "Ref({})", id),
        }
    }
}


#[derive(Debug, PartialEq)]
pub enum Number {

    Int(i64),
    Uint(u64),
    Float(f64)

}


impl Display for Number {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Number::Int(n) => write!(f, "{}", n),
            Number::Uint(n) => write!(f, "{}", n),
            Number::Float(n) => write!(f, "{}", n),
        }
    }
}


#[derive(Debug, PartialEq)]
pub enum LiteralValue<'a> {

    Char (char),
    String (&'a str),

    Array { dt: DataType, items: Vec<Value<'a>> },

    Numeric (Number),

}


impl LiteralValue<'_> {

    pub fn data_type(&self) -> DataType {
        match self {
            LiteralValue::Char(_) => DataType::Char,
            LiteralValue::String(_) => DataType::String,
            LiteralValue::Array { dt, .. } => DataType::Array(Box::new(dt.clone())),
            LiteralValue::Numeric(n) => match n {
                // Use a default 32-bit type for numbers. If the number is too big, use a 64-bit type.
                Number::Int(i) => if *i > std::i32::MAX as i64 || *i < std::i32::MIN as i64 { DataType::I64 } else { DataType::I32 },
                Number::Uint(u) => if *u > std::u32::MAX as u64 { DataType::U64 } else { DataType::U32 },
                Number::Float(f) => if *f > std::f32::MAX as f64 || *f < std::f32::MIN as f64 { DataType::F64 } else { DataType::F32 },
            },
        }
    }

}


impl Display for LiteralValue<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LiteralValue::Char(c) => write!(f, "'{}'", c),
            LiteralValue::String(s) => write!(f, "\"{}\"", s),
            LiteralValue::Array { dt, items } => write!(f, "[{}]: [{:?}]", dt, items),
            LiteralValue::Numeric(n) => write!(f, "{}", n),
        }
    }
}


#[derive(Debug, PartialEq)]
pub enum TokenKind<'a> {

    Op (Ops),
    Value (Value<'a>),
    DataType (DataType),

    RefType,

    Fn,
    Let,
    As,

    Arrow,
    Semicolon,
    Colon,
    Comma,
    Mut,

    ArrayTypeOpen,
    ArrayOpen,
    SquareClose,

    FunctionParamsOpen,
    ParOpen,
    ParClose,

    ScopeOpen,
    ScopeClose,

}


pub enum Priority {

    Zero = 0,
    Least,

    Declaration,

    AddSub,
    MulDivMod,

    LogicalOr,
    LogicalAnd,
    BitwiseOr,
    BitwiseXor,
    BitwiseAnd,
    Equality,
    Comparison,
    Bitshift,
    Not,

    Ref,

    Max

}


impl TokenKind<'_> {

    pub fn type_priority(&self) -> i32 {
        (match self {
            TokenKind::Op(op) => match op {

                Ops::Add |
                Ops::Sub
                 => Priority::AddSub,

                Ops::Mul |
                Ops::Div |
                Ops::Mod
                 => Priority::MulDivMod,

                Ops::Return |
                Ops::Jump |
                Ops::Assign
                 => Priority::Least,

                Ops::Deref |
                Ops::Ref
                 => Priority::Ref,

                Ops::Call => Priority::Max,
                
                Ops::Equal |
                Ops::NotEqual
                 => Priority::Equality,

                Ops::Greater |
                Ops::Less |
                Ops::GreaterEqual |
                Ops::LessEqual
                 => Priority::Comparison,

                Ops::BitwiseNot |
                Ops::LogicalNot
                 => Priority::Not,

                Ops::LogicalAnd => Priority::LogicalAnd,
                Ops::LogicalOr => Priority::LogicalOr,

                Ops::BitShiftRight |
                Ops::BitShiftLeft
                 => Priority::Bitshift,

                Ops::BitwiseOr => Priority::BitwiseOr,
                Ops::BitwiseAnd => Priority::BitwiseAnd,
                Ops::BitwiseXor => Priority::BitwiseXor,   

            },

            TokenKind::Fn |
            TokenKind::Let
                => Priority::Declaration,

            TokenKind::Value(_) |
            TokenKind::DataType(_) |
            TokenKind::Arrow |
            TokenKind::Semicolon |
            TokenKind::Colon |
            TokenKind::Comma |
            TokenKind::ScopeClose |
            TokenKind::SquareClose |
            TokenKind::ParClose |
            TokenKind::Mut
             => Priority::Zero,

            TokenKind::RefType |
            TokenKind::As
             => Priority::Ref,

            TokenKind::ArrayOpen |
            TokenKind::ParOpen |
            TokenKind::ScopeOpen { .. } |
            TokenKind::FunctionParamsOpen |
            TokenKind::ArrayTypeOpen
             => Priority::Max,

        } as i32)
    }

}


#[derive(Debug)]
pub struct Token<'a> {

    pub value: TokenKind<'a>,
    pub token: StringToken<'a>,
    pub unit_path: &'a Path,
    pub priority: i32,

}


impl Token<'_> {

    pub fn new<'a>(value: TokenKind<'a>, source_token: StringToken<'a>, unit_path: &'a Path, base_priority: i32) -> Token<'a> {

        let value_priority = value.type_priority();

        Token {
            value,
            token: source_token,
            unit_path,
            // The priority of the token is the sum of the base priority and the value priority.
            // If the value priority is zero, the token should not be evaluated.
            priority: if value_priority == Priority::Zero as i32 { 0 } else { base_priority + value_priority },
        }
    }

}


impl Display for Token<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.value {
            TokenKind::Op(op) => write!(f, "{}", op),
            TokenKind::Value(v) => write!(f, "{}", v),
            TokenKind::DataType(dt) => write!(f, "DataType({})", dt),
            TokenKind::Fn => write!(f, "fn"),
            TokenKind::Let => write!(f, "let"),
            TokenKind::Arrow => write!(f, "->"),
            TokenKind::Semicolon => write!(f, ";"),
            TokenKind::ArrayOpen => write!(f, "["),
            TokenKind::SquareClose => write!(f, "]"),
            TokenKind::ParOpen => write!(f, "("),
            TokenKind::ParClose => write!(f, ")"),
            TokenKind::ScopeOpen { .. } => write!(f, "{{"),
            TokenKind::ScopeClose => write!(f, "}}"),
            TokenKind::Colon => write!(f, ":"),
            TokenKind::Comma => write!(f, ","),
            TokenKind::Mut => write!(f, "mut"),
            TokenKind::FunctionParamsOpen => write!(f, "FunctionParams"),
            TokenKind::ArrayTypeOpen => write!(f, "ArrayType"),
            TokenKind::RefType => write!(f, "RefType"),
            TokenKind::As => write!(f, "as"),
        }
    }
}

