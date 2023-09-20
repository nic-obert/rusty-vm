use std::fmt::Display;

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
pub enum Token<'a> {

    Op (Ops),
    Symbol (&'a str),
    Value (Value),
    DataType (DataType),

    Fn,
    Let,

    Arrow,
    Semicolon,

    GroupOpen,
    GroupClose,

    ParOpen,
    ParClose,

    ScopeOpen,
    ScopeClose,

}


impl Display for Token<'_> {

    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!()
    }

}

