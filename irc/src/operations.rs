use std::fmt::Display;


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

    // pub const fn arg_types(&self) -> &'static [DataType] {
    //     match self {
    //         Ops::Add |
    //         Ops::Sub |
    //         Ops::Mul |
    //         Ops::Div |
    //         Ops::Mod => vec![integer_types!(), float_types!()].concat().as_slice(),

    //         Ops::Equal |
    //         Ops::NotEqual |
    //         Ops::Greater |
    //         Ops::Less |
    //         Ops::GreaterEqual |
    //         Ops::LessEqual => &[DataType::I8, DataType::I16, DataType::I32, DataType::I64, DataType::U8, DataType::U16, DataType::U32, DataType::U64, DataType::F32, DataType::F64],
    //         Ops::Assign => todo!(),
    //         Ops::Deref => todo!(),
    //         Ops::Ref => todo!(),
    //         Ops::Call => todo!(),
    //         Ops::Return => todo!(),
    //         Ops::Jump => todo!(),
    //         Ops::LogicalNot => todo!(),
    //         Ops::BitwiseNot => todo!(),
    //         Ops::LogicalAnd => todo!(),
    //         Ops::LogicalOr => todo!(),
    //         Ops::BitShiftLeft => todo!(),
    //         Ops::BitShiftRight => todo!(),
    //         Ops::BitwiseOr => todo!(),
    //         Ops::BitwiseAnd => todo!(),
    //         Ops::BitwiseXor => todo!(),
    //     }
    // }

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

