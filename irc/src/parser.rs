use rust_vm_lib::ir::IRCode;

use crate::data_types::DataType;
use crate::operations;
use crate::error;
use crate::token::Value;
use crate::token::TokenKind;
use crate::symbol_table::{Symbol, SymbolTable, ScopeID};
use crate::token_tree::ChildrenType;
use crate::token_tree::ScopeBlock;
use crate::token_tree::TokenNode;
use crate::token_tree::TokenTree;


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


/// Recursively parse the tokens into a hierarchical tree structure based on nested scopes.
/// 
/// The contents of the scopes are moved into the children of the scope tokens.
fn parse_scope_hierarchy(tokens: &mut TokenTree<'_>) {
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
            // Remove the closing scope token
            inner_scope.drop_last();
            
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


/// Divide the tree into a list of separate statements based on semicolons and scopes.
fn divide_statements<'a>(mut tokens: TokenTree<'a>, symbol_table: &mut SymbolTable, parent_scope: Option<ScopeID>) -> ScopeBlock<'a> {

    let scope_id = symbol_table.add_scope(parent_scope);
    let mut block = ScopeBlock::new(scope_id);
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
                    block.statements.push(statement);
                }

                node = tokens.first;
            },

            TokenKind::ScopeOpen => {
                // Recursively parse the nested scope into separate statements

                // Extract the scope statements from the scope token children if the scope isn't empty and convert them into a list of statements
                if let Some(ChildrenType::Tree(children_tree)) = node_ref.children.take() {
                    node_ref.children = Some(ChildrenType::Block(divide_statements(children_tree, symbol_table, Some(scope_id))));
                }

                let statement = tokens.extract_slice(tokens.first, node);
                block.statements.push(statement);

                node = tokens.first;
            },

            _ => node = node_ref.right,
        }
    }

    if !tokens.is_empty() {
        block.statements.push(tokens);
    }

    block
}


fn parse_block_hierarchy(block: &mut ScopeBlock, symbol_table: &mut SymbolTable, source: &IRCode) {
    // Recursively parse the statements' hierarchy
    // Do not check the types of the operators yet. This will be done in the next pass when the symbol table is created.

    for statement in &mut block.statements {
        
        while let Some(node) = find_highest_priority(statement) {

            let node_ref = unsafe { &mut *node };
            if node_ref.item.priority == 0 {
                // No more operations to parse
                break;
            }
            // Set the priority to 0 so that the node is not visited again
            node_ref.item.priority = 0;
    
            // Useful macros to get tokens without forgetting that the token pointers of extracted tokens are invalidated
            macro_rules! extract_left {
                () => {
                    statement.extract_node(node_ref.left)
                };
            }
            macro_rules! extract_right {
                () => {
                    statement.extract_node(node_ref.right)
                };
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
                        let left = extract_left!().unwrap_or_else(
                            || error::expected_argument(node_ref.item.unit_path, &node_ref.item.value, node_ref.item.token.line_number(), node_ref.item.token.column, &source[node_ref.item.token.line_index()], format!("Missing left argument for operator {}.", op).as_str())
                        );
                    
                        let right = extract_right!().unwrap_or_else(
                            || error::expected_argument(node_ref.item.unit_path, &node_ref.item.value, node_ref.item.token.line_number(), node_ref.item.token.column, &source[node_ref.item.token.line_index()], format!("Missing right argument for operator {}.", op).as_str())
                        );
                    
                        node_ref.children = Some(ChildrenType::List(vec![*left, *right]));
                    },
    
                    // Unary operators left:
                    operations::Ops::Deref |
                    operations::Ops::Ref |
                    operations::Ops::LogicalNot |
                    operations::Ops::BitwiseNot
                     => {
                        let left = extract_left!().unwrap_or_else(
                            || error::expected_argument(node_ref.item.unit_path, &node_ref.item.value, node_ref.item.token.line_number(), node_ref.item.token.column, &source[node_ref.item.token.line_index()], format!("Missing left argument for operator {}.", *op).as_str())
                        );
                        node_ref.children = Some(ChildrenType::List(vec![*left]));
                    },
    
                    // Unary operators right:
                    operations::Ops::Return |
                    operations::Ops::Jump 
                     => {
                        let right = extract_right!().unwrap_or_else(
                            || error::expected_argument(node_ref.item.unit_path, &node_ref.item.value, node_ref.item.token.line_number(), node_ref.item.token.column, &source[node_ref.item.token.line_index()], format!("Missing right argument for operator {}.", *op).as_str())
                        );
                        node_ref.children = Some(ChildrenType::List(vec![*right]));
                    },
    
                    // Other operators:
                    operations::Ops::Call => {
                        // Functin call is a list of tokens separated by commas enclosed in parentheses
                        // Statements inside the parentheses have already been parsed into single top-level tokens because of their higher priority
    
                        let arguments = extract_list_like_delimiter_contents(statement, node_ref, &node_ref.item.value, &TokenKind::ParClose, source);
                        node_ref.children = Some(ChildrenType::List(arguments));
                    },
                    
                },
    
                TokenKind::ParOpen => {
                    // Extract the tokens within the parentheses
                    let inner_tokens = extract_list_like_delimiter_contents(statement, node_ref, &node_ref.item.value, &TokenKind::ParClose, source);
                    node_ref.children = Some(ChildrenType::List(inner_tokens));
                },
    
                TokenKind::ArrayOpen => {
                    // Extract the tokens within the square brackets
                    let inner_tokens = extract_list_like_delimiter_contents(statement, node_ref, &node_ref.item.value, &TokenKind::SquareClose, source);
                    node_ref.children = Some(ChildrenType::List(inner_tokens));
                },
    
                TokenKind::ScopeOpen => {
                    // Parse the children statements of the scope.
                    // The children have already been extracted and divided into separate statements.
                    
                    if let Some(ChildrenType::Block(statements)) = &mut node_ref.children {
                        parse_block_hierarchy(statements, symbol_table, source);
                    }
                },
    
                TokenKind::Let => {
                    // Syntax: let [mut] <name>: <type> 
    
                    // This node can either be the symbol name or the mut keyword
                    let next_node = extract_right!().unwrap_or_else(
                        || error::expected_argument(node_ref.item.unit_path, &node_ref.item.value, node_ref.item.token.line_number(), node_ref.item.token.column, &source[node_ref.item.token.line_index()], "Missing variable name after let in variable declaration.")
                    );
    
                    let (mutable, name_token) = if matches!(next_node.item.value, TokenKind::Mut) {
                        let name = extract_right!().unwrap_or_else(
                            || error::expected_argument(node_ref.item.unit_path, &node_ref.item.value, node_ref.item.token.line_number(), node_ref.item.token.column, &source[node_ref.item.token.line_index()], "Missing variable name after let in variable declaration.")
                        );
                        (true, name)
                    } else {
                        (false, next_node)
                    };
    
                    let symbol_name: &str = if let TokenKind::Value(Value::Symbol { id }) = &name_token.item.value {
                        id
                    } else {
                        error::invalid_argument(node_ref.item.unit_path, &node_ref.item.value, node_ref.item.token.line_number(), node_ref.item.token.column, &source[node_ref.item.token.line_index()], format!("Invalid variable name {:?} in variable declaration.", name_token.item.value).as_str())
                    };
    
                    let _colon = extract_right!().unwrap_or_else(
                        || error::expected_argument(node_ref.item.unit_path, &node_ref.item.value, node_ref.item.token.line_number(), node_ref.item.token.column, &source[node_ref.item.token.line_index()], "Missing colon after variable name in variable declaration.")
                    );
    
                    // The data type should be a single top-level token because of its higher priority
                    let data_type_node = extract_right!().unwrap_or_else(
                        || error::expected_argument(node_ref.item.unit_path, &node_ref.item.value, node_ref.item.token.line_number(), node_ref.item.token.column, &source[node_ref.item.token.line_index()], "Missing data type after colon in variable declaration.")
                    );
                    let data_type = if let TokenKind::DataType(data_type) = data_type_node.item.value {
                        data_type
                    } else {
                        error::invalid_argument(node_ref.item.unit_path, &node_ref.item.value, node_ref.item.token.line_number(), node_ref.item.token.column, &source[node_ref.item.token.line_index()], format!("Invalid data type {:?} in variable declaration.", data_type_node.item.value).as_str())
                    };

                    // Declare the new symbol in the local scope
                    symbol_table.declare_symbol(
                        Symbol::new(symbol_name.to_string(), data_type, mutable),
                        block.scope_id
                    );
    
                    // Transform this node into a symbol node (the declaration let is no longer needed since the symbol is already declared)
                    node_ref.substitute(*name_token);
                },
    
                TokenKind::Fn => {
                    // Function declaration syntax:
                    // fn <name>(<arguments>) -> <return type> { <body> }
    
                    let name_node = extract_right!().unwrap_or_else(
                        || error::expected_argument(node_ref.item.unit_path, &node_ref.item.value, node_ref.item.token.line_number(), node_ref.item.token.column, &source[node_ref.item.token.line_index()], "Missing function name after fn in function declaration.")
                    );
                    let function_name: &str = if let TokenKind::Value(Value::Symbol { id }) = name_node.item.value {
                        id
                    } else {
                        error::invalid_argument(node_ref.item.unit_path, &node_ref.item.value, node_ref.item.token.line_number(), node_ref.item.token.column, &source[node_ref.item.token.line_index()], format!("Invalid function name {:?} in function declaration.", name_node.item.value).as_str())
                    };
    
                    // The parameters should be a single top-level token because of its higher priority
                    let params = extract_right!().unwrap_or_else(
                        || error::expected_argument(node_ref.item.unit_path, &node_ref.item.value, node_ref.item.token.line_number(), node_ref.item.token.column, &source[node_ref.item.token.line_index()], "Missing arguments after function name in function declaration.")
                    );
                    let params = if matches!(params.item.value, TokenKind::FunctionParamsOpen) {
                        if let Some(ChildrenType::FunctionParams(params)) = params.children {
                            params
                        } else {
                            unreachable!("Invalid token kind during statement hierarchy parsing: {:?}. This token kind should be a FunctionParamsOpen token and have FunctionParams children.", params.item.value)
                        }
                    } else {
                        error::invalid_argument(node_ref.item.unit_path, &node_ref.item.value, node_ref.item.token.line_number(), node_ref.item.token.column, &source[node_ref.item.token.line_index()], format!("Invalid parameter declaration {:?} in function declaration. Function parameters must be enclosed in parentheses ().", params.item.value).as_str())
                    };
                    
                    // Extract the arrow ->
                    extract_right!().map(
                        |node| if matches!(node.item.value, TokenKind::Arrow) {
                            node
                        } else {
                            error::invalid_argument(node_ref.item.unit_path, &node_ref.item.value, node_ref.item.token.line_number(), node_ref.item.token.column, &source[node_ref.item.token.line_index()], format!("Invalid token {:?} in function declaration. Expected an arrow -> after the function arguments.", node.item.value).as_str())
                        }
                    ).unwrap_or_else(
                        || error::expected_argument(node_ref.item.unit_path, &node_ref.item.value, node_ref.item.token.line_number(), node_ref.item.token.column, &source[node_ref.item.token.line_index()], "Missing arrow after function arguments in function declaration.")
                    );

                    // The return type should be a single top-level token because of its higher priority
                    let return_type = extract_right!().map(
                        |node| if let TokenKind::DataType(data_type) = node.item.value {
                            data_type
                        } else {
                            error::invalid_argument(node_ref.item.unit_path, &node_ref.item.value, node_ref.item.token.line_number(), node_ref.item.token.column, &source[node_ref.item.token.line_index()], format!("Invalid return type {:?} in function declaration.", node.item.value).as_str())
                        }
                    ).unwrap_or_else(
                        || error::expected_argument(node_ref.item.unit_path, &node_ref.item.value, node_ref.item.token.line_number(), node_ref.item.token.column, &source[node_ref.item.token.line_index()], "Missing return type after arrow in function declaration.")
                    );
                    
                    // The body is one top-level scope token because it gets parsed first
                    let body: ScopeBlock = extract_right!().map(
                        |node| if matches!(node.item.value, TokenKind::ScopeOpen) {
                            if let Some(ChildrenType::Block(body)) = node.children {
                                body
                            } else {
                                unreachable!("Invalid token kind during statement hierarchy parsing: {:?}. This token kind should be a ScopeOpen token and have Block children.", node.item.value)
                            }
                        } else {
                            error::invalid_argument(node_ref.item.unit_path, &node_ref.item.value, node_ref.item.token.line_number(), node_ref.item.token.column, &source[node_ref.item.token.line_index()], format!("Invalid body {:?} in function declaration. Function body must be enclosed in curly braces {{}}.", node.item.value).as_str())
                        }
                    ).unwrap_or_else(
                        || error::expected_argument(node_ref.item.unit_path, &node_ref.item.value, node_ref.item.token.line_number(), node_ref.item.token.column, &source[node_ref.item.token.line_index()], "Missing body after return type in function declaration.")
                    );
                    
                    let function_type = DataType::Function { 
                        params: params.iter().map(|param| param.1.clone()).collect(), // Take only the data type of the parameter
                        return_type: Box::new(return_type.clone())
                    };
                    symbol_table.declare_symbol(
                        Symbol::new(function_name.to_string(), function_type, false), // mutable = false because functions are not mutable
                        block.scope_id
                    );

                    node_ref.children = Some(ChildrenType::Function { name: function_name, params, return_type, body });
                },

                TokenKind::FunctionParamsOpen => {
                    // Syntax: (<name>: <type>, <name>: <type>, ...)

                    let mut params: Vec<(String, DataType)> = Vec::new();

                    let mut expected_comma: bool = false;
                    loop {
                        let param_node = extract_right!().unwrap_or_else(
                            || error::expected_argument(node_ref.item.unit_path, &node_ref.item.value, node_ref.item.token.line_number(), node_ref.item.token.column, &source[node_ref.item.token.line_index()], format!("Missing parameter or closing delimiter for operator {:?}.", node_ref.item.value).as_str())
                        );

                        match param_node.item.value {

                            TokenKind::ParClose => break,

                            TokenKind::Comma => if expected_comma {
                                // Set to false because you cannot have two adjacent commas
                                expected_comma = false;
                            } else {
                                error::unexpected_token(param_node.item.unit_path, &param_node.item, param_node.item.token.line_number(), param_node.item.token.column, &source[param_node.item.token.line_index()], "Unexpected comma in this context. Did you add an extra comma?");
                            },

                            TokenKind::Value(Value::Symbol { id }) => {

                                let name = id.to_string();

                                // Extract the colon
                                extract_right!().map(
                                    |param_node| if !matches!(param_node.item.value, TokenKind::Colon) {
                                        error::invalid_argument(param_node.item.unit_path, &node_ref.item.value, param_node.item.token.line_number(), param_node.item.token.column, &source[param_node.item.token.line_index()], format!("Invalid token {:?} in function declaration. Expected a colon : after the parameter name.", param_node.item.value).as_str());
                                    }
                                ).unwrap_or_else(
                                    || error::expected_argument(node_ref.item.unit_path, &node_ref.item.value, node_ref.item.token.line_number(), node_ref.item.token.column, &source[node_ref.item.token.line_index()], format!("Missing colon after parameter name {:?} in function declaration.", name).as_str())
                                );

                                let data_type = extract_right!().unwrap_or_else(
                                    || error::expected_argument(node_ref.item.unit_path, &node_ref.item.value, node_ref.item.token.line_number(), node_ref.item.token.column, &source[node_ref.item.token.line_index()], "Missing data type after colon in function declaration.")
                                );
                                let data_type = if let TokenKind::DataType(data_type) = data_type.item.value {
                                    data_type
                                } else {
                                    error::invalid_argument(param_node.item.unit_path, &node_ref.item.value, param_node.item.token.line_number(), param_node.item.token.column, &source[param_node.item.token.line_index()], format!("Invalid data type {:?} in function declaration.", data_type.item.value).as_str())
                                };

                                params.push((name, data_type));

                                // A comma is expected after each argument except the last one
                                expected_comma = true;
                            },

                            _ => unreachable!("Invalid token kind during statement hierarchy parsing: {:?}. This token kind shouldn't have children.", param_node.item.value)
                        }
                    }

                    node_ref.children = Some(ChildrenType::FunctionParams(params));
                },

                TokenKind::ArrayTypeOpen => {
                    // TODO: an array slice may be implemented at a later date. this array type would then require a size (or infer it from the context)
                    // Syntax: [<type>]

                    let element_type = extract_right!().map(
                        |node| if let TokenKind::DataType(data_type) = node.item.value {
                            data_type
                        } else {
                            error::invalid_argument(node_ref.item.unit_path, &node_ref.item.value, node_ref.item.token.line_number(), node_ref.item.token.column, &source[node_ref.item.token.line_index()], format!("Invalid data type {:?} in array declaration.", node.item.value).as_str())
                        }
                    ).unwrap_or_else(
                        || error::expected_argument(node_ref.item.unit_path, &node_ref.item.value, node_ref.item.token.line_number(), node_ref.item.token.column, &source[node_ref.item.token.line_index()], "Missing data type after array type open bracket in array declaration.")
                    );
                    
                    // Extract the closing square bracket ]
                    extract_right!().map(
                        |node| if matches!(node.item.value, TokenKind::SquareClose) {
                            node
                        } else {
                            error::invalid_argument(node_ref.item.unit_path, &node_ref.item.value, node_ref.item.token.line_number(), node_ref.item.token.column, &source[node_ref.item.token.line_index()], format!("Invalid token {:?} in array declaration. Expected a closing bracket ] after the array type.", node.item.value).as_str())
                        }
                    ).unwrap_or_else(
                        || error::expected_argument(node_ref.item.unit_path, &node_ref.item.value, node_ref.item.token.line_number(), node_ref.item.token.column, &source[node_ref.item.token.line_index()], "Missing array type close bracket after array type in array declaration.")
                    );
                    
                    let array_type = DataType::Array(Box::new(element_type));
                    // Transform this node into a data type node
                    node_ref.item.value = TokenKind::DataType(array_type);
                },  

                TokenKind::RefType => {
                    // Syntax: &<type>

                    let element_type = extract_right!().map(
                        |node| if let TokenKind::DataType(data_type) = node.item.value {
                            data_type
                        } else {
                            error::invalid_argument(node_ref.item.unit_path, &node_ref.item.value, node_ref.item.token.line_number(), node_ref.item.token.column, &source[node_ref.item.token.line_index()], format!("Invalid data type {:?} in reference declaration.", node.item.value).as_str())
                        }
                    ).unwrap_or_else(
                        || error::expected_argument(node_ref.item.unit_path, &node_ref.item.value, node_ref.item.token.line_number(), node_ref.item.token.column, &source[node_ref.item.token.line_index()], "Missing data type after reference type symbol in reference declaration.")
                    );
                    
                    let ref_type = DataType::Ref(Box::new(element_type));
                    // Transform this node into a data type node
                    node_ref.item.value = TokenKind::DataType(ref_type);
                },
                
                _ => unreachable!("Invalid token kind during statement hierarchy parsing: {:?}. This token kind shouldn't have children.", node_ref.item.value)
            }
        }

    }
}


/// Extract comma-separated tokens within a delimiter (parentheses, square brackets, etc.)
fn extract_list_like_delimiter_contents<'a>(tokens: &mut TokenTree<'a>, start_delimiter: *mut TokenNode<'a>, operator: &TokenKind<'_>, delimiter: &TokenKind<'_>, source: &IRCode) -> Vec<TokenNode<'a>> {
    
    let mut arguments = Vec::new();

    let start_delimiter = unsafe { &mut *start_delimiter };

    // Set to false because the first token in a collection can't be a comma
    let mut expected_comma: bool = false;
   
    // Extract the arguments within the delimiters
    loop {

        let node = tokens.extract_node(start_delimiter.right).unwrap_or_else(
            || error::expected_argument(start_delimiter.item.unit_path, operator, start_delimiter.item.token.line_number(), start_delimiter.item.token.column, &source[start_delimiter.item.token.line_index()], format!("Missing argument or closing delimiter for operator {:?}.", operator).as_str())
        );

        match &node.item.value {

            t if t == delimiter => break,

            TokenKind::Comma => if expected_comma {
                // Set to false because you cannot have two adjacent commas
                expected_comma = false;
            } else {
                error::unexpected_token(node.item.unit_path, &node.item, node.item.token.line_number(), node.item.token.column, &source[node.item.token.line_index()], "Unexpected comma in this context. Did you add an extra comma?");
            },

            _ => {
                // The token type will be checked later
                arguments.push(*node);
                // A comma is expected after each argument except the last one
                expected_comma = true;
            }
        }
    }

    arguments
}


fn find_highest_priority<'a>(tokens: &TokenTree<'a>) -> Option<*mut TokenNode<'a>> {

    let mut highest_priority: Option<*mut TokenNode> = None;
    let mut node = tokens.first;

    while !node.is_null() {

        if let Some(hp) = highest_priority {
            if unsafe { (*node).item.priority > (*hp).item.priority } {
                highest_priority = Some(node);
            }
        } else {
            highest_priority = Some(node);
        }

        node = unsafe { (*node).right };
    }

    highest_priority
}


fn resolve_symbols_and_types(block: &mut ScopeBlock, source: &IRCode) {
    

    
}


pub fn build_ast<'a>(mut tokens: TokenTree<'a>, source: &IRCode) -> (ScopeBlock<'a>, SymbolTable) {

    parse_scope_hierarchy(&mut tokens);

    let mut symbol_table = SymbolTable::new();

    let mut outer_block = divide_statements(tokens, &mut symbol_table, None);

    println!("Statements after division:\n{}", outer_block);

    parse_block_hierarchy(&mut outer_block, &mut symbol_table, source);

    println!("\n\nStatement hierarchy:\n{}", outer_block);

    resolve_symbols_and_types(&mut outer_block, source);

    println!("\n\nAfter symbol resolution:\n{}", outer_block);

    (outer_block, symbol_table)
}

