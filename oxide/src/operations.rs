use std::fmt::Display;

use crate::data_types::{DataType, LiteralValue, Number};
use crate::data_types::dt_macros::*;
use crate::match_unreachable;


pub mod op_macros {

    #[macro_export]
    macro_rules! binary_operators {
        () => {
            Ops::Add | Ops::Sub | Ops::Mul | Ops::Div | Ops::Mod | Ops::Equal | Ops::NotEqual | Ops::Greater | Ops::Less | Ops::GreaterEqual | Ops::LessEqual | Ops::LogicalAnd | Ops::LogicalOr | Ops::BitShiftLeft | Ops::BitShiftRight | Ops::BitwiseOr | Ops::BitwiseAnd | Ops::BitwiseXor | Ops::ArrayIndexOpen | Ops::Assign
        };
    }

    #[macro_export]
    macro_rules! unary_operators {
        () => {
            Ops::Deref | Ops::Ref | Ops::LogicalNot | Ops::BitwiseNot
        };
    }

}


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
    pub const fn returns_a_value(&self) -> bool {
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
             => matches!(data_type, numeric_pattern!()),

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
                1 => !matches!(data_type, DataType::Void), // Disallow setting values to void
                _ => unreachable!("Invalid position for assignment operator")
            },

            Ops::Deref => matches!(data_type, pointer_pattern!()),

            Ops::Equal |
            Ops::NotEqual |
            Ops::Ref
             => true,

            Ops::FunctionCallOpen => matches!(data_type, DataType::Function { .. }),
            Ops::Return => true,            
            
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
             => &["number"],

            Ops::Greater |
            Ops::Less |
            Ops::GreaterEqual |
            Ops::LessEqual |
            Ops::Mul |
            Ops::Div |
            Ops::Mod
             => &["number"],

            Ops::Assign => match position {
                0 => &["symbol", "dereference"],
                1 => &["value"],
                _ => unreachable!("Invalid position for assignment operator")
            },

            Ops::Deref => &["pointer"],

            Ops::Return |
            Ops::Equal |
            Ops::NotEqual |
            Ops::Ref
             => &["value"],

            Ops::FunctionCallOpen => &["function"],

            Ops::ArrayIndexOpen => match position {
                0 => &["array"],
                1 => &["unsigned integer"],
                _ => unreachable!("Invalid position for array index operator")
            }

            
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


    pub const fn is_allowed_at_compile_time(&self) -> bool {
        match self {
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
            Ops::ArrayIndexOpen |
            Ops::Assign  // Assign is allowed only when initializing an immutable symbol with a constant expression
            => true,
            
            Ops::Deref |
            Ops::Ref |
            Ops::FunctionCallOpen |
            Ops::Return
             => false
        }
    }


    pub fn execute(self, args: &[LiteralValue]) -> Result<LiteralValue, &'static str> {

        assert!(self.is_allowed_at_compile_time(), "Operator {:?} cannot be executed at compile-time", self);

        match self {
            Ops::Add => {
                let (n1, n2) = match_unreachable!([LiteralValue::Numeric(n1), LiteralValue::Numeric(n2)] = args, (n1, n2));
                Ok(LiteralValue::Numeric(n1.add(n2)))
            },
            Ops::Sub => {
                let (n1, n2) = match_unreachable!([LiteralValue::Numeric(n1), LiteralValue::Numeric(n2)] = args, (n1, n2));
                Ok(LiteralValue::Numeric(n1.sub(n2)))
            },
            Ops::Mul => {
                let (n1, n2) = match_unreachable!([LiteralValue::Numeric(n1), LiteralValue::Numeric(n2)] = args, (n1, n2));
                Ok(LiteralValue::Numeric(n1.mul(n2)))
            },
            Ops::Div => {
                let (n1, n2) = match_unreachable!([LiteralValue::Numeric(n1), LiteralValue::Numeric(n2)] = args, (n1, n2));
                match n1.div(n2) {
                    Ok(n) => Ok(LiteralValue::Numeric(n)),
                    Err(()) => Err("Division by zero")
                }
            },
            Ops::Mod => {
                let (n1, n2) = match_unreachable!([LiteralValue::Numeric(n1), LiteralValue::Numeric(n2)] = args, (n1, n2));
                match n1.modulo(n2) {
                    Ok(n) => Ok(LiteralValue::Numeric(n)),
                    Err(()) => Err("Division by zero")
                }
            },
            Ops::Equal => {
                let (a, b) = match_unreachable!([a, b] = args, (a, b));
                Ok(LiteralValue::Bool(a.equal(b)))
            },
            Ops::NotEqual => {
                let (a, b) = match_unreachable!([a, b] = args, (a, b));
                Ok(LiteralValue::Bool(!a.equal(b)))
            },
            Ops::Greater => {
                let (n1, n2) = match_unreachable!([LiteralValue::Numeric(n1), LiteralValue::Numeric(n2)] = args, (n1, n2));
                Ok(LiteralValue::Bool(n1.greater(n2)))
            },
            Ops::Less => {
                let (n1, n2) = match_unreachable!([LiteralValue::Numeric(n1), LiteralValue::Numeric(n2)] = args, (n1, n2));
                Ok(LiteralValue::Bool(n1.less(n2)))
            },
            Ops::GreaterEqual => {
                let (n1, n2) = match_unreachable!([LiteralValue::Numeric(n1), LiteralValue::Numeric(n2)] = args, (n1, n2));
                Ok(LiteralValue::Bool(n1.greater_equal(n2)))
            },
            Ops::LessEqual => {
                let (n1, n2) = match_unreachable!([LiteralValue::Numeric(n1), LiteralValue::Numeric(n2)] = args, (n1, n2));
                Ok(LiteralValue::Bool(n1.less_equal(n2)))
            },
            Ops::LogicalNot => {
                let b = match_unreachable!([LiteralValue::Bool(b)] = args, *b);
                Ok(LiteralValue::Bool(!b))
            },
            Ops::BitwiseNot => {
                let n = match_unreachable!([LiteralValue::Numeric(n)] = args, n);
                Ok(LiteralValue::Numeric(n.bitwise_not()))
            },
            Ops::LogicalAnd => {
                let (b1, b2) = match_unreachable!([LiteralValue::Bool(b1), LiteralValue::Bool(b2)] = args, (*b1, *b2));
                Ok(LiteralValue::Bool(b1 && b2))
            },
            Ops::LogicalOr => {
                let (b1, b2) = match_unreachable!([LiteralValue::Bool(b1), LiteralValue::Bool(b2)] = args, (*b1, *b2));
                Ok(LiteralValue::Bool(b1 || b2))
            },
            Ops::BitShiftLeft => {
                let (n1, n2) = match_unreachable!([LiteralValue::Numeric(n1), LiteralValue::Numeric(n2)] = args, (n1, n2));
                Ok(LiteralValue::Numeric(n1.bitshift_left(n2)))
            },
            Ops::BitShiftRight => {
                let (n1, n2) = match_unreachable!([LiteralValue::Numeric(n1), LiteralValue::Numeric(n2)] = args, (n1, n2));
                Ok(LiteralValue::Numeric(n1.bitshift_right(n2)))
            },
            Ops::BitwiseOr => {
                let (n1, n2) = match_unreachable!([LiteralValue::Numeric(n1), LiteralValue::Numeric(n2)] = args, (n1, n2));
                Ok(LiteralValue::Numeric(n1.bitwise_or(n2)))
            },
            Ops::BitwiseAnd => {
                let (n1, n2) = match_unreachable!([LiteralValue::Numeric(n1), LiteralValue::Numeric(n2)] = args, (n1, n2));
                Ok(LiteralValue::Numeric(n1.bitwise_and(n2)))
            },
            Ops::BitwiseXor => {
                let (n1, n2) = match_unreachable!([LiteralValue::Numeric(n1), LiteralValue::Numeric(n2)] = args, (n1, n2));
                Ok(LiteralValue::Numeric(n1.bitwise_xor(n2)))
            },
            Ops::ArrayIndexOpen => {
                let (items, index) = match_unreachable!([LiteralValue::Array { items, .. }, LiteralValue::Numeric(Number::Uint(index))] = args, (items, *index));
                
                if index as usize >= items.len() {
                    return Err("Index out of bounds");
                }

                Ok(items[index as usize].clone())
            },

            _ => unreachable!("Operator {:?} cannot be executed at compile-time. The previous assertion couldn't catch this, so this function and is_allowed_at_compile_time don't match.", self)
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



