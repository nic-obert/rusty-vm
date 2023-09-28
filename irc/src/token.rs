use std::fmt::Display;
use std::path::Path;

use crate::operations::Ops;
use crate::data_types::DataType;


#[derive(Debug)]
pub enum Value {

    Literal { value: LiteralValue },
    Symbol { id: String }

}


#[derive(Debug)]
pub enum Number {

    Int(i64),
    Uint(u64),
    Float(f64)

}


#[derive(Debug)]
pub enum LiteralValue {

    Char (char),
    String (String),

    Array { dt: DataType, items: Vec<Value> },

    Numeric (Number),

}


#[derive(Debug)]
pub struct Function {

    id: String,
    args: Vec<DataType>,
    ret: DataType,

}


#[derive(Debug)]
pub enum TokenValue<'a> {

    Op (Ops),
    Symbol (&'a str),
    Value (Value),
    DataType (DataType),

    Fn,
    Let,

    Arrow,
    Semicolon,

    SquareOpen,
    SquareClose,

    ParOpen,
    ParClose,

    ScopeOpen,
    ScopeClose,

}


pub enum Priority {

    Zero = 0,
    Least,

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


impl TokenValue<'_> {

    pub fn type_priority(&self) -> usize {
        (match self {
            TokenValue::Op(op) => match op {

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

            TokenValue::Fn |
            TokenValue::Let
             => Priority::Least,

            TokenValue::Symbol(_) |
            TokenValue::Value(_) |
            TokenValue::DataType(_) |
            TokenValue::Arrow |
            TokenValue::Semicolon
             => Priority::Zero,

            TokenValue::SquareOpen |
            TokenValue::SquareClose |
            TokenValue::ParOpen |
            TokenValue::ParClose |
            TokenValue::ScopeOpen |
            TokenValue::ScopeClose
             => Priority::Max,

        } as usize)
    }

}


impl Display for TokenValue<'_> {

    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!()
    }

}


pub struct Token<'a> {

    pub value: TokenValue<'a>,
    pub line: usize,
    pub start: usize,
    pub unit_path: &'a Path,
    pub priority: usize,

}


impl Token<'_> {

    pub fn new<'a>(value: TokenValue<'a>, line: usize, start: usize, unit_path: &'a Path, base_priority: usize) -> Token<'a> {

        let value_priority = value.type_priority();

        Token {
            value,
            line,
            start,
            unit_path,
            priority: base_priority + value_priority
        }
    }

}

