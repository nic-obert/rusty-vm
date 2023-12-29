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
    Call,
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
            Ops::Call
        )
    }

    pub fn is_allowed_type(&self, data_type: &DataType, position: u8) -> bool {
        match self {
            Ops::Add => matches!(data_type, numeric!() | pointer!()),
            Ops::Sub => matches!(data_type, numeric!() | pointer!()),
            Ops::Mul => matches!(data_type, numeric!()),
            Ops::Div => matches!(data_type, numeric!()),
            Ops::Mod => matches!(data_type, numeric!()),
            Ops::Assign => match position {
                0 => matches!(data_type, DataType::Ref(_)),
                1 => true,
                _ => unreachable!("Invalid position for assignment operator")
            },
            Ops::Deref => matches!(data_type, DataType::Ref(_)),
            Ops::Ref => true,
            Ops::Call => matches!(data_type, DataType::Function { .. }),
            Ops::Return => true,
            Ops::Jump => matches!(data_type, unsigned_integer!()),
            Ops::Equal => true,
            Ops::NotEqual => true,
            Ops::Greater => matches!(data_type, numeric!()),
            Ops::Less => matches!(data_type, numeric!()),
            Ops::GreaterEqual => matches!(data_type, numeric!()),
            Ops::LessEqual => matches!(data_type, numeric!()),
            Ops::LogicalNot => matches!(data_type, DataType::Bool),
            Ops::BitwiseNot => matches!(data_type, integer!()),
            Ops::LogicalAnd => matches!(data_type, DataType::Bool),
            Ops::LogicalOr => matches!(data_type, DataType::Bool),
            Ops::BitShiftLeft => matches!(data_type, integer!()),
            Ops::BitShiftRight => matches!(data_type, integer!()),
            Ops::BitwiseOr => matches!(data_type, integer!()),
            Ops::BitwiseAnd => matches!(data_type, integer!()),
            Ops::BitwiseXor => matches!(data_type, integer!())
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
            Ops::Call => "Call",
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
        })
    }
}

