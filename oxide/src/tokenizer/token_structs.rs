use std::fmt::Display;
use std::path::Path;
use std::rc::Rc;
use std::ptr;

use crate::lang::data_types::{DataType, LiteralValue};
use crate::symbol_table::ScopeDiscriminant;
use crate::ast::{SyntaxNode, UnparsedScopeBlock};
use crate::match_unreachable;
use crate::utils::write_indent;


#[derive(Debug)]
pub enum TokenParsingNodeValue<'a> {
    LexToken(Token<'a>),
    SyntaxToken(SyntaxNode<'a>),
    RawScope { inner_tokens: TokenParsingList<'a>, token: Token<'a> },
    UnparsedScope { statements: UnparsedScopeBlock<'a>, token: Token<'a> },

    /// Placeholder value used to replace a node's value when it is extracted to satisfy the Rust's initialization rules.
    /// This enum variant should never be used in the code, nor be read.
    Placeholder
}


impl<'a> TokenParsingNodeValue<'a> {

    pub fn assume_lex_token(self) -> Token<'a> {
        match self {
            TokenParsingNodeValue::LexToken(token) => token,
            _ => unreachable!()
        }
    }


    pub fn assume_raw_scope(self) -> (TokenParsingList<'a>, Token<'a>) {
        match self {
            TokenParsingNodeValue::RawScope { inner_tokens, token } => (inner_tokens, token),
            _ => unreachable!()
        }
    }


    pub fn source_token(&self) -> &SourceToken {
        match self {
            TokenParsingNodeValue::LexToken(token) => &token.source_token,
            TokenParsingNodeValue::SyntaxToken(node) => &node.token,
            TokenParsingNodeValue::RawScope { token, .. } => &token.source_token,
            TokenParsingNodeValue::UnparsedScope { token, .. } => &token.source_token,
            TokenParsingNodeValue::Placeholder => unreachable!()
        }
    }

}


#[derive(Debug)]
pub struct TokenParsingNode<'a> {
    pub value: TokenParsingNodeValue<'a>,
    left: *mut TokenParsingNode<'a>,
    right: *mut TokenParsingNode<'a>,
}


impl<'a> TokenParsingNode<'a> {

    /// Return the node's value and replaces it with a placeholder value.
    pub fn extract_value(&mut self) -> TokenParsingNodeValue<'a> {
        std::mem::replace(&mut self.value, TokenParsingNodeValue::Placeholder)
    }


    pub fn source_token(&self) -> &SourceToken {
        self.value.source_token()
    }


    pub unsafe fn left(&self) -> *mut TokenParsingNode<'a> {
        self.left
    }


    pub unsafe fn right(&self) -> *mut TokenParsingNode<'a> {
        self.right
    }
    

    pub unsafe fn assume_lex_token(&self) -> &Token {
        match &self.value {
            TokenParsingNodeValue::LexToken(token) => token,
            _ => unreachable!("Argument was assumed to be a LexToken, but {:#?} was found", self.value)
        }
    }


    pub fn lex_token_extract_value(self) -> Result<Token<'a>, Self> {
        match self.value {
            TokenParsingNodeValue::LexToken(token) => Ok(token),
            _ => Err(self)
        }
    }


    pub unsafe fn assume_lex_token_extract_value(self) -> Token<'a> {
        match self.value {
            TokenParsingNodeValue::LexToken(token) => token,
            _ => unreachable!("Argument was assumed to be a LexToken, but {:#?} was found", self.value)
        }
    }


    pub fn syntax_node_extract_value(self) -> Result<SyntaxNode<'a>, Self> {
        match self.value {
            TokenParsingNodeValue::SyntaxToken(node) => Ok(node),
            _ => Err(self)
        }
    }


    pub unsafe fn assume_syntax_node_extract_value(self) -> SyntaxNode<'a> {
        match self.value {
            TokenParsingNodeValue::SyntaxToken(node) => node,
            _ => unreachable!("Argument was assumed to be a SyntaxToken, but {:#?} was found", self.value)
        }
    }
 
}


#[derive(Debug)]
pub struct TokenParsingList<'a> {
    head: *mut TokenParsingNode<'a>,
    tail: *mut TokenParsingNode<'a>,
}

impl<'a> TokenParsingList<'a> {

    pub fn is_empty(&self) -> bool {
        self.head.is_null()
    }   


    pub fn has_one_item(&self) -> bool {
        !self.head.is_null() && self.head == self.tail
    }


    pub unsafe fn head(&self) -> *mut TokenParsingNode<'a> {
        self.head
    }


    // pub unsafe fn tail(&self) -> *mut TokenParsingNode<'a> {
    //     self.tail
    // }


    pub fn new() -> TokenParsingList<'a> {
        TokenParsingList {
            head: std::ptr::null_mut(),
            tail: std::ptr::null_mut(),
        }
    }


    pub fn push_token(&mut self, value: Token<'a>) {
        let node = Box::new(TokenParsingNode {
            value: TokenParsingNodeValue::LexToken(value),
            left: std::ptr::null_mut(),
            right: std::ptr::null_mut(),
        });

        let node = Box::into_raw(node);

        if self.head.is_null() {
            self.head = node;
        } else {
            unsafe {
                (*self.tail).right = node;
                (*node).left = self.tail;
            }
        }

        self.tail = node;
    }


    pub fn extract_first(&mut self) -> Option<Box<TokenParsingNode<'a>>> {
        unsafe { self.extract_node(self.head) }
    }


    pub unsafe fn extract_node(&mut self, node: *mut TokenParsingNode<'a>) -> Option<Box<TokenParsingNode<'a>>> {

        if node.is_null() {
            return None;
        }

        let node_ref = node.as_mut().unwrap();

        if node == self.head {
            self.head = node_ref.right;
        } else {
            (*node_ref.left).right = node_ref.right;
        }

        if node == self.tail {
            self.tail = node_ref.left;
        } else {
            (*node_ref.right).left = node_ref.left;
        }

        node_ref.left = std::ptr::null_mut();
        node_ref.right = std::ptr::null_mut();

        Some(Box::from_raw(node))
    }


    pub fn drop_last(&mut self) {
        if self.tail.is_null() {
            return;
        }

        unsafe {
            let old_tail = self.tail;
            self.tail = (*old_tail).left;
            if self.tail.is_null() {
                self.head = std::ptr::null_mut();
            } else {
                (*self.tail).right = std::ptr::null_mut();
            }

            drop(Box::from_raw(old_tail));
        }
    }


    pub unsafe fn extract_slice(&mut self, start: *mut TokenParsingNode<'a>, end: *mut TokenParsingNode<'a>) -> TokenParsingList<'a> {

        // Remove the slice from the list
        if (*start).left.is_null() {
            // start is the first node
            self.head = (*end).right;
        } else {
            // start is not the first node
            (*(*start).left).right = (*end).right;
        }

        if (*end).right.is_null() {
            // end is the last node
            self.tail = (*start).left;
        } else {
            // end is not the last node
            (*(*end).right).left = (*start).left;
        }

        (*start).left = ptr::null_mut();
        (*end).right = ptr::null_mut();

        TokenParsingList {
            head: start,
            tail: end,
        }
    }

    
    /// Returns a reference to the last element of the list, if present.
    /// Assumes the last element is a LexToken. This function should only be called when the list is known to contain only LexTokens.
    pub fn last_token(&self) -> Option<&Token<'a>> {
        unsafe { self.tail.as_ref() }.map(
            |tail|
                match_unreachable!(TokenParsingNodeValue::LexToken(token) = &tail.value, token)
        )
    }


    pub fn iter(&self) -> TokenParsingListIterator<'a> {
        TokenParsingListIterator {
            next: self.head,
            next_back: self.tail,
        }
    }


    pub fn iter_mut(&mut self) -> TokenParsingListIteratorMut<'a> {
        TokenParsingListIteratorMut {
            current: self.head,
        }
    }


    /// Iterate over the list of lexical tokens.
    /// Assume that the list only contains lexical tokens.
    pub fn iter_lex_tokens(&self) -> LexTokenParsingListIterator<'a> {
        LexTokenParsingListIterator {
            next: self.head,
            next_back: self.tail,
        }
    }


    pub fn fmt_indented(&self, indent: usize, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {

        write_indent(f, indent)?;

        for node in self.iter() {
            
            match &node.value {
                TokenParsingNodeValue::LexToken(token) => {
                    write!(f, "{} ", token)?;
                },
                TokenParsingNodeValue::SyntaxToken(node) => {
                    // This probably will never be executed since when SyntaxTokens are introducen in the list, the list gets consumed and converted into a ScopeBlock.
                    node.fmt_indented(indent, f)?;
                },
                TokenParsingNodeValue::RawScope { inner_tokens, token } => {
                    writeln!(f, "{}", token)?;
                    inner_tokens.fmt_indented(indent + 1, f)?;
                },
                TokenParsingNodeValue::UnparsedScope { statements, token: _ } => {
                    statements.fmt_indented(indent + 1, f)?;
                },
                TokenParsingNodeValue::Placeholder => unreachable!("Placeholder value found in TokenParsingList"),
            }
        }

        Ok(())
    }

}

impl Display for TokenParsingList<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.fmt_indented(0, f)
    }
}

impl Drop for TokenParsingList<'_> {
    fn drop(&mut self) {
        let mut node = self.head;

        while !node.is_null() {
            let owned_node = unsafe { Box::from_raw(node) };
            node = owned_node.right;
        } 
    }
}


pub struct LexTokenParsingListIterator<'a> {
    next: *const TokenParsingNode<'a>,
    next_back: *const TokenParsingNode<'a>,
}

impl<'a> Iterator for LexTokenParsingListIterator<'a> {
    type Item = &'a Token<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        
        if let Some(current_ref) = unsafe { self.next.as_ref() } {
            self.next = current_ref.right;
            match &current_ref.value {
                TokenParsingNodeValue::LexToken(token) => Some(token),
                _ => unreachable!()
            }
        } else {
            None
        }
    }
}

impl<'a> DoubleEndedIterator for LexTokenParsingListIterator<'a> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if let Some(current_ref) = unsafe { self.next_back.as_ref() } {
            self.next_back = current_ref.left;
            match &current_ref.value {
                TokenParsingNodeValue::LexToken(token) => Some(token),
                _ => unreachable!()
            }
        } else {
            None
        }
    }
}


pub struct TokenParsingListIterator<'a> {
    next: *const TokenParsingNode<'a>,
    next_back: *const TokenParsingNode<'a>,
}

impl<'a> DoubleEndedIterator for TokenParsingListIterator<'a> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if let Some(current_ref) = unsafe { self.next_back.as_ref() } {
            self.next_back = current_ref.left;
            Some(current_ref)
        } else {
            None
        }
    }
}

impl<'a> Iterator for TokenParsingListIterator<'a> {
    type Item = &'a TokenParsingNode<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(current_ref) = unsafe { self.next.as_ref() } {
            self.next = current_ref.right;
            Some(current_ref)
        } else {
            None
        }
    }
}


pub struct TokenParsingListIteratorMut<'a> {
    current: *mut TokenParsingNode<'a>,
}

impl<'a> Iterator for TokenParsingListIteratorMut<'a> {
    type Item = &'a mut TokenParsingNode<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(current_ref) = unsafe { self.current.as_mut() } {
            self.current = current_ref.right;
            Some(current_ref)
        } else {
            None
        }
    }
}


#[derive(Debug)]
pub struct SourceToken<'a> {

    pub string: &'a str,
    pub unit_path: &'a Path,
    pub line_index: usize,
    pub column: usize,

}

impl Display for SourceToken<'_> {

    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.string)
    }

}

impl SourceToken<'_> {

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

    Literal { value: Rc<LiteralValue> },
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

pub type TokenPriority = i16;


#[derive(Debug)]
pub enum TokenKind<'a> {

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

impl TokenKind<'_> {

    // pub fn literal_value(&self) -> Option<&LiteralValue> {
    //     if let TokenKind::Value(Value::Literal { value }) = self { Some(value) } else { None }
    // }


    pub fn type_priority(&self) -> TokenPriority {
        (match self {

            TokenKind::Add |
            TokenKind::Sub
                => Priority::Add_Sub,

            TokenKind::Mul |
            TokenKind::Div |
            TokenKind::Mod
                => Priority::Mul_Div_Mod,

            TokenKind::Return |
            TokenKind::Assign |
            TokenKind::Break | 
            TokenKind::Continue
                => Priority::Least_Assignment_FlowBreak,

            TokenKind::Deref |
            TokenKind::Ref
                => Priority::TakeRef,

            TokenKind::FunctionCallOpen |
            TokenKind::ArrayIndexOpen
                => Priority::Delimiter,
            
            TokenKind::Equal |
            TokenKind::NotEqual
                => Priority::Equality,

            TokenKind::Greater |
            TokenKind::Less |
            TokenKind::GreaterEqual |
            TokenKind::LessEqual
                => Priority::Comparison,

            TokenKind::BitwiseNot |
            TokenKind::LogicalNot
                => Priority::Not,

            TokenKind::LogicalAnd => Priority::LogicalAnd,
            TokenKind::LogicalOr => Priority::LogicalOr,

            TokenKind::BitShiftRight |
            TokenKind::BitShiftLeft
                => Priority::Bitshift,

            TokenKind::BitwiseOr => Priority::BitwiseOr,
            TokenKind::BitwiseAnd => Priority::BitwiseAnd,
            TokenKind::BitwiseXor => Priority::BitwiseXor,   

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
                
            TokenKind::DataType(_) |
            TokenKind::Value(_)
                => Priority::PreProcess,

        } as TokenPriority)
    }

}


#[derive(Debug)]
pub struct Token<'a> {

    pub value: TokenKind<'a>,
    pub source_token: Rc<SourceToken<'a>>,
    pub priority: TokenPriority,

}

impl Token<'_> {

    pub fn new<'a>(value: TokenKind<'a>, source_token: SourceToken<'a>, base_priority: TokenPriority) -> Token<'a> {

        let value_priority = value.type_priority();

        Token {
            value,
            source_token: Rc::new(source_token),
            // The priority of the token is the sum of the base priority and the value priority.
            // If the value priority is zero, the token should not be evaluated.
            priority: if value_priority == Priority::Zero as TokenPriority { 0 } else { base_priority + value_priority },
        }
    }

}

impl Display for Token<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.value {
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
            TokenKind::Add => write!(f, "+"),
            TokenKind::Sub => write!(f, "-"),
            TokenKind::Mul => write!(f, "*"),
            TokenKind::Div => write!(f, "/"),
            TokenKind::Mod => write!(f, "%"),
            TokenKind::Assign => write!(f, "="),
            TokenKind::Deref => write!(f, "deref"),
            TokenKind::Continue => write!(f, "continue"),
            TokenKind::Ref => write!(f, "ref"),
            TokenKind::FunctionCallOpen => write!(f, "Call"),
            TokenKind::Return => write!(f, "return"),
            TokenKind::Equal => write!(f, "=="),
            TokenKind::NotEqual => write!(f, "!="),
            TokenKind::Greater => write!(f, ">"),
            TokenKind::Less => write!(f, "<"),
            TokenKind::GreaterEqual => write!(f, ">="),
            TokenKind::LessEqual => write!(f, "<="),
            TokenKind::LogicalNot => write!(f, "!"),
            TokenKind::BitwiseNot => write!(f, "~"),
            TokenKind::LogicalAnd => write!(f, "&&"),
            TokenKind::LogicalOr => write!(f, "||"),
            TokenKind::BitShiftLeft => write!(f, "<<"),
            TokenKind::BitShiftRight => write!(f, ">>"),
            TokenKind::BitwiseOr => write!(f, "|"),
            TokenKind::BitwiseAnd => write!(f, "&"),
            TokenKind::BitwiseXor => write!(f, "^"),
            TokenKind::ArrayIndexOpen => write!(f, "Index"),
            TokenKind::Break => write!(f, "break"),
        }
    }
}

