use std::ptr;

use crate::data_types::DataType;
use crate::symbol_table::ScopeID;
use crate::token::Token;


/// Struct containing the statements of a scope and its symbol table.
pub struct ScopeBlock<'a> {
    pub statements: Vec<TokenTree<'a>>,
    pub scope_id: ScopeID,
}

impl<'a> ScopeBlock<'a> {    

    pub fn new(scope_id: ScopeID) -> Self {
        Self {
            statements: Vec::new(),
            scope_id,
        }
    }

}

impl std::fmt::Display for ScopeBlock<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:#?}", self.statements)
    }
}


pub enum ChildrenType<'a> {
    /// A list of syntax nodes
    List (Vec<TokenNode<'a>>),
    /// Temporary type used during parsing to store the children of a scope token
    Tree (TokenTree<'a>),
    /// A list of statements (e.g. a function body)
    Block (ScopeBlock<'a>),
    /// A list of function parameters (pairs of name and type)
    FunctionParams (Vec<(String, DataType)>), 
    Function { name: &'a str, params: Vec<(String, DataType)>, return_type: DataType, body: ScopeBlock<'a> },
}


pub struct TokenNode<'a> {

    pub left: *mut TokenNode<'a>,
    pub right: *mut TokenNode<'a>,

    /// The syntactical children of the token (its operands or its contents)
    pub children: Option<ChildrenType<'a>>,

    /// The source code token
    pub item: Token<'a>,

    /// The data type this node evaluates to
    pub data_type: DataType,

}

impl<'a> TokenNode<'a> {

    pub fn new(item: Token<'_>) -> TokenNode<'_> {
        TokenNode {
            left: ptr::null_mut(),
            right: ptr::null_mut(),
            children: None,
            item,
            data_type: DataType::Void,
        }
    }

    pub fn substitute(&'a mut self, other: TokenNode<'a>) {
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


    pub fn drop_first(&mut self) {
        if self.first.is_null() {
            return;
        }

        unsafe {
            let new_first = (*self.first).right;
            (*self.first).right = ptr::null_mut();
            if !new_first.is_null() {
                (*new_first).left = ptr::null_mut();
            }
            // Drop the first node
            let _ = Box::from_raw(self.first);
            self.first = new_first;
        }
    }


    pub fn first_item(&self) -> Option<&'a Token> {
        if self.first.is_null() {
            None
        } else {
            Some(unsafe { &(*self.first).item })
        }
    }


    pub fn last_item(&self) -> Option<&'a Token> {
        if self.last.is_null() {
            None
        } else {
            Some(unsafe { &(*self.last).item })
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


    pub fn into_vec(self) -> Vec<*mut TokenNode<'a>> {
        let mut vec = Vec::new();
        let mut node = self.first;

        while !node.is_null() {
            vec.push(node);
            node = unsafe { (*node).right };
        }

        vec
    }


    pub fn print_tokens_only(&self, indent: usize) {
        let mut node = self.first;

        while !node.is_null() {
            let node_ref = unsafe { &(*node) };
            for _ in 0..indent {
                print!("  ");
            }
            println!("{}", node_ref.item.token.string);
            node = node_ref.right;
        }
    }


    pub fn fmt_detailed(&self, indent: usize, f: &mut std::fmt::Formatter) -> std::fmt::Result {

        /// Helper function to write a token to the formatter in a consistent format
        fn write_node(node: &TokenNode<'_>, indent: usize, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            // format: "<indent> | <token>"
            
            for _ in 0..indent {
                write!(f, "  ")?;
            }
            write!(f, "| ")?;
            writeln!(f, "{:?}", node.item.value)?;

            if let Some(children) = &node.children {
                match children {
                    ChildrenType::List (children) => {
                        for child in children {
                            write_node(child, indent + 1, f)?;
                        }
                    },
                    ChildrenType::Tree (children) => {
                        children.fmt_detailed(indent + 1, f)?;
                    },
                    ChildrenType::Block (block) => {
                        for statement in block.statements.iter() {
                            statement.fmt_detailed(indent + 1, f)?;
                            writeln!(f, "EndStatement")?;
                        }
                    },
                    ChildrenType::FunctionParams (params) => {
                        for (name, data_type) in params {
                            for _ in 0..indent + 1 {
                                write!(f, "  ")?;
                            }
                            writeln!(f, "{}: {}", name, data_type)?;
                        }
                    },
                    ChildrenType::Function { name, params, return_type, body } => {
                        write!(f, "fn {} (", name)?;
                        for (i, (name, data_type)) in params.iter().enumerate() {
                            write!(f, "{}: {}", name, data_type)?;
                            if i < params.len() - 1 {
                                write!(f, ", ")?;
                            }
                        }
                        writeln!(f, ") -> {}", return_type)?;
                        
                        for statement in body.statements.iter() {
                            statement.fmt_detailed(indent + 1, f)?;
                            writeln!(f, "EndStatement")?;
                        }
                    },
                }
            }
            
            Ok(())
        }

        let mut node = self.first;

        while let Some(node_ref) = unsafe { node.as_ref() } {
            write_node(node_ref, indent, f)?;
            node = node_ref.right;
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
        self.fmt_detailed(0, f)
    }
}

