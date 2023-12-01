use rust_vm_lib::ir::IRCode;

use crate::token::{Token, TokenKind};

use std::ptr;
use std::fmt::Debug;


pub type Statements<'a> = Vec<TokenTree<'a>>;


pub struct TokenNode<'a> {

    pub left: *mut TokenNode<'a>,
    pub right: *mut TokenNode<'a>,

    pub children: Option<TokenTree<'a>>,

    pub item: Token<'a>,

}

impl TokenNode<'_> {

    pub fn new(item: Token<'_>) -> TokenNode<'_> {
        TokenNode {
            left: ptr::null_mut(),
            right: ptr::null_mut(),
            children: None,
            item,
        }
    }

}


pub struct TokenTree<'a> {

    first: *mut TokenNode<'a>,
    last: *mut TokenNode<'a>,

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


    pub fn is_empty(&self) -> bool {
        self.first.is_null()
    }


    pub fn has_one_item(&self) -> bool {
        !self.first.is_null() && self.first == self.last
    }


    /// Extracts a slice of the token tree and returns it as a new token tree
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


    pub fn fmt(&self, indent: usize, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let mut node = self.first;

        while !node.is_null() {
            let node_ref = unsafe { &(*node) };

            for _ in 0..indent {
                write!(f, "  ")?;
            }
            write!(f, "| ")?;

            if let TokenKind::ScopeOpen { statements: Some(statements) } = &node_ref.item.value {
                writeln!(f, "ScopeOpen:")?;

                for statement in statements {
                    statement.fmt(indent + 1, f)?;
                    writeln!(f, "EndStatement")?;
                }

                writeln!(f)?;
            } else {
                writeln!(f, "{:?}", node_ref.item.value)?;

                if let Some(children) = &node_ref.children {
                    children.fmt(indent + 1, f)?;
                }
            }            

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


impl Debug for TokenTree<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.fmt(0, f)
    }
}


fn find_next_scope(tokens: *mut TokenNode) -> Option<*mut TokenNode> {
    let mut token = tokens;

    while !token.is_null() {
        
        if matches!(unsafe { &(*token).item.value }, &TokenKind::ScopeOpen { .. }) {
            return Some(token);
        }

        token = unsafe { (*token).right };
    }

    None
}


fn find_scope_end(tokens: *mut TokenNode) -> *mut TokenNode {

    let mut scope_depth: usize = 1;

    let mut token = tokens;
    while !token.is_null() {

        match unsafe { &(*token).item.value } {
            TokenKind::ScopeOpen { .. } => scope_depth += 1,
            TokenKind::ScopeClose => scope_depth -= 1,
            _ => (),
        }

        if scope_depth == 0 {
            return token;
        }

        token = unsafe { (*token).right };
    }

    panic!("Scope end not found. This is a bug.");
}


pub fn parse_scope_hierarchy(tokens: &mut TokenTree<'_>) {
    // This function shouldn't fail because the tokenizer has already checked that the scopes are balanced

    let mut node = tokens.first;

    while !node.is_null() {

        if let Some(scope_node) = find_next_scope(node) {
            let scope_start = unsafe { (*scope_node).right };
            let scope_end = find_scope_end(scope_start);

            node = unsafe { (*scope_end).right };
            
            // Don't parse empty scopes
            if scope_start == scope_end {
                continue;
            }

            let mut inner_scope = tokens.extract_slice(scope_start, scope_end);
            
            // Recursively parse the inner scope hierarchy
            parse_scope_hierarchy(&mut inner_scope);

            unsafe {
                (*scope_node).children = Some(inner_scope);
            }

        } else {
            node = unsafe { (*node).right };
        }

    }
}


pub fn divide_statements(mut tokens: TokenTree) -> Statements {
    // Divide the tree into separate statements recursively

    let mut statements = Statements::new();
    let mut node = tokens.first;

    while !node.is_null() {

        let node_ref = unsafe { &mut *node };
        match &mut node_ref.item.value {

            TokenKind::Semicolon => {
                // End of statement
                let mut statement = tokens.extract_slice(tokens.first, node);

                // Check if the statement is empty (contains only a semicolon)
                if !statement.has_one_item() {
                    // Drop the semicolon token
                    statement.drop_last();
                    statements.push(statement);
                }

                node = tokens.first;
            },

            TokenKind::ScopeOpen { statements: scope_statements } => {
                // End of statement
                // Recursively parse the nested scope into separate statements

                // Extract the scope statements from the scope token children if the scope isn't empty
                if let Some(children) = node_ref.children.take() {
                    *scope_statements = Some(divide_statements(children));
                }

                let statement = tokens.extract_slice(tokens.first, node);
                statements.push(statement);

                node = tokens.first;
            },

            _ => node = node_ref.right,
        }
    }

    if !tokens.is_empty() {
        statements.push(tokens);
    }

    statements
}


// fn find_highest_priority(tokens: &TokenTree) -> Option<*mut TokenNode> {

//     let mut highest_priority: Option<*mut TokenNode> = None;
//     let mut node = tokens.first;

//     while !node.is_null() {
            
//         // Don't search past the end of the statement or into a new scope
//         if matches!(node.item.value, TokenKind::Semicolon | TokenKind::ScopeOpen) {
//             break;
//         }

//         if let Some(hp) = highest_priority {
//             if node.item.priority > hp.item.priority {
//                 highest_priority = Some(node);
//             }
//         }
//     }

//     highest_priority
// }


// pub fn parse_operator_hierarchy(tokens: &mut TokenTree<'_>, source: &IRCode) {
//     // Recursively parse the operator hierarchy based on token priority

//     loop {

//         let node = find_highest_priority(tokens);

//     }


// }

