use std::fmt::Display;
use std::path::Path;
use std::ptr::null_mut;

use crate::operations::Ops;
use crate::data_types::DataType;


#[derive(Debug)]
pub enum Value<'a> {

    Literal { value: LiteralValue<'a> },
    Symbol { id: &'a str }

}


impl Display for Value<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Literal { value } => write!(f, "Literal({})", value),
            Value::Symbol { id } => write!(f, "Ref({})", id),
        }
    }
}


#[derive(Debug)]
pub enum Number {

    Int(i64),
    Uint(u64),
    Float(f64)

}


impl Display for Number {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Number::Int(n) => write!(f, "{}", n),
            Number::Uint(n) => write!(f, "{}", n),
            Number::Float(n) => write!(f, "{}", n),
        }
    }
}


#[derive(Debug)]
pub enum LiteralValue<'a> {

    Char (char),
    String (String),

    Array { dt: DataType, items: Vec<Value<'a>> },

    Numeric (Number),

}


impl Display for LiteralValue<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LiteralValue::Char(c) => write!(f, "'{}'", c),
            LiteralValue::String(s) => write!(f, "\"{}\"", s),
            LiteralValue::Array { dt, items } => write!(f, "[{}]: [{:?}]", dt, items),
            LiteralValue::Numeric(n) => write!(f, "{}", n),
        }
    }
}


#[derive(Debug)]
pub struct Function {

    id: String,
    args: Vec<DataType>,
    ret: DataType,

}


#[derive(Debug)]
pub enum TokenKind<'a> {

    Op (Ops),
    Value (Value<'a>),
    DataType (DataType),

    Fn,
    Let,

    Arrow,
    Semicolon,
    Colon,

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


impl TokenKind<'_> {

    pub fn type_priority(&self) -> usize {
        (match self {
            TokenKind::Op(op) => match op {

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

            TokenKind::Fn |
            TokenKind::Let
             => Priority::Least,

            TokenKind::Value(_) |
            TokenKind::DataType(_) |
            TokenKind::Arrow |
            TokenKind::Semicolon |
            TokenKind::Colon
             => Priority::Zero,

            TokenKind::SquareOpen |
            TokenKind::SquareClose |
            TokenKind::ParOpen |
            TokenKind::ParClose |
            TokenKind::ScopeOpen |
            TokenKind::ScopeClose
             => Priority::Max,

        } as usize)
    }

}


pub struct Token<'a> {

    pub value: TokenKind<'a>,
    pub line: usize,
    pub start: usize,
    pub unit_path: &'a Path,
    pub priority: usize,

}


impl Token<'_> {

    pub fn new<'a>(value: TokenKind<'a>, line: usize, start: usize, unit_path: &'a Path, base_priority: usize) -> Token<'a> {

        let value_priority = value.type_priority();

        Token {
            value,
            line,
            start,
            unit_path,
            priority: base_priority + value_priority,
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
            TokenKind::SquareOpen => write!(f, "["),
            TokenKind::SquareClose => write!(f, "]"),
            TokenKind::ParOpen => write!(f, "("),
            TokenKind::ParClose => write!(f, ")"),
            TokenKind::ScopeOpen => write!(f, "{{"),
            TokenKind::ScopeClose => write!(f, "}}"),
            TokenKind::Colon => write!(f, ":"),
        }
    }
}


pub struct TokenList<'a> {

    first: *mut TokenNode<'a>,
    last: *mut TokenNode<'a>

}


impl TokenList<'_> {

    pub fn new<'a>() -> TokenList<'a> {
        TokenList {
            first: null_mut(),
            last: null_mut()
        }
    }


    pub fn append(&mut self, token: Token) {
        let token = unsafe {
            // Cast away lifetimes
            std::mem::transmute::<Token<'_>, Token<'static>>(token)
        };

        if self.last.is_null() {
            self.last = Box::leak(Box::new(TokenNode::new(token, null_mut()))) as *mut TokenNode;
            self.first = self.last;
        } else {
            let new_node = Box::leak(Box::new(TokenNode::new(token, self.last))) as *mut TokenNode;
            unsafe {
                (*self.last).right = new_node;
            }
            self.last = new_node;
        }
    }


    pub fn last(&self) -> Option<&Token> {
        if self.last.is_null() {
            None
        } else {
            unsafe {
                Some(&(*self.last).token)
            }
        }
    }

}


impl<'a> IntoIterator for TokenList<'a> {
    type Item = &'a Token<'a>;

    type IntoIter = TokenIterator<'a>;

    fn into_iter(self) -> Self::IntoIter {
        TokenIterator {
            node: self.first
        }
    }
}


pub struct TokenIterator<'a> {

    node: *mut TokenNode<'a>

}


impl<'a> Iterator for TokenIterator<'a> {
    type Item = &'a Token<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.node.is_null() {
            None
        } else {
            let node = unsafe {
                &mut *self.node
            };
            self.node = node.right;
            Some(&node.token)
        }
    }
}


struct TokenNode<'a> {

    pub token: Token<'a>,

    pub left: *mut TokenNode<'a>,
    pub right: *mut TokenNode<'a>,

    pub children: Vec<TokenNode<'a>>

}


impl TokenNode<'_> {

    pub fn new<'a>(token: Token<'a>, left: *mut TokenNode<'a>) -> TokenNode<'a> {
        TokenNode { 
            token,
            left,
            right: null_mut(),
            children: vec![]
        }
    }

}

