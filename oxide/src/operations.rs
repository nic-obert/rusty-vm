use std::fmt::Display;

use crate::data_types::DataType;
use crate::data_types::dt_macros::*;


#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Ops {

    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Assign,
    Deref,
    Ref,
    FunctionCallOpen,
    Return,
    Jump,
    Equal,
    NotEqual,
    Greater,
    Less,
    GreaterEqual,
    LessEqual,
    LogicalNot,
    BitwiseNot,
    LogicalAnd,
    LogicalOr,
    BitShiftLeft,
    BitShiftRight,
    BitwiseOr,
    BitwiseAnd,
    BitwiseXor,
    ArrayIndexOpen,

}


impl Ops {

    /// Return whether the operator returns a value or not.
    pub fn returns_a_value(&self) -> bool {
        matches!(self, 
            Ops::Add |
            Ops::Sub |
            Ops::Mul |
            Ops::Div |
            Ops::Mod |
            Ops::Equal |
            Ops::NotEqual |
            Ops::Greater |
            Ops::Less |
            Ops::GreaterEqual |
            Ops::LessEqual |
            Ops::LogicalNot |
            Ops::BitwiseNot |
            Ops::LogicalAnd |
            Ops::LogicalOr |
            Ops::BitShiftLeft |
            Ops::BitShiftRight |
            Ops::BitwiseOr |
            Ops::BitwiseAnd |
            Ops::BitwiseXor |
            Ops::Ref |
            Ops::Deref |
            Ops::FunctionCallOpen |
            Ops::ArrayIndexOpen
        )
    }

    pub fn is_allowed_type(&self, data_type: &DataType, position: u8) -> bool {
        match self {

            Ops::Add |
            Ops::Sub
             => matches!(data_type, numeric_pattern!() | pointer_pattern!()),

            Ops::Greater |
            Ops::Less |
            Ops::GreaterEqual |
            Ops::LessEqual |
            Ops::Mul |
            Ops::Div |
            Ops::Mod
             => matches!(data_type, numeric_pattern!()),

            Ops::Assign => match position {
                0 => matches!(data_type, DataType::Ref(_)),
                1 => true,
                _ => unreachable!("Invalid position for assignment operator")
            },

            Ops::Deref => matches!(data_type, pointer_pattern!()),

            Ops::Equal |
            Ops::NotEqual |
            Ops::Ref
             => true,

            Ops::FunctionCallOpen => matches!(data_type, DataType::Function { .. }),
            Ops::Return => true,
            Ops::Jump => matches!(data_type, unsigned_integer_pattern!()),
            
            
            Ops::LogicalNot |
            Ops::LogicalAnd |
            Ops::LogicalOr
             => matches!(data_type, DataType::Bool),

            Ops::BitwiseNot |
            Ops::BitShiftLeft |
            Ops::BitShiftRight |
            Ops::BitwiseOr |
            Ops::BitwiseAnd |
            Ops::BitwiseXor
             => matches!(data_type, integer_pattern!()),
            
            Ops::ArrayIndexOpen => match position {
                0 => matches!(data_type, DataType::Array { .. }),
                1 => matches!(data_type, unsigned_integer_pattern!()),
                _ => unreachable!("Invalid position for array index operator")
            }
        }
    }

    pub fn allowed_types(&self, position: u8) -> &'static [&'static str] {
        match self {

            Ops::Add |
            Ops::Sub
             => &["numeric", "pointer"],

            Ops::Greater |
            Ops::Less |
            Ops::GreaterEqual |
            Ops::LessEqual |
            Ops::Mul |
            Ops::Div |
            Ops::Mod
             => &["numeric"],

            Ops::Assign => match position {
                0 => &["symbol", "dereference"],
                1 => &["any"],
                _ => unreachable!("Invalid position for assignment operator")
            },

            Ops::Deref => &["pointer"],

            Ops::Return |
            Ops::Equal |
            Ops::NotEqual |
            Ops::Ref
             => &["value"],

            Ops::FunctionCallOpen => &["function"],

            Ops::Jump |
            Ops::ArrayIndexOpen
             => &["unsigned integer"],

            
            Ops::LogicalNot |
            Ops::LogicalAnd |
            Ops::LogicalOr
             => &["bool"],

            Ops::BitwiseNot |
            Ops::BitShiftLeft |
            Ops::BitShiftRight |
            Ops::BitwiseOr |
            Ops::BitwiseAnd |
            Ops::BitwiseXor
             => &["integer"]
        }
    }

}


impl Display for Ops {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            Ops::Add => "+",
            Ops::Sub => "-",
            Ops::Mul => "*",
            Ops::Div => "/",
            Ops::Mod => "%",
            Ops::Assign => "=",
            Ops::Deref => "Deref",
            Ops::Ref => "Ref",
            Ops::FunctionCallOpen => "Call",
            Ops::Return => "return",
            Ops::Jump => "jmp",
            Ops::Equal => "==",
            Ops::NotEqual => "!=",
            Ops::Greater => ">",
            Ops::Less => "<",
            Ops::GreaterEqual => ">=",
            Ops::LessEqual => "<=",
            Ops::LogicalNot => "!",
            Ops::BitwiseNot => "~",
            Ops::LogicalAnd => "&&",
            Ops::LogicalOr => "||",
            Ops::BitShiftLeft => ">>",
            Ops::BitShiftRight => "<<",
            Ops::BitwiseOr => "|",
            Ops::BitwiseAnd => "&",
            Ops::BitwiseXor => "^",
            Ops::ArrayIndexOpen => "Index"
        })
    }
}

