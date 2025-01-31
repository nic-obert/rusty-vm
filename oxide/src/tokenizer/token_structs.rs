use std::fmt;
use std::rc::Rc;
use std::ptr::NonNull;

use crate::lang::{DataType, DataTypeName, LiteralValue};


#[derive(Debug)]
pub struct SourceToken<'a> {
    line_index: usize,
    pub column_index: usize,
    /// The actual string from the source code.
    pub string: &'a str,
}

impl<'a> SourceToken<'a> {

    pub fn new(line_index: usize, column_index: usize, string: &'a str) -> SourceToken<'a> {
        Self {
            line_index,
            column_index,
            string,
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

/// Operator prioriy is inspired by C
#[allow(non_camel_case_types)]
pub enum Priority {

    Zero = 0,
    Least_Assignment,

    Declaration,

    ControlFlow,

    LogicalOr,
    LogicalAnd,

    BitOr,
    BitXor,
    BitAnd,

    Equality,
    Comparison,

    Bitshift,

    Add_Sub,
    Mul_Div_Mod,

    Not_TakeRef_Deref_TypeCast_UnaryMinus,

    /// Delimiters have the maximum priority.
    Delimiter,

    /// Special priority used to pre-process certain tokens who might have a non-complete value (symbols and their scope discriminant)
    /// TODO: Use/import statements?
    PreProcess

}


pub struct TokenList<'a> {
    first: Option<NonNull<Token<'a>>>,
    last: Option<NonNull<Token<'a>>>
}

impl<'a> TokenList<'a> {

    pub fn new() -> Self {
        Self {
            first: None,
            last: None
        }
    }


    pub fn push(&mut self, source: Rc<SourceToken<'a>>, value: TokenValue<'a>, base_priority: TokenPriority) {

        let new_node = NonNull::new(Box::leak(Box::new(Token {
            source,
            priority: if value.type_priority() == 0 { value.type_priority() + base_priority } else { 0 },
            value,
            left: self.last,
            right: None
        }))).unwrap();

        if let Some(mut last) = self.last {
            unsafe {
                last.as_mut().right = Some(new_node);
            }
        } else {
            self.first = Some(new_node);
        }

        self.last = Some(new_node);
    }


    pub fn iter_lex_tokens_rev(&self) -> impl Iterator<Item = &Token<'a>> {
        gen {
            let mut next = self.last;
            while let Some(cursor) = next {
                let token = unsafe { cursor.as_ref() };
                next = token.left;
                yield token;
            }
        }
    }

}

impl fmt::Debug for TokenList<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut next = self.first;
        while let Some(cursor) = next {
            let token = unsafe { cursor.as_ref() };
            next = token.right;
            writeln!(f, "{:?}", token)?;
        }
        Ok(())
    }
}


#[derive(Debug)]
pub struct Token<'a> {
    source: Rc<SourceToken<'a>>,
    priority: TokenPriority,
    pub(super) value: TokenValue<'a>,

    left: Option<NonNull<Token<'a>>>,
    right: Option<NonNull<Token<'a>>>,
}


#[derive(Debug)]
pub enum TokenValue<'a> {

    Add,
    Sub,
    UnaryMinus,
    Mul,
    Div,
    Mod,
    Assign,
    AddAssign,
    SubAssign,
    MulAssign,
    DivAssign,
    ModAssign,
    Deref,
    TakeRef,
    FunctionCallOpen,
    Return,
    Equal,
    NotEqual,
    Greater,
    Less,
    GreaterEqual,
    LessEqual,
    LogicalNot,
    BitNot,
    LogicalAnd,
    BitAnd,
    LogicalOr,
    BitOr,
    BitXor,
    BitShiftLeft,
    BitShiftRight,
    ArrayIndexOpen,
    Break,
    Continue,

    Value (Value<'a>),

    // RefType,
    BuiltinType (DataType),

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


    pub const fn type_priority(&self) -> TokenPriority {
        (match self {

            TokenValue::Add |
            TokenValue::Sub
                => Priority::Add_Sub,

            TokenValue::Mul |
            TokenValue::Div |
            TokenValue::Mod
                => Priority::Mul_Div_Mod,

            TokenValue::Assign |
            TokenValue::AddAssign |
            TokenValue::SubAssign |
            TokenValue::MulAssign |
            TokenValue::DivAssign |
            TokenValue::ModAssign
                => Priority::Least_Assignment,

            TokenValue::Return |
            TokenValue::Break |
            TokenValue::Continue |
            TokenValue::If |
            TokenValue::Else |
            TokenValue::While |
            TokenValue::Loop |
            TokenValue::DoWhile
                => Priority::ControlFlow,

            TokenValue::Equal |
            TokenValue::NotEqual
                => Priority::Equality,

            TokenValue::Greater |
            TokenValue::Less |
            TokenValue::GreaterEqual |
            TokenValue::LessEqual
                => Priority::Comparison,

            TokenValue::LogicalAnd => Priority::LogicalAnd,
            TokenValue::LogicalOr => Priority::LogicalOr,

            TokenValue::BitShiftRight |
            TokenValue::BitShiftLeft
                => Priority::Bitshift,

            TokenValue::BitOr => Priority::BitOr,
            TokenValue::BitAnd => Priority::BitAnd,
            TokenValue::BitXor => Priority::BitXor,

            // Const has to be the top-level node in constant declaration
            // const a: B = 1 + 2; --> const a: B = +(1, 2) --> const(a, B, +(1, 2))
            TokenValue::Const |
            TokenValue::Static |
            // Same story with typedef
            TokenValue::TypeDef |
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

            // TokenValue::RefType |
            TokenValue::As |
            TokenValue::BitNot |
            TokenValue::LogicalNot |
            TokenValue::Deref |
            TokenValue::TakeRef |
            TokenValue::UnaryMinus
                => Priority::Not_TakeRef_Deref_TypeCast_UnaryMinus,

            TokenValue::FunctionParamsOpen |
            TokenValue::FunctionCallOpen |
            TokenValue::ArrayIndexOpen |
            TokenValue::ArrayOpen |
            TokenValue::ParOpen |
            TokenValue::ScopeOpen
                => Priority::Delimiter,

            TokenValue::BuiltinType(_) |
            TokenValue::Value(_)
                => Priority::PreProcess,

        } as TokenPriority)
    }

}


#[derive(Debug)]
pub enum Value<'a> {

    Literal (LiteralValue<'a>),
    Symbol (&'a str)

}
