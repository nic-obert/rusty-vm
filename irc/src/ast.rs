use rust_vm_lib::ir::IRCode;

use crate::token::{Token, TokenKind};

use std::ptr;


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
                (*node).left = self.last;
                self.last = node;
            }
        }
    }


    pub fn first_item(&self) -> Option<&Token> {
        if self.first.is_null() {
            None
        } else {
            Some(unsafe { &(*self.first).item })
        }
    }


    pub fn last_item(&self) -> Option<&Token> {
        if self.last.is_null() {
            None
        } else {
            Some(unsafe { &(*self.last).item })
        }
    }


    pub fn is_empty(&self) -> bool {
        self.first.is_null()
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


    pub fn print(&self, indent: usize) -> std::fmt::Result {
        let mut node = self.first;

        while !node.is_null() {
            let node_ref = unsafe { &(*node) };

            for _ in 0..indent {
                print!(" | ");
            }

            println!("{:?}", node_ref.item.value);

            if let Some(children) = &node_ref.children {
                children.print(indent + 1)?;
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


// fn find_highest_priority<'a>(tokens: &'a TokenList<'a>) -> Option<&'a BushNode<Token<'a>>> {

//     let mut highest_priority: Option<&'a BushNode<Token>> = None;

//     for node in tokens.iter_nodes() {
            
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


fn find_next_scope(tokens: *mut TokenNode) -> Option<*mut TokenNode> {
    let mut token = tokens;

    while !token.is_null() {
        
        if matches!(unsafe { &(*token).item.value }, &TokenKind::ScopeOpen) {
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
            TokenKind::ScopeOpen => scope_depth += 1,
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


// TODO: create a new bush-like data structure that is specialized for this use case





pub fn parse_scope_hierarchy(tokens: &mut TokenTree<'_>) {
    // This function shouldn't fail because the tokenizer has already checked 
    // that the scopes are balanced

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

