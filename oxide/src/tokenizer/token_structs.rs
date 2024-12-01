use std::{path::Path, ptr::NonNull, rc::Rc};

use crate::{lang::{DataType, LiteralValue}, symbol_table::SymbolID};


#[derive(Debug)]
pub struct SourceToken<'a> {
    line_index: usize,
    pub column_index: usize,
    /// The actual string from the source code.
    pub string: &'a str,
    /// The module this token was found in.
    pub module_path: &'a Path,
}

impl<'a> SourceToken<'a> {

    pub fn new(line_index: usize, column_index: usixe, string: &'a str, module_path: &'a Path) -> SourceToken<'a> {
        Self {
            line_index,
            column_index,
            string,
            module_path
        }
    }

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


pub type TokenPriority = i16;


#[allow(non_camel_case_types)]
pub enum Priority {

    Zero = 0,
    Least_Assignment_FlowBreak,

    Declaration,

    ControlFlow,

    Add_Sub,
    Mul_Div_Mod,

    Or,
    Xor,
    And,
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


#[derive(Debug)]
pub struct Token<'a> {
    source: Rc<SourceToken<'a>>,
    priority: TokenPriority,
    value: TokenValue<'a>,

    left: Option<NonNull<Token<'a>>>,
    right: Option<NonNull<Token<'a>>>,
}


pub type BinaryArgs<'a> = Option<(Box<Token<'a>>, Box<Token<'a>>)>;
pub type UnaryArgs<'a> = Option<Box<Token<'a>>>;


#[derive(Debug)]
pub enum TokenValue<'a> {

    Add (BinaryArgs<'a>),
    Sub (BinaryArgs<'a>),
    Mul (BinaryArgs<'a>),
    Div (BinaryArgs<'a>),
    Mod (BinaryArgs<'a>),
    Assign (BinaryArgs<'a>),
    Deref (Option<Box<Token<'a>>>),
    Ref (Option<Box<Token<'a>>>),
    FunctionCallOpen (Option<Box<[Token<'a>]>>),
    Return (Option<Box<Token<'a>>>),
    Equal (BinaryArgs<'a>),
    NotEqual (BinaryArgs<'a>),
    Greater (BinaryArgs<'a>),
    Less (BinaryArgs<'a>),
    GreaterEqual (BinaryArgs<'a>),
    LessEqual (BinaryArgs<'a>),
    Not (Option<Box<Token<'a>>>),
    And (BinaryArgs<'a>),
    Or (BinaryArgs<'a>),
    Xor (BinaryArgs<'a>),
    BitShiftLeft (BinaryArgs<'a>),
    BitShiftRight (BinaryArgs<'a>),
    ArrayIndexOpen (BinaryArgs<'a>),
    Break,
    Continue,

    Value (Value<'a>),
    DataType (Rc<DataType>),

    RefType,

    Const,
    Static,
    TypeDef,
    Fn,
    Let,
    As (BinaryArgs<'a>),
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


impl TokenValue<'_> {

    // pub fn literal_value(&self) -> Option<&LiteralValue> {
    //     if let TokenKind::Value(Value::Literal { value }) = self { Some(value) } else { None }
    // }


    pub fn type_priority(&self) -> TokenPriority {
        (match self {

            TokenValue::Add (_) |
            TokenValue::Sub
                => Priority::Add_Sub,

            TokenValue::Mul |
            TokenValue::Div |
            TokenValue::Mod
                => Priority::Mul_Div_Mod,

            TokenValue::Return |
            TokenValue::Assign |
            TokenValue::Break |
            TokenValue::Continue
                => Priority::Least_Assignment_FlowBreak,

            TokenValue::Deref |
            TokenValue::Ref
                => Priority::TakeRef,

            TokenValue::FunctionCallOpen |
            TokenValue::ArrayIndexOpen
                => Priority::Delimiter,

            TokenValue::Equal |
            TokenValue::NotEqual
                => Priority::Equality,

            TokenValue::Greater |
            TokenValue::Less |
            TokenValue::GreaterEqual |
            TokenValue::LessEqual
                => Priority::Comparison,

            TokenValue::BitwiseNot |
            TokenValue::LogicalNot
                => Priority::Not,

            TokenValue::LogicalAnd => Priority::And,
            TokenValue::LogicalOr => Priority::Or,

            TokenValue::BitShiftRight |
            TokenValue::BitShiftLeft
                => Priority::Bitshift,

            TokenValue::BitwiseOr => Priority::BitwiseOr,
            TokenValue::BitwiseAnd => Priority::BitwiseAnd,
            TokenValue::BitwiseXor => Priority::BitwiseXor,

            // Const has to be the top-level node in constant declaration
            // const a: B = 1 + 2; --> const a: B = +(1, 2) --> const(a, B, +(1, 2))
            TokenValue::Const |
            TokenValue::Static |
            // Same story with typedef
            TokenValue::TypeDef
                => Priority::Least_Assignment_FlowBreak,

            TokenValue::Fn |
            TokenValue::Let
                => Priority::Declaration,

            TokenValue::Arrow |
            TokenValue::Semicolon |
            TokenValue::Colon |
            TokenValue::Comma |
            TokenValue::ScopeClose |
            TokenValue::SquareClose |
            TokenValue::ParClose |
            TokenValue::Mut
                => Priority::Zero,

            TokenValue::RefType => Priority::TakeRef,
            TokenValue::As => Priority::Ref_Cast,

            TokenValue::ArrayOpen |
            TokenValue::ParOpen |
            TokenValue::ScopeOpen { .. } |
            TokenValue::FunctionParamsOpen |
            TokenValue::ArrayTypeOpen
                => Priority::Delimiter,

            TokenValue::If |
            TokenValue::Else |
            TokenValue::While |
            TokenValue::Loop |
            TokenValue::DoWhile
                => Priority::ControlFlow,

            TokenValue::DataType(_) |
            TokenValue::Value(_)
                => Priority::PreProcess,

        } as TokenPriority)
    }

}


#[derive(Debug)]
pub enum Value<'a> {

    Literal { value: LiteralValue },
    Symbol { id: SymbolID<'a> }

}
