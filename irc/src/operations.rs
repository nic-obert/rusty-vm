use std::fmt::Display;



#[derive(Debug)]
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

