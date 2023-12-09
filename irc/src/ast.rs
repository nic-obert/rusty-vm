use rust_vm_lib::ir::IRCode;

use crate::operations;
use crate::error;
use crate::operations::Ops;
use crate::token::{Token, TokenKind, is_value_holder};

use std::ptr;
use std::fmt::Debug;


pub type Statements<'a> = Vec<TokenTree<'a>>;


pub enum ChildrenType<'a> {
    List (Vec<*mut TokenNode<'a>>),
    Tree (TokenTree<'a>),
    Code (Statements<'a>),
}

pub struct TokenNode<'a> {

    pub left: *mut TokenNode<'a>,
    pub right: *mut TokenNode<'a>,

    pub children: Option<ChildrenType<'a>>,

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


    pub fn extract_node(&mut self, node: *mut TokenNode<'a>) -> Option<*mut TokenNode<'a>> {
        // Remove the node from the tree assuming it is in the tree and return it

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

        Some(node)
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


    pub fn fmt(&self, indent: usize, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let mut node = self.first;

        while !node.is_null() {
            let node_ref = unsafe { &(*node) };

            for _ in 0..indent {
                write!(f, "  ")?;
            }
            write!(f, "| ")?;

            writeln!(f, "{:?}", node_ref.item.value)?;

            if let Some(children) = &node_ref.children {
                match children {
                    ChildrenType::List (children) => {
                        for child in children {
                            for _ in 0..indent {
                                write!(f, "  ")?;
                            }
                            write!(f, "| ")?;
                            writeln!(f, "{:?}", unsafe { &(**child).item.value })?;
                        }
                    },
                    ChildrenType::Tree (children) => {
                        children.fmt(indent + 1, f)?;
                    },
                    ChildrenType::Code (children) => {
                        for statement in children {
                            statement.fmt(indent + 1, f)?;
                            writeln!(f, "EndStatement")?;
                        }
                    },
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


fn find_next_token<'a>(tokens: *mut TokenNode<'a>, target: &TokenKind<'_>) -> Option<*mut TokenNode<'a>> {
    let mut token = tokens;

    while !token.is_null() {

        let token_ref = unsafe { &(*token).item };
        
        if matches!(&token_ref.value, target) {
            return Some(token);
        }

        token = unsafe { (*token).right };
    }

    None
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
                (*scope_node).children = Some(ChildrenType::Tree(inner_scope));
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

            TokenKind::ScopeOpen => {
                // End of statement
                // Recursively parse the nested scope into separate statements

                // Extract the scope statements from the scope token children if the scope isn't empty and convert them into a list of statements
                if let Some(ChildrenType::Tree(children_tree)) = node_ref.children.take() {
                    node_ref.children = Some(ChildrenType::Code(divide_statements(children_tree)));
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


pub fn parse_statements_hierarchy(statements: &mut Statements, source: &IRCode) {
    // Recursively parse the statements' hierarchy
    // Do not check the types of the operators yet. This will be done in the next pass when the symbol table is created.

    for statement in statements {
        parse_statement_hierarchy(statement, source);
    }
}


/// Extract comma-separated tokens within a delimiter (parentheses, square brackets, etc.)
fn extract_delimiter_contents<'a>(tokens: &mut TokenTree<'a>, next_token: *mut TokenNode<'a>, operator: &TokenKind<'_>, delimiter: &TokenKind<'_>, source: &IRCode) -> Vec<*mut TokenNode<'a>> {
    let mut arguments = Vec::new();

    let mut arg = next_token;

    // Extract the arguments within the delimiters
    loop {

        let arg_ref = unsafe { &mut *arg };

        match &arg_ref.item.value {

            t if t == delimiter => break,     

            token => {

                // Check if token is a valid list argument (can hold a value e.g. literal, variable, function call, etc. but not a function definition, colon, comma...)
                if is_value_holder(token) {
                    arguments.push(arg);
                } else {
                    error::unexpected_token(arg_ref.item.unit_path, &arg_ref.item, arg_ref.item.line, arg_ref.item.column, &source[arg_ref.item.line], format!("Unexpected token {:?} in this context. Expected a value holder token.", arg_ref.item.value).as_str());
                }

                // Check if the next token is a comma
                let next_token = tokens.extract_node(arg_ref.right).unwrap_or_else(
                    || error::expected_argument(arg_ref.item.unit_path, operator, arg_ref.item.line, arg_ref.item.column, &source[arg_ref.item.line], format!("Missing argument or closing delimiter for operator {:?}.", operator).as_str())
                );
                let next_token_ref = unsafe { &mut *next_token };
                arg = if matches!(next_token_ref.item.value, TokenKind::Comma) {
                    // The next token is a comma, so skip it
                    tokens.extract_node(next_token_ref.right).unwrap_or_else(
                        || error::expected_argument(next_token_ref.item.unit_path, operator, next_token_ref.item.line, next_token_ref.item.column, &source[next_token_ref.item.line], format!("Missing argument or closing delimiter for operator {:?}.", operator).as_str())
                    )
                } else {
                    // The next token is not a comma. The next iteration will determine if it's a valid closing delimiter or an invalid generic token
                    next_token
                };
            }
        }
    }

    arguments
}


fn parse_statement_hierarchy(tokens: &mut TokenTree<'_>, source: &IRCode) {
    // Recursively parse the statement hierarchy based on token priority
    // Do not check the types of the operators yet. This will be done in the next pass when the symbol table is created.

    while let Some(node) = find_highest_priority(tokens) {

        let node_ref = unsafe { &mut *node };
        if node_ref.item.priority == 0 {
            // No more operations to parse
            break;
        }

        // Satisfy the operator requirements
        match &node_ref.item.value {
            TokenKind::Op(op) => match op {

                // Binary operators:
                operations::Ops::Add |
                operations::Ops::Sub |
                operations::Ops::Mul |
                operations::Ops::Div |
                operations::Ops::Mod |
                operations::Ops::Assign |
                operations::Ops::Equal |
                operations::Ops::NotEqual |
                operations::Ops::Greater |
                operations::Ops::Less |
                operations::Ops::GreaterEqual |
                operations::Ops::LessEqual |
                operations::Ops::LogicalAnd |
                operations::Ops::LogicalOr |
                operations::Ops::BitShiftLeft |
                operations::Ops::BitShiftRight |
                operations::Ops::BitwiseOr |
                operations::Ops::BitwiseAnd |
                operations::Ops::BitwiseXor 
                 => {
                    let left = tokens.extract_node(node_ref.left).unwrap_or_else(
                        || error::expected_argument(node_ref.item.unit_path, &node_ref.item.value, node_ref.item.line, node_ref.item.column, &source[node_ref.item.line], format!("Missing left argument for operator {}.", op).as_str())
                    );
                
                    let right = tokens.extract_node(node_ref.right).unwrap_or_else(
                        || error::expected_argument(node_ref.item.unit_path, &node_ref.item.value, node_ref.item.line, node_ref.item.column, &source[node_ref.item.line], format!("Missing right argument for operator {}.", op).as_str())
                    );
                
                    node_ref.children = Some(ChildrenType::List(vec![left, right]));
                },

                // Unary operators left:
                operations::Ops::Deref |
                operations::Ops::Ref |
                operations::Ops::LogicalNot |
                operations::Ops::BitwiseNot
                 => {
                    let left = tokens.extract_node(node_ref.left).unwrap_or_else(
                        || error::expected_argument(node_ref.item.unit_path, &node_ref.item.value, node_ref.item.line, node_ref.item.column, &source[node_ref.item.line], format!("Missing left argument for operator {}.", *op).as_str())
                    );
                    node_ref.children = Some(ChildrenType::List(vec![left]));
                },

                // Unary operators right:
                operations::Ops::Return |
                operations::Ops::Jump 
                 => {
                    let right = tokens.extract_node(node_ref.right).unwrap_or_else(
                        || error::expected_argument(node_ref.item.unit_path, &node_ref.item.value, node_ref.item.line, node_ref.item.column, &source[node_ref.item.line], format!("Missing right argument for operator {}.", *op).as_str())
                    );
                    node_ref.children = Some(ChildrenType::List(vec![right]));
                },

                // Other operators:
                operations::Ops::Call => {
                    // Functin call is a list of tokens separated by commas enclosed in parentheses
                    // Statements inside the parentheses have already been parsed into single top-level tokens because of their higher priority

                    let arguments = extract_delimiter_contents(tokens, node_ref.right, &node_ref.item.value, &TokenKind::ParClose, source);
                    node_ref.children = Some(ChildrenType::List(arguments));
                },
                
            },

            TokenKind::ParOpen => {
                // Extract the tokens within the parentheses
                let inner_tokens = extract_delimiter_contents(tokens, node_ref.right, &node_ref.item.value, &TokenKind::ParClose, source);
                node_ref.children = Some(ChildrenType::List(inner_tokens));
            },

            TokenKind::SquareOpen => {
                // Extract the tokens within the square brackets
                let inner_tokens = extract_delimiter_contents(tokens, node_ref.right, &node_ref.item.value, &TokenKind::SquareClose, source);
                node_ref.children = Some(ChildrenType::List(inner_tokens));
            },

            TokenKind::ScopeOpen => {
                // Parse the children of the scope. The children have already been extracted and divided into separate statements
                todo!()
            },

            TokenKind::Fn => todo!(),
            TokenKind::Let => todo!(),
            
            _ => unreachable!("Invalid token kind during statement hierarchy parsing: {:?}. This token kind shouldn't have children.", node_ref.item.value)
        }


    }
}


fn find_highest_priority<'a>(tokens: &TokenTree<'a>) -> Option<*mut TokenNode<'a>> {

    let mut highest_priority: Option<*mut TokenNode> = None;
    let mut node = tokens.first;

    while !node.is_null() {

        if let Some(hp) = highest_priority {
            unsafe {
                if (*node).item.priority > (*hp).item.priority {
                highest_priority = Some(node);
            }}
        }

        node = unsafe { (*node).right };
    }

    highest_priority
}

