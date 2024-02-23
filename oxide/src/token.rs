use std::fmt::Display;
use std::path::Path;
use std::rc::Rc;

use crate::operations::Ops;
use crate::data_types::{DataType, LiteralValue};
use crate::symbol_table::ScopeDiscriminant;


#[derive(Debug)]
pub struct StringToken<'a> {

    pub string: &'a str,
    pub unit_path: &'a Path,
    pub line_index: usize,
    pub column: usize,

}

impl Display for StringToken<'_> {

    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.string)
    }

}

impl StringToken<'_> {

    /// Returns the line number of the token, starting from 1
    /// 
    /// This is used to display the line number in the error message.
    #[inline]
    pub const fn line_number(&self) -> usize {
        self.line_index + 1
    }

    /// Returns the line index of the token, starting from 0. 
    /// 
    /// This is used to index the line in the source code.
    #[inline]
    pub const fn line_index(&self) -> usize {
        self.line_index
    }

}


#[derive(Debug)]
pub enum Value<'a> {

    Literal { value: LiteralValue },
    Symbol { name: &'a str, scope_discriminant: ScopeDiscriminant }

}


impl Display for Value<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Literal { value } => write!(f, "Literal({})", value),
            Value::Symbol { name, .. } => write!(f, "Ref({})", name),
        }
    }
}


#[derive(Debug)]
pub enum TokenKind<'a> {

    Op (Ops),
    Value (Value<'a>),
    DataType (Rc<DataType>),

    RefType,

    Const,
    Static,
    TypeDef,
    Fn,
    Let,
    As,
    If,
    Else,
    While,
    Loop,
    DoWhile,

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


#[allow(non_camel_case_types)]
pub enum Priority {

    Zero = 0,
    Least_Assignment_FlowBreak,

    Declaration,

    ControlFlow,

    Add_Sub,
    Mul_Div_Mod,

    LogicalOr,
    LogicalAnd,
    BitwiseOr,
    BitwiseXor,
    BitwiseAnd,
    Equality,
    Comparison,
    Bitshift,
    Not,

    Ref_Cast,
    TakeRef,

    /// Delimiters have the maximum priority.
    Delimiter,

    /// Special priority used to pre-process certain tokens who might have a non-complete value (symbols and their scope discriminant)
    PreProcess

}


impl TokenKind<'_> {

    pub fn literal_value(&self) -> Option<&LiteralValue> {
        if let TokenKind::Value(Value::Literal { value }) = self { Some(value) } else { None }
    }


    pub fn type_priority(&self) -> TokenPriority {
        (match self {
            TokenKind::Op(op) => match op {

                Ops::Add |
                Ops::Sub
                 => Priority::Add_Sub,

                Ops::Mul |
                Ops::Div |
                Ops::Mod
                 => Priority::Mul_Div_Mod,

                Ops::Return |
                Ops::Assign |
                Ops::Break | 
                Ops::Continue
                 => Priority::Least_Assignment_FlowBreak,

                Ops::Deref { .. } |
                Ops::Ref { .. }
                 => Priority::TakeRef,

                Ops::FunctionCallOpen |
                Ops::ArrayIndexOpen
                 => Priority::Delimiter,
                
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

            // Const has to be the top-level node in constant declaration
            // const a: B = 1 + 2; --> const a: B = +(1, 2) --> const(a, B, +(1, 2))
            TokenKind::Const |
            TokenKind::Static |
            // Same story with typedef
            TokenKind::TypeDef
             => Priority::Least_Assignment_FlowBreak,

            TokenKind::Fn |
            TokenKind::Let 
                => Priority::Declaration,

            TokenKind::Value(Value::Literal { .. }) |
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

            TokenKind::RefType => Priority::TakeRef,
            TokenKind::As => Priority::Ref_Cast,

            TokenKind::ArrayOpen |
            TokenKind::ParOpen |
            TokenKind::ScopeOpen { .. } |
            TokenKind::FunctionParamsOpen |
            TokenKind::ArrayTypeOpen
             => Priority::Delimiter,
            
            TokenKind::If |
            TokenKind::Else |
            TokenKind::While |
            TokenKind::Loop |
            TokenKind::DoWhile
             => Priority::ControlFlow,

            TokenKind::Value(Value::Symbol { .. }) => Priority::PreProcess,

        } as TokenPriority)
    }

}


pub type TokenPriority = i16;


#[derive(Debug)]
pub struct Token<'a> {

    pub value: TokenKind<'a>,
    pub token: Rc<StringToken<'a>>,
    pub priority: TokenPriority,

}


impl Token<'_> {

    pub fn new<'a>(value: TokenKind<'a>, source_token: StringToken<'a>, base_priority: TokenPriority) -> Token<'a> {

        let value_priority = value.type_priority();

        Token {
            value,
            token: Rc::new(source_token),
            // The priority of the token is the sum of the base priority and the value priority.
            // If the value priority is zero, the token should not be evaluated.
            priority: if value_priority == Priority::Zero as TokenPriority { 0 } else { base_priority + value_priority },
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
            TokenKind::If => write!(f, "if"),
            TokenKind::Else => write!(f, "else"),
            TokenKind::While => write!(f, "while"),
            TokenKind::Loop => write!(f, "loop"),
            TokenKind::Const => write!(f, "const"),
            TokenKind::TypeDef => write!(f, "typedef"),
            TokenKind::DoWhile => write!(f, "do-while"),
            TokenKind::Static => write!(f, "static"),
        }
    }
}

