use std::ptr;
use std::rc::Rc;

use crate::data_types::{DataType, LiteralValue};
use crate::match_unreachable;
use crate::symbol_table::{ScopeDiscriminant, ScopeID};
use crate::token::Token;


#[derive(Debug)]
pub struct UnparsedScopeBlock<'a> {
    pub statements: Vec<TokenTree<'a>>,
    pub scope_id: ScopeID,
}

impl UnparsedScopeBlock<'_> {

    pub fn new(scope_id: ScopeID) -> Self {
        Self {
            statements: Vec::new(),
            scope_id
        }
    }

}

pub struct ScopeBlock<'a> {
    pub statements: Vec<TokenNode<'a>>,
    pub scope_id: ScopeID,
}


impl ScopeBlock<'_> {    

    pub fn new(scope_id: ScopeID) -> Self {
        Self {
            statements: Vec::new(),
            scope_id,
        }
    }

    pub fn return_type(&self) -> Rc<DataType> {
        if let Some(last_statement) = self.statements.last() {
            last_statement.data_type.clone()
        } else {
            DataType::Void.into()
        }
    }

    pub fn return_value_literal(&self) -> Option<&LiteralValue> {
        if let Some(last_statement) = self.statements.last() {
            last_statement.item.value.literal_value()
        } else {
            None
        }
    }

}

impl std::fmt::Debug for ScopeBlock<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for statement in &self.statements {
            statement.fmt_indented(0, f)?;
        }
        Ok(())
    }
}


#[derive(Debug)]
pub struct IfBlock<'a> {
    pub condition: TokenNode<'a>,
    pub body: ScopeBlock<'a>,
}


pub struct FunctionParam<'a> {
    pub name: &'a str,
    pub data_type: Rc<DataType>,
    pub mutable: bool,
}


#[derive(Debug)]
pub enum ChildrenType<'a> {
    /// A list of syntax nodes
    List (Vec<TokenNode<'a>>),
    /// Temporary type used during parsing to store the children of a scope token
    Tree (TokenTree<'a>),
    /// A list of statements (e.g. a function body)
    ParsedBlock (ScopeBlock<'a>),
    UnparsedBlock (UnparsedScopeBlock<'a>),
    /// A list of function parameters (pairs of name and type)
    FunctionParams (Vec<FunctionParam<'a>>), 
    Function { name: &'a str, signature: Rc<DataType>, body: ScopeBlock<'a> },
    TypeCast { target_type: Rc<DataType>, expr: Box<TokenNode<'a>> },
    Call { callable: Box<TokenNode<'a>>, args: Vec<TokenNode<'a>> },
    Binary (Box<TokenNode<'a>>, Box<TokenNode<'a>>),
    Unary (Box<TokenNode<'a>>),
    While { condition: Box<TokenNode<'a>>, body: ScopeBlock<'a> },
    IfChain { if_chain: Vec<IfBlock<'a>>, else_block: Option<ScopeBlock<'a>> },
    Const { name: &'a str, discriminant: ScopeDiscriminant, data_type: Rc<DataType>, definition: Box<TokenNode<'a>> },
    TypeDef { name: &'a str, definition: Rc<DataType>, },
}


/// A syntax node in the code.
/// This node is used to construct a bush that represents the program's syntactical hierarchy.
#[derive(Debug)]
pub struct TokenNode<'a> {

    pub left: *mut TokenNode<'a>,
    pub right: *mut TokenNode<'a>,

    /// The syntactical children of the token (its operands or its contents)
    pub children: Option<ChildrenType<'a>>,

    /// The source code token
    pub item: Token<'a>,

    /// The data type this node evaluates to
    pub data_type: Rc<DataType>,

    /// Whether the node may have side effects.
    /// If this is false, then the node is guaranteed to not have side effects.
    /// Since the compiler must be conservative, this field might be set to true even if the node does not have side effects, if the compiler can't determine that it doesn't.
    pub has_side_effects: bool,

}

impl<'a> TokenNode<'a> {

    pub fn new(item: Token<'_>) -> TokenNode<'_> {
        TokenNode {
            left: ptr::null_mut(),
            right: ptr::null_mut(),
            children: None,
            item,
            data_type: Rc::new(DataType::Void),
            has_side_effects: false,
        }
    }

    /// Replace the contents of this node with the contents of another node (without changing the position of this node in the tree)
    /// 
    /// The other node is automatically consumed and dropped after the substitution.
    pub fn substitute(&mut self, other: TokenNode<'a>) {
        self.children = other.children;
        self.item = other.item;
        self.data_type = other.data_type;
    }

    pub fn left(&'a self) -> Option<&'a TokenNode> {
        if self.left.is_null() {
            None
        } else {
            Some(unsafe { &*self.left })
        }
    }

    #[allow(dead_code)]
    pub fn right(&'a self) -> Option<&'a TokenNode> {
        if self.right.is_null() {
            None
        } else {
            Some(unsafe { &*self.right })
        }
    }


    pub fn fmt_indented(&self, indent: usize, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // format: "<indent> | <token>"

        fn write_indent(f: &mut std::fmt::Formatter, indent: usize) -> std::fmt::Result {
            for _ in 0..indent {
                write!(f, "  ")?;
            }
            Ok(())
        }
        
        write_indent(f, indent)?;
        writeln!(f, "| {:?} (p: {}) (dt: {})", self.item.value, self.item.priority, self.data_type)?;

        if let Some(children) = &self.children {
            match children {
                ChildrenType::TypeDef { name, definition } => {
                    write!(f, "typedef {name} = {}", definition.name())?;
                }
                ChildrenType::List (children) => {
                    for child in children {
                        child.fmt_indented(indent + 1, f)?;
                    }
                },
                ChildrenType::Tree (children) => {
                    children.fmt_indented(indent+ 1, f)?;
                },
                ChildrenType::ParsedBlock (block) => {
                    for statement in block.statements.iter() {
                        statement.fmt_indented(indent + 1, f)?;
                        write_indent(f, indent + 1)?;
                        writeln!(f, "---")?;
                    }
                },
                ChildrenType::UnparsedBlock(children) => {
                    for statement in children.statements.iter() {
                        statement.fmt_indented(indent+ 1, f)?;
                        write_indent(f, indent + 1)?;
                        writeln!(f, "---")?;
                    }
                }
                ChildrenType::FunctionParams (params) => {
                    for param in params {
                        write_indent(f, indent)?;
                        writeln!(f, "{:?}", param)?;
                    }
                },
                ChildrenType::Function { name, signature, body } => {
                    write!(f, "fn {} (", name)?;
                    let (params, return_type) = match_unreachable!(DataType::Function { params, return_type } = &**signature, (params, return_type));
                    for (i, data_type) in params.iter().enumerate() {
                        write!(f, "{data_type}")?;
                        if i < params.len() - 1 {
                            write!(f, ", ")?;
                        }
                    }
                    writeln!(f, ") -> {}", return_type)?;
                    
                    for statement in body.statements.iter() {
                        statement.fmt_indented(indent + 1, f)?;
                        write_indent(f, indent + 1)?;
                        writeln!(f, "---")?;
                    }
                },
                ChildrenType::TypeCast { target_type: data_type, expr: value } => {
                    write!(f, "({:?}) as {}", value.item.value, data_type)?;
                },
                ChildrenType::Call { callable, args } => {
                    write!(f, "{:?}(", callable.item.value)?;
                    for (i, arg) in args.iter().enumerate() {
                        write!(f, "{:?}", arg.item.value)?;
                        if i < args.len() - 1 {
                            write!(f, ", ")?;
                        }
                    }
                    write!(f, ")")?;
                },
                ChildrenType::Binary (op1, op2) => {
                    op1.fmt_indented(indent + 1, f)?;
                    op2.fmt_indented(indent + 1, f)?;
                },
                ChildrenType::Unary (op) => {
                    op.fmt_indented(indent + 1, f)?;
                },
                ChildrenType::While { condition, body } => {
                    writeln!(f, "while")?;
                    condition.fmt_indented(indent + 1, f)?;
                    writeln!(f, "do")?;
                    for statement in body.statements.iter() {
                        statement.fmt_indented(indent + 1, f)?;
                        write_indent(f, indent + 1)?;
                        writeln!(f, "---")?;
                    }
                },
                ChildrenType::IfChain { if_chain: ifs, else_block } => {
                    for if_node in ifs.iter() {
                        writeln!(f, "if")?;
                        if_node.condition.fmt_indented(indent + 1, f)?;
                        writeln!(f, "then")?;
                        for statement in if_node.body.statements.iter() {
                            statement.fmt_indented(indent + 1, f)?;
                            write_indent(f, indent + 1)?;
                            writeln!(f, "---")?;
                        }
                    }

                    if let Some(else_block) = else_block {
                        writeln!(f, "else")?;
                        for statement in else_block.statements.iter() {
                            statement.fmt_indented(indent + 1, f)?;
                            write_indent(f, indent + 1)?;
                            writeln!(f, "---")?;
                        }
                    }
                },
                ChildrenType::Const { name, discriminant: _, data_type, definition } => {
                    write!(f, "const {name}: {data_type} = ")?;
                    definition.fmt_indented(indent + 1, f)?;
                },
            }
        }
        
        Ok(())
    }

}


pub struct TokenTreeIterator<'a> {
    current: *const TokenNode<'a>,
}

impl TokenTreeIterator<'_> {
    pub fn new<'a>(tree: &TokenTree<'a>) -> TokenTreeIterator<'a> {
        TokenTreeIterator {
            current: tree.first,
        }
    }
}

impl<'a> Iterator for TokenTreeIterator<'a> {
    type Item = &'a TokenNode<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let current = self.current;
        if !current.is_null() {
            let current_ref = unsafe { &*current };
            self.current = current_ref.right;
            Some(current_ref)
        } else {
            None
        }
    }
}


pub struct TokenTree<'a> {

    /// This field is public to allow the parser to access it directly
    pub first: *mut TokenNode<'a>,
    /// This field is public to allow the parser to access it directly
    pub last: *mut TokenNode<'a>,

}

impl<'a> TokenTree<'a> {

    pub fn new() -> Self {
        Self {
            first: ptr::null_mut(),
            last: ptr::null_mut(),
        }
    }


    pub fn from_slice(start: *mut TokenNode<'a>, end: *mut TokenNode<'a>) -> Self {
        Self {
            first: start,
            last: end,
        }
    }


    pub fn append(&mut self, item: Token<'a>) {
        
        let node = Box::leak(Box::new(TokenNode::new(item)));

        if self.first.is_null() {
            self.first = node;
            self.last = node;
        } else {
            unsafe {
                (*self.last).right = node;
                node.left = self.last;
                self.last = node;
            }
        }
    }


    pub fn drop_last(&mut self) {
        if self.last.is_null() {
            return;
        }

        unsafe {
            let new_last = (*self.last).left;
            (*self.last).left = ptr::null_mut();
            if !new_last.is_null() {
                (*new_last).right = ptr::null_mut();
            }
            // Drop the last node
            let _ = Box::from_raw(self.last);
            self.last = new_last;
        }
    }


    pub fn last_node(&self) -> Option<&'a TokenNode<'a>> {
        if self.last.is_null() {
            None
        } else {
            Some(unsafe { &(*self.last) })
        }
    }


    pub fn is_empty(&self) -> bool {
        self.first.is_null()
    }


    pub fn has_one_item(&self) -> bool {
        !self.first.is_null() && self.first == self.last
    }


    pub fn extract_first(&mut self) -> Option<Box<TokenNode<'a>>> {
        self.extract_node(self.first)
    }


    /// Remove the node from the tree assuming it is in the tree, and return it as a boxed pointer to prevent memory leaks
    pub fn extract_node(&mut self, node: *mut TokenNode<'a>) -> Option<Box<TokenNode<'a>>> {

        if node.is_null() {
            return None;
        }

        let node_ref = unsafe { &mut *node };
        
        if node == self.first {
            // node is the first node
            self.first = node_ref.right;
        } else {
            // node is not the first node
            unsafe {
                (*node_ref.left).right = node_ref.right;
            }
        }

        if node == self.last {
            // node is the last node
            self.last = node_ref.left;
        } else {
            // node is not the last node
            unsafe {
                (*node_ref.right).left = node_ref.left;
            }
        }

        node_ref.left = ptr::null_mut();
        node_ref.right = ptr::null_mut();

        Some(unsafe { Box::from_raw(node) })
    }


    /// Extracts a slice of the token tree and returns it as a new token tree.
    /// 
    /// Both start and end are included in the slice
    pub fn extract_slice(&mut self, start: *mut TokenNode<'a>, end: *mut TokenNode<'a>) -> TokenTree<'a> {
        
        // Remove the slice from the tree
        unsafe {
            if (*start).left.is_null() {
                // start is the first node
                self.first = (*end).right;
            } else {
                // start is not the first node
                (*(*start).left).right = (*end).right;
            }

            if (*end).right.is_null() {
                // end is the last node
                self.last = (*start).left;
            } else {
                // end is not the last node
                (*(*end).right).left = (*start).left;
            }

            (*start).left = ptr::null_mut();
            (*end).right = ptr::null_mut();
        }

        // Create a new token tree from the slice
        TokenTree::from_slice(start, end)
    }


    pub fn iter(&self) -> TokenTreeIterator<'a> {
        TokenTreeIterator::new(self)
    }    


    pub fn fmt_indented(&self, indent: usize, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for statement in self.iter() {
            statement.fmt_indented(indent + 1, f)?;
        }
        Ok(())
    }

}


impl Drop for TokenTree<'_> {
    fn drop(&mut self) {
        let mut node = self.first;

        while !node.is_null() {
            let owned_node = unsafe { Box::from_raw(node) };
            node = owned_node.right;
        }        
    }
}


impl std::fmt::Debug for TokenTree<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.fmt_indented(0, f)
    }
}


impl std::fmt::Debug for FunctionParam<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}: {}", if self.mutable { "mut " } else { "" }, self.name, self.data_type)
    }
}

