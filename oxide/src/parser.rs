use rust_vm_lib::ir::IRCode;

use crate::data_types::dt_macros::unsigned_integer_pattern;
use crate::data_types::DataType;
use crate::error;
use crate::operations::Ops;
use crate::token::LiteralValue;
use crate::token::Value;
use crate::token::TokenKind;
use crate::symbol_table::{Symbol, SymbolTable, ScopeID};
use crate::token_tree::ChildrenType;
use crate::token_tree::ScopeBlock;
use crate::token_tree::TokenNode;
use crate::token_tree::TokenTree;


/// Whether the token kind represents an expression.
fn is_expression(token_kind: &TokenKind) -> bool {
    match token_kind {
        TokenKind::Value(_) |
        TokenKind::ArrayOpen |
        TokenKind::As 
         => true,

        TokenKind::Op(op) => op.returns_a_value(),

        _ => false,
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


/// Recursively parse the tokens into a hierarchical tree structure based on nested scopes.
/// 
/// The contents of the scopes are moved into the children of the scope tokens.
fn parse_scope_hierarchy(tokens: &mut TokenTree<'_>) {
    // This function shouldn't fail because the tokenizer has already checked that the scopes are balanced

    let mut node = tokens.first;

    while !node.is_null() {

        if let Some(scope_node) = find_next_scope(node) {
            let first_scope_element = unsafe { (*scope_node).right };
            let scope_end = find_scope_end(first_scope_element);

            // The node in the next iteration will be the node after the scope end
            node = unsafe { (*scope_end).right };
            
            // Don't parse empty scopes
            if first_scope_element == scope_end {
                // Remove the closing scope token
                tokens.extract_node(scope_end);
                continue;
            }

            let mut inner_scope = tokens.extract_slice(first_scope_element, scope_end);
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


/// Divide the scope token tree into a list of separate statements based on semicolons
fn divide_statements<'a>(mut tokens: TokenTree<'a>, symbol_table: &mut SymbolTable, parent_scope: Option<ScopeID>) -> ScopeBlock<'a> {

    let scope_id = symbol_table.add_scope(parent_scope);
    let mut block = ScopeBlock::new(scope_id);
    let mut node = tokens.first;

    while let Some(node_ref) = unsafe { node.as_mut() } {

        match node_ref.item.value {

            TokenKind::Semicolon => {
                // End of statement
                let mut statement = tokens.extract_slice(tokens.first, node);

                // Ignore empty statements (only has one semicolon token)
                if !statement.has_one_item() {
                    // Drop the semicolon token because it is not needed anymore
                    // The last token is guaranteed to be a semicolon because of it's the current node
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

                // Scopes are not their own statements: they are treated as expressions like in Rust.

                node = node_ref.right;
            },

            _ => node = node_ref.right,
        }
    }

    // Add any remaining tokens as the last statement
    if !tokens.is_empty() {
        block.statements.push(tokens);
    }
    
    block
}


fn parse_block_hierarchy(block: &mut ScopeBlock, symbol_table: &mut SymbolTable, source: &IRCode) {
    // Recursively parse the statements' hierarchy
    // Do not check the types of the operators yet. This will be done in the next pass when the symbol table is created.

    for statement in &mut block.statements {
        
        #[allow(unused_unsafe)] // A bug in the linter causes the below unsafe block to be marked as unnecessary, but removing it causes a compiler error
        while let Some(node) = find_highest_priority(statement)
            .and_then(|node_ptr| unsafe { node_ptr.as_mut() }) // Convert the raw pointer to a mutable reference
        {

            if node.item.priority == 0 {
                // No more operations to parse
                break;
            }
            // Set the priority to 0 so that the node is not visited again
            node.item.priority = 0;
    
            // Useful macros to get tokens without forgetting that the token pointers of extracted tokens are invalidated
            macro_rules! extract_left {
                () => {
                    statement.extract_node(node.left)
                };
            }
            macro_rules! extract_right {
                () => {
                    statement.extract_node(node.right)
                };
            }
    
            // Satisfy the operator requirements
            match node.item.value {
                TokenKind::Op(op) => match op {
    
                    // Binary operators:
                    Ops::Add |
                    Ops::Sub |
                    Ops::Mul |
                    Ops::Div |
                    Ops::Mod |
                    Ops::Assign |
                    Ops::Equal |
                    Ops::NotEqual |
                    Ops::Greater |
                    Ops::Less |
                    Ops::GreaterEqual |
                    Ops::LessEqual |
                    Ops::LogicalAnd |
                    Ops::LogicalOr |
                    Ops::BitShiftLeft |
                    Ops::BitShiftRight |
                    Ops::BitwiseOr |
                    Ops::BitwiseAnd |
                    Ops::BitwiseXor 
                     => {
                        let left = extract_left!().unwrap_or_else(
                            || error::expected_argument(node.item.unit_path, &node.item.value, node.item.token.line_number(), node.item.token.column, &source[node.item.token.line_index()], format!("Missing left argument for operator {}.", op).as_str())
                        );
                    
                        let right = extract_right!().unwrap_or_else(
                            || error::expected_argument(node.item.unit_path, &node.item.value, node.item.token.line_number(), node.item.token.column, &source[node.item.token.line_index()], format!("Missing right argument for operator {}.", op).as_str())
                        );
                    
                        node.children = Some(ChildrenType::Binary(left, right));
                    },
    
                    // Unary operators left:
                    Ops::Deref |
                    Ops::Ref |
                    Ops::LogicalNot |
                    Ops::BitwiseNot
                     => {
                        let left = extract_left!().unwrap_or_else(
                            || error::expected_argument(node.item.unit_path, &node.item.value, node.item.token.line_number(), node.item.token.column, &source[node.item.token.line_index()], format!("Missing left argument for operator {}.", op).as_str())
                        );
                        node.children = Some(ChildrenType::Unary(left));
                    },
    
                    // Unary operators right:
                    Ops::Jump => {
                        // Syntax: jump <expression>

                        let right = extract_right!().unwrap_or_else(
                            || error::expected_argument(node.item.unit_path, &node.item.value, node.item.token.line_number(), node.item.token.column, &source[node.item.token.line_index()], format!("Missing right argument for operator {}.", op).as_str())
                        );

                        node.children = Some(ChildrenType::Unary(right));
                    },
                    Ops::Return => {
                        // Syntax: return <expression>
                        // Syntax: return

                        extract_right!().map(
                            |expr| if !is_expression(&expr.item.value) {
                                error::invalid_argument(node.item.unit_path, &node.item.value, expr.item.token.line_number(), expr.item.token.column, &source[expr.item.token.line_index()], format!("Invalid expression {:?} after return operator.", expr.item.value).as_str());
                            } else { 
                                node.children = Some(ChildrenType::Unary(expr));
                            }
                        );
                    },
    
                    // Other operators:
                    Ops::FunctionCallOpen => {
                        // Functin call is a list of tokens separated by commas enclosed in parentheses
                        // Statements inside the parentheses have already been parsed into single top-level tokens because of their higher priority
                        
                        let callable = extract_left!().unwrap_or_else(
                            || error::expected_argument(node.item.unit_path, &node.item.value, node.item.token.line_number(), node.item.token.column, &source[node.item.token.line_index()], "Missing function name before function call operator.")
                        );
                        // Check for expression instead of only a symbol because expressions can evaluate to function pointers
                        if !is_expression(&callable.item.value) {
                            error::invalid_argument(node.item.unit_path, &node.item.value, callable.item.token.line_number(), callable.item.token.column, &source[callable.item.token.line_index()], format!("Invalid function name or function-returning expression {:?} before function call operator.", callable.item.value).as_str());
                        }

                        let args = extract_list_like_delimiter_contents(statement, node, &node.item.value, &TokenKind::ParClose, source);
                        if args.iter().any(|node| !is_expression(&node.item.value)) {
                            error::invalid_argument(node.item.unit_path, &node.item.value, node.item.token.line_number(), node.item.token.column, &source[node.item.token.line_index()], "Invalid argument in function call. Arguments must be expressions.");
                        }
                        
                        node.children = Some(ChildrenType::Call { callable, args });
                    },

                    Ops::ArrayIndexOpen => {
                        // Syntax: <expression>[<expression>]

                        let array_expression = extract_left!().unwrap(); // Unwrap because the tokenizer interprets `[` as array index only if the previous token is an expression
                        if !is_expression(&array_expression.item.value) {
                            error::invalid_argument(node.item.unit_path, &node.item.value, node.item.token.line_number(), node.item.token.column, &source[node.item.token.line_index()], format!("Expected an array-like expression before an array subscription operator. Got {:?}", array_expression.item.value).as_str());
                        }

                        let index_expression = extract_right!().unwrap(); // Must have a right node for brackets to be balanced
                        if !is_expression(&index_expression.item.value) {
                            error::invalid_argument(node.item.unit_path, &node.item.value, node.item.token.line_number(), node.item.token.column, &source[node.item.token.line_index()], format!("Invalid argument in array subscript operator. Expected an expression, got {:?}", index_expression.item.value).as_str());
                        }
                        
                        // Extract closing bracket
                        extract_right!().map(|node| if !matches!(node.item.value, TokenKind::SquareClose) {
                            error::expected_argument(node.item.unit_path, &node.item.value, node.item.token.line_number(), node.item.token.column, &source[node.item.token.line_index()], "Expected closing square bracket after expression in array subscription.")
                        }).unwrap(); // Unwrap because delimiters are guaranteed to be balanced by the tokenizer

                        node.children = Some(ChildrenType::Binary(array_expression, index_expression));
                    }
                    
                },
    
                TokenKind::ParOpen => {
                    // Syntax: (<expression>)

                    // Extract the next token (either an expression or a closing parenthesis)
                    match extract_right!() {

                        Some(next_node) => match &next_node.item.value {

                            // Empty parentheses are not allowed, as they would evaluate to a void value
                            TokenKind::ParClose => error::expected_argument(node.item.unit_path, &node.item.value, node.item.token.line_number(), node.item.token.column, &source[node.item.token.line_index()], "Empty parentheses are not allowed because they would evaluate to a void value."),
                                
                            tk if is_expression(tk) => {
                                // next_node is here guaranteed to be a value or an expression

                                // Check if the next token is a closing parenthesis (because the parentheses contain only one top-level node)
                                extract_right!().map(|node| if matches!(node.item.value, TokenKind::ParClose) {
                                    node
                                } else {
                                    error::invalid_argument(node.item.unit_path, &node.item.value, node.item.token.line_number(), node.item.token.column, &source[node.item.token.line_index()], format!("Invalid token {:?} in parentheses after inner expression. Expected a closing parenthesis ).", node.item.value).as_str())
                                }).unwrap(); // Unwrap because the tokenizer should guarantee that the parentheses are balanced, so there should be no missing closing parenthesis

                                // Transform this parenthesis node into its inner expression
                                node.substitute(*next_node)
                            },

                            _ => error::invalid_argument(node.item.unit_path, &node.item.value, node.item.token.line_number(), node.item.token.column, &source[node.item.token.line_index()], format!("Invalid token {:?} in parentheses. Expected an expression.", next_node.item.value).as_str())
                        },

                        None => unreachable!("Parenthesis open token has no right token. Tokenizer should have already caught this. This is a bug."),
                    }
                },
    
                TokenKind::ArrayOpen => {
                    // Extract the tokens within the square brackets
                    let inner_tokens = extract_list_like_delimiter_contents(statement, node, &node.item.value, &TokenKind::SquareClose, source);
                    
                    // Get the inner tokens value and check if they really are value-holding tokens
                    let inner_tokens: Vec<TokenNode> = inner_tokens.into_iter().map(
                        |node| if is_expression(&node.item.value) {
                            node
                        } else {
                            error::invalid_argument(node.item.unit_path, &node.item.value, node.item.token.line_number(), node.item.token.column, &source[node.item.token.line_index()], format!("Invalid token {:?} in array declaration. Expected an expression.", node.item.value).as_str())
                        }
                    ).collect();
                    
                    node.children = Some(ChildrenType::List(inner_tokens));
                },
    
                TokenKind::ScopeOpen => {
                    // Parse the children statements of the scope, if any.
                    // The children have already been extracted and divided into separate statements.
                    
                    if let Some(ChildrenType::Block(statements)) = &mut node.children {
                        parse_block_hierarchy(statements, symbol_table, source);
                    }
                },
    
                TokenKind::Let => {
                    // TODO: add support for symbol type inference based on the assigned value, if any
                    // Syntax: let [mut] <name>: <type> 
    
                    // This node can either be the symbol name or the mut keyword
                    let next_node = extract_right!().unwrap_or_else(
                        || error::expected_argument(node.item.unit_path, &node.item.value, node.item.token.line_number(), node.item.token.column, &source[node.item.token.line_index()], "Missing variable name after let in variable declaration.")
                    );
    
                    let (mutable, name_token) = if matches!(next_node.item.value, TokenKind::Mut) {
                        let name = extract_right!().unwrap_or_else(
                            || error::expected_argument(node.item.unit_path, &node.item.value, node.item.token.line_number(), node.item.token.column, &source[node.item.token.line_index()], "Missing variable name after let in variable declaration.")
                        );
                        (true, name)
                    } else {
                        (false, next_node)
                    };
    
                    let symbol_name: &str = if let TokenKind::Value(Value::Symbol { id }) = &name_token.item.value {
                        id
                    } else {
                        error::invalid_argument(node.item.unit_path, &node.item.value, node.item.token.line_number(), node.item.token.column, &source[node.item.token.line_index()], format!("Invalid variable name {:?} in variable declaration.", name_token.item.value).as_str())
                    };
    
                    let _colon = extract_right!().unwrap_or_else(
                        || error::expected_argument(node.item.unit_path, &node.item.value, node.item.token.line_number(), node.item.token.column, &source[node.item.token.line_index()], "Missing colon after variable name in variable declaration.")
                    );
    
                    // The data type should be a single top-level token because of its higher priority
                    let data_type_node = extract_right!().unwrap_or_else(
                        || error::expected_argument(node.item.unit_path, &node.item.value, node.item.token.line_number(), node.item.token.column, &source[node.item.token.line_index()], "Missing data type after colon in variable declaration.")
                    );
                    let data_type = if let TokenKind::DataType(data_type) = data_type_node.item.value {
                        data_type
                    } else {
                        error::invalid_argument(node.item.unit_path, &node.item.value, node.item.token.line_number(), node.item.token.column, &source[node.item.token.line_index()], format!("Invalid data type {:?} in variable declaration.", data_type_node.item.value).as_str())
                    };

                    // Declare the new symbol in the local scope
                    symbol_table.declare_symbol(
                        Symbol::new(symbol_name.to_string(), data_type, mutable),
                        block.scope_id
                    );
    
                    // Transform this node into a symbol node (the declatataor let is no longer needed since the symbol is already declared)
                    node.substitute(*name_token);
                },
    
                TokenKind::Fn => {
                    // Function declaration syntax:
                    // fn <name>(<arguments>) -> <return type> { <body> }
                    // fn <name>(argumenta>) { <body> }

                    // Default return type is void, unless specified by the arrow ->
                    let mut return_type = DataType::Void;
    
                    let name_node = extract_right!().unwrap_or_else(
                        || error::expected_argument(node.item.unit_path, &node.item.value, node.item.token.line_number(), node.item.token.column, &source[node.item.token.line_index()], "Missing function name after fn in function declaration.")
                    );
                    let function_name: &str = if let TokenKind::Value(Value::Symbol { id }) = name_node.item.value {
                        id
                    } else {
                        error::invalid_argument(node.item.unit_path, &node.item.value, node.item.token.line_number(), node.item.token.column, &source[node.item.token.line_index()], format!("Invalid function name {:?} in function declaration.", name_node.item.value).as_str())
                    };
    
                    // The parameters should be a single top-level token because of its higher priority
                    let params = extract_right!().unwrap_or_else(
                        || error::expected_argument(node.item.unit_path, &node.item.value, node.item.token.line_number(), node.item.token.column, &source[node.item.token.line_index()], "Missing arguments after function name in function declaration.")
                    );
                    let params = if matches!(params.item.value, TokenKind::FunctionParamsOpen) {
                        if let Some(ChildrenType::FunctionParams(params)) = params.children {
                            params
                        } else {
                            unreachable!("Invalid token kind during statement hierarchy parsing: {:?}. This token kind should be a FunctionParamsOpen token and have FunctionParams children.", params.item.value)
                        }
                    } else {
                        error::invalid_argument(node.item.unit_path, &node.item.value, node.item.token.line_number(), node.item.token.column, &source[node.item.token.line_index()], format!("Invalid parameter declaration {:?} in function declaration. Function parameters must be enclosed in parentheses ().", params.item.value).as_str())
                    };
                    
                    // Extract the arrow -> or the function body
                    let node_after_params = extract_right!().unwrap_or_else(
                        || error::expected_argument(node.item.unit_path, &node.item.value, node.item.token.line_number(), node.item.token.column, &source[node.item.token.line_index()], "Missing arrow or function body after function parameters in function declaration.")
                    );

                    // Check if it's an arrow -> or a function body, return the function body
                    let body_node = if matches!(node_after_params.item.value, TokenKind::Arrow) {
                        // Extract the function return type
                        let return_type_node = extract_right!().unwrap_or_else(
                            || error::expected_argument(node.item.unit_path, &node.item.value, node.item.token.line_number(), node.item.token.column, &source[node.item.token.line_index()], "Missing return type after arrow in function declaration.")
                        );
                        if let TokenKind::DataType(data_type) = return_type_node.item.value {
                            return_type = data_type;
                        } else {
                            error::invalid_argument(node.item.unit_path, &node.item.value, return_type_node.item.token.line_number(), return_type_node.item.token.column, &source[return_type_node.item.token.line_index()], format!("Invalid return type {:?} in function declaration.", return_type_node.item.value).as_str());
                        }

                        // The body node is the one after the return type
                        extract_right!()
                    } else {
                        // If there is no arrow ->, the node_after_params is the function body
                       Some(node_after_params)
                    };

                    // The body is one top-level scope token because it gets parsed first
                    let body: ScopeBlock = body_node.map(
                        |node| if matches!(node.item.value, TokenKind::ScopeOpen) {
                            if let Some(ChildrenType::Block(body)) = node.children {
                                body
                            } else {
                                unreachable!("Invalid token kind during statement hierarchy parsing: {:?}. This token kind should be a ScopeOpen token and have Block children.", node.item.value)
                            }
                        } else {
                            error::invalid_argument(node.item.unit_path, &node.item.value, node.item.token.line_number(), node.item.token.column, &source[node.item.token.line_index()], format!("Invalid body {:?} in function declaration. Function body must be enclosed in curly braces {{}}.", node.item.value).as_str())
                        }
                    ).unwrap_or_else(
                        || error::expected_argument(node.item.unit_path, &node.item.value, node.item.token.line_number(), node.item.token.column, &source[node.item.token.line_index()], "Missing body after return type in function declaration.")
                    );
                    
                    let function_type = DataType::Function { 
                        params: params.iter().map(|param| param.1.clone()).collect(), // Take only the data type of the parameter
                        return_type: Box::new(return_type.clone())
                    };
                    symbol_table.declare_symbol(
                        Symbol::new(function_name.to_string(), function_type, false), // mutable = false because functions are not mutable
                        block.scope_id
                    );

                    node.children = Some(ChildrenType::Function { name: function_name, params, return_type, body });
                },

                TokenKind::FunctionParamsOpen => {
                    // Syntax: (<name>: <type>, <name>: <type>, ...)

                    let mut params: Vec<(String, DataType)> = Vec::new();

                    let mut expected_comma: bool = false;
                    loop {
                        let param_node = extract_right!().unwrap_or_else(
                            || error::expected_argument(node.item.unit_path, &node.item.value, node.item.token.line_number(), node.item.token.column, &source[node.item.token.line_index()], format!("Missing parameter or closing delimiter for operator {:?}.", node.item.value).as_str())
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
                                        error::invalid_argument(param_node.item.unit_path, &node.item.value, param_node.item.token.line_number(), param_node.item.token.column, &source[param_node.item.token.line_index()], format!("Invalid token {:?} in function declaration. Expected a colon : after the parameter name.", param_node.item.value).as_str());
                                    }
                                ).unwrap_or_else(
                                    || error::expected_argument(node.item.unit_path, &node.item.value, node.item.token.line_number(), node.item.token.column, &source[node.item.token.line_index()], format!("Missing colon after parameter name {:?} in function declaration.", name).as_str())
                                );

                                let data_type = extract_right!().unwrap_or_else(
                                    || error::expected_argument(node.item.unit_path, &node.item.value, node.item.token.line_number(), node.item.token.column, &source[node.item.token.line_index()], "Missing data type after colon in function declaration.")
                                );
                                let data_type = if let TokenKind::DataType(data_type) = data_type.item.value {
                                    data_type
                                } else {
                                    error::invalid_argument(param_node.item.unit_path, &node.item.value, param_node.item.token.line_number(), param_node.item.token.column, &source[param_node.item.token.line_index()], format!("Invalid data type {:?} in function declaration.", data_type.item.value).as_str())
                                };

                                params.push((name, data_type));

                                // A comma is expected after each argument except the last one
                                expected_comma = true;
                            },

                            _ => unreachable!("Invalid token kind during statement hierarchy parsing: {:?}. This token kind shouldn't have children.", param_node.item.value)
                        }
                    }

                    node.children = Some(ChildrenType::FunctionParams(params));
                },

                TokenKind::ArrayTypeOpen => {
                    // TODO: an array slice may be implemented at a later date. this array type would then require a size (or infer it from the context)
                    // Syntax: [<type>]

                    let element_type = extract_right!().map(
                        |node| if let TokenKind::DataType(data_type) = node.item.value {
                            data_type
                        } else {
                            error::invalid_argument(node.item.unit_path, &node.item.value, node.item.token.line_number(), node.item.token.column, &source[node.item.token.line_index()], format!("Invalid data type {:?} in array declaration.", node.item.value).as_str())
                        }
                    ).unwrap_or_else(
                        || error::expected_argument(node.item.unit_path, &node.item.value, node.item.token.line_number(), node.item.token.column, &source[node.item.token.line_index()], "Missing data type after array type open bracket in array declaration.")
                    );
                    
                    // Extract the closing square bracket ]
                    extract_right!().map(
                        |node| if matches!(node.item.value, TokenKind::SquareClose) {
                            node
                        } else {
                            error::invalid_argument(node.item.unit_path, &node.item.value, node.item.token.line_number(), node.item.token.column, &source[node.item.token.line_index()], format!("Invalid token {:?} in array declaration. Expected a closing bracket ] after the array type.", node.item.value).as_str())
                        }
                    ).unwrap_or_else(
                        || error::expected_argument(node.item.unit_path, &node.item.value, node.item.token.line_number(), node.item.token.column, &source[node.item.token.line_index()], "Missing array type close bracket after array type in array declaration.")
                    );
                    
                    let array_type = DataType::Array(Box::new(element_type));
                    // Transform this node into a data type node
                    node.item.value = TokenKind::DataType(array_type);
                },  

                TokenKind::RefType => {
                    // Syntax: &<type>

                    let element_type = extract_right!().map(
                        |node| if let TokenKind::DataType(data_type) = node.item.value {
                            data_type
                        } else {
                            error::invalid_argument(node.item.unit_path, &node.item.value, node.item.token.line_number(), node.item.token.column, &source[node.item.token.line_index()], format!("Invalid data type {:?} in reference declaration.", node.item.value).as_str())
                        }
                    ).unwrap_or_else(
                        || error::expected_argument(node.item.unit_path, &node.item.value, node.item.token.line_number(), node.item.token.column, &source[node.item.token.line_index()], "Missing data type after reference type symbol in reference declaration.")
                    );
                    
                    let ref_type = DataType::Ref(Box::new(element_type));
                    // Transform this node into a data type node
                    node.item.value = TokenKind::DataType(ref_type);
                },

                TokenKind::As => {
                    // Syntax: <expression> as <type>

                    let expr = extract_left!().unwrap_or_else(
                        || error::expected_argument(node.item.unit_path, &node.item.value, node.item.token.line_number(), node.item.token.column, &source[node.item.token.line_index()], format!("Missing expression before operator {:?}.", node.item.value).as_str())
                    );
                    if !is_expression(&expr.item.value) {
                        error::invalid_argument(node.item.unit_path, &node.item.value, expr.item.token.line_number(), expr.item.token.column, &source[expr.item.token.line_index()], format!("Invalid expression {:?} before operator {:?}.", expr.item.value, node.item.value).as_str());
                    }
                    
                    let data_type = extract_right!().map(
                        |node| if let TokenKind::DataType(data_type) = node.item.value {
                            data_type
                        } else {
                            error::invalid_argument(node.item.unit_path, &node.item.value, node.item.token.line_number(), node.item.token.column, &source[node.item.token.line_index()], format!("Invalid data type {:?} in type cast.", node.item.value).as_str())
                        }
                    ).unwrap_or_else(
                        || error::expected_argument(node.item.unit_path, &node.item.value, node.item.token.line_number(), node.item.token.column, &source[node.item.token.line_index()], "Missing data type after type cast operator in type cast.")
                    );

                    node.children = Some(ChildrenType::TypeCast { data_type, expr });
                },
                
                _ => unreachable!("Invalid token kind during statement hierarchy parsing: {:?}. This token kind shouldn't have children.", node.item.value)
            }
        }

    }
}


/// Extract comma-separated tokens within a delimiter (parentheses, square brackets, etc.).
/// 
/// Removes the closing delimiter from the token list without including it in the returned arguments.
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


/// Find the token node with the highest priority in the uppermost layer of the tree.
fn find_highest_priority<'a>(tokens: &TokenTree<'a>) -> Option<*mut TokenNode<'a>> {

    let mut highest_priority: Option<&TokenNode> = None;

    for node in tokens.iter() {
        if let Some(hp) = highest_priority {
            if node.item.priority > hp.item.priority {
                highest_priority = Some(node);
            }
        } else {
            highest_priority = Some(node);
        }
    }

    // Convert the immutable reference to a mutable pointer
    highest_priority.map(|node| node as *const TokenNode as *mut TokenNode)
}


/// Recursively resolve the type of this expression and check if its children have the correct types.
fn resolve_expression_types(expression: &mut TokenNode, scope_id: ScopeID, outer_function_return: Option<&DataType>, symbol_table: &SymbolTable, source: &IRCode) {

    expression.data_type = match &expression.item.value {

        TokenKind::Op(operator) => {
            // Resolve and check the types of the operands first
            // Based on the operand values, determine the type of the operator
            
            match operator {

                Ops::Deref => if let Some(ChildrenType::Unary(operand)) = &mut expression.children {

                    resolve_expression_types(operand, scope_id, outer_function_return, symbol_table, source);

                    if let DataType::Ref(data_type) = &operand.data_type {
                        *data_type.clone()
                    } else {
                        error::type_error(expression.item.unit_path, &[DataType::Ref(Box::new(DataType::Void)).name()], &operand.data_type, operand.item.token.line_number(), operand.item.token.column, &source[operand.item.token.line_index()], format!("Invalid type {:?} for operator {:?}. Expected a reference or an expression that evaluates to a reference.", operand.data_type, operator).as_str());
                    }

                } else {
                    unreachable!()
                },

                Ops::Ref => if let Some(ChildrenType::Unary(operand)) = &mut expression.children {

                    resolve_expression_types(operand, scope_id, outer_function_return, symbol_table, source);

                    DataType::Ref(Box::new(operand.data_type.clone()))

                } else {
                    unreachable!()
                },

                // Binary operators that return a boolean
                Ops::Equal |
                Ops::NotEqual |
                Ops::Greater |
                Ops::Less |
                Ops::GreaterEqual |
                Ops::LessEqual |
                Ops::LogicalAnd |
                Ops::LogicalOr 
                 => if let Some(ChildrenType::Binary(op1, op2)) = &mut expression.children {

                    resolve_expression_types(op1, scope_id, outer_function_return, symbol_table, source);
                    resolve_expression_types(op2, scope_id, outer_function_return, symbol_table, source);

                    if !operator.is_allowed_type(&op1.data_type, 0) {
                        error::type_error(expression.item.unit_path, operator.allowed_types(0), &op1.data_type, op1.item.token.line_number(), op1.item.token.column, &source[op1.item.token.line_index()], format!("Invalid type {:?} for operator {:?}.", op1.data_type, operator).as_str());
                    }
                    if !operator.is_allowed_type(&op2.data_type, 1) {
                        error::type_error(expression.item.unit_path, operator.allowed_types(1), &op2.data_type, op2.item.token.line_number(), op2.item.token.column, &source[op2.item.token.line_index()], format!("Invalid type {:?} for operator {:?}.", op2.data_type, operator).as_str());
                    }

                    // Operands must have the same type
                    if op1.data_type != op2.data_type {
                        error::type_error(expression.item.unit_path, &[op1.data_type.name()], &op2.data_type, op2.item.token.line_number(), op2.item.token.column, &source[op2.item.token.line_index()], format!("Operator {:?} has operands of different types {:?} and {:?}.", operator, op1.data_type, op2.data_type).as_str());
                    }

                    DataType::Bool
                } else {
                    unreachable!();
                },

                // Unary operators that return a boolean
                Ops::LogicalNot => if let Some(ChildrenType::Unary(operand)) = &mut expression.children {

                    resolve_expression_types(operand, scope_id, outer_function_return, symbol_table, source);

                    if !operator.is_allowed_type(&operand.data_type, 0) {
                        error::type_error(expression.item.unit_path, operator.allowed_types(0), &operand.data_type, operand.item.token.line_number(), operand.item.token.column, &source[operand.item.token.line_index()], format!("Invalid type {:?} for operator {:?}.", operand.data_type, operator).as_str());
                    }

                    DataType::Bool
                } else {
                    unreachable!();
                },

                // Unary operators whose return type is the same as the operand type
                Ops::BitwiseNot => if let Some(ChildrenType::Unary(operand)) = &mut expression.children {

                    resolve_expression_types(operand, scope_id, outer_function_return, symbol_table, source);

                    if !operator.is_allowed_type(&operand.data_type, 0) {
                        error::type_error(expression.item.unit_path, operator.allowed_types(0), &operand.data_type, operand.item.token.line_number(), operand.item.token.column, &source[operand.item.token.line_index()], format!("Invalid type {:?} for operator {:?}.", operand.data_type, operator).as_str());
                    }

                    operand.data_type.clone()
                } else {
                    unreachable!();
                },

                // Binary operators whose return type is the same as the operand type
                Ops::Add |
                Ops::Sub |
                Ops::Mul |
                Ops::Div |
                Ops::Mod |
                Ops::BitShiftLeft |
                Ops::BitShiftRight |
                Ops::BitwiseOr |
                Ops::BitwiseAnd |
                Ops::BitwiseXor 
                 => if let Some(ChildrenType::Binary(op1, op2)) = &mut expression.children {

                    resolve_expression_types(op1, scope_id, outer_function_return, symbol_table, source);
                    resolve_expression_types(op2, scope_id, outer_function_return, symbol_table, source);

                    if !operator.is_allowed_type(&op1.data_type, 0) {
                        error::type_error(expression.item.unit_path, operator.allowed_types(0), &op1.data_type, op1.item.token.line_number(), op1.item.token.column, &source[op1.item.token.line_index()], format!("Invalid type {:?} for operator {:?}.", op1.data_type, operator).as_str());
                    }
                    if !operator.is_allowed_type(&op2.data_type, 1) {
                        error::type_error(expression.item.unit_path, operator.allowed_types(1), &op2.data_type, op2.item.token.line_number(), op2.item.token.column, &source[op2.item.token.line_index()], format!("Invalid type {:?} for operator {:?}.", op2.data_type, operator).as_str());
                    }

                    // Check if the operands have the same type
                    if op1.data_type != op2.data_type {
                        // Here ot.clone() is acceptable because the program will exit after this error
                        error::type_error(expression.item.unit_path, &[op1.data_type.name()], &op2.data_type, op2.item.token.line_number(), op2.item.token.column, &source[op2.item.token.line_index()], format!("Operator {:?} has operands of different types {:?} and {:?}.", operator, op1.data_type, &op2.data_type).as_str());
                    }
                    
                    op1.data_type.clone()
                } else {
                    unreachable!();
                },

                Ops::FunctionCallOpen => if let Some(ChildrenType::Call { callable, args }) = &mut expression.children {
                    
                    // Resolve the type of the callable operand
                    resolve_expression_types(callable, scope_id, outer_function_return, symbol_table, source);

                    // Check if the callable operand is indeed callable (a function symbol or a function pointer)
                    let (param_types, return_type): (&[DataType], &DataType) = match &callable.data_type {

                        DataType::Function { params, return_type } => (params, return_type),
                        DataType::Ref(dt) if matches!(**dt, DataType::Function { .. }) => if let DataType::Function { params, return_type } = &**dt {
                            (params, return_type)
                        } else {
                            unreachable!("Invalid data type during expression type resolution: {:?}. This is a bug.", dt)
                        },

                        _ => error::type_error(expression.item.unit_path, &[DataType::Function { params: Vec::new(), return_type: Box::new(DataType::Void) }.name()], &callable.data_type, callable.item.token.line_number(), callable.item.token.column, &source[callable.item.token.line_index()], format!("Invalid type {:?} for operator {:?}. Expected a function.", callable.data_type, operator).as_str())
                    };

                    // Check if the number of arguments matches the number of parameters
                    // Check this before resolving the types of the arguments to avoid unnecessary work
                    if args.len() != param_types.len() {
                        error::type_error(expression.item.unit_path, &[DataType::Function { params: Vec::new(), return_type: Box::new(DataType::Void) }.name()], &callable.data_type, callable.item.token.line_number(), callable.item.token.column, &source[callable.item.token.line_index()], format!("Invalid number of arguments for function {:?}. Expected {} arguments, but got {}.", callable.data_type, param_types.len(), args.len()).as_str());
                    }                    

                    // Resolve the types of the arguments and check if they match the function parameters
                    for (arg, expected_type) in args.iter_mut().zip(param_types) {
                        resolve_expression_types(arg, scope_id, outer_function_return, symbol_table, source);

                        if arg.data_type != *expected_type {
                            error::type_error(expression.item.unit_path, &[expected_type.name()], &arg.data_type, arg.item.token.line_number(), arg.item.token.column, &source[arg.item.token.line_index()], format!("Invalid type {:?} for argument. Expected {:?}.", arg.data_type, expected_type).as_str());
                        }
                    }

                    // The return type of the function call is the return type of the function
                    return_type.clone()
                } else {
                    unreachable!("Operator {:?} from expression {:?} should have children of type ChildrenType::Call, but the expression has children of type {:?} instead. This is a bug.", operator, expression, expression.children)
                },

                Ops::Return => {

                    // A return statement is only allowed inside a function
                    let return_type = outer_function_return.unwrap_or_else(
                        || error::syntax_error(expression.item.unit_path, expression.item.token.line_number(), expression.item.token.column, &source[expression.item.token.line_index()], "Return statement is only allowed inside a function.")
                    );

                    // Resolve the type of the return value, if any
                    if let Some(children) = &mut expression.children {

                        let return_expr = if let ChildrenType::Unary (children) = children { children } else { unreachable!(); };

                        resolve_expression_types(return_expr, scope_id, outer_function_return, symbol_table, source);
                        
                        // Check if the return type matches the outer function return type
                        if return_expr.data_type != *return_type {
                            error::type_error(expression.item.unit_path, &[return_type.name()], &return_expr.data_type, return_expr.item.token.line_number(), return_expr.item.token.column, &source[return_expr.item.token.line_index()], format!("Invalid return type {:?}. Expected {:?}.", return_expr.data_type, return_type).as_str());
                        }
                    } else if !matches!(return_type, DataType::Void) {
                        // Check if the return statements is missing a return value when the function return type is not void
                        error::type_error(expression.item.unit_path, &[return_type.name()], &DataType::Void, expression.item.token.line_number(), expression.item.token.column, &source[expression.item.token.line_index()], format!("Invalid return type {:?}. Expected {:?}.", DataType::Void, return_type).as_str());
                    }

                    // A return statement evaluates to void
                    DataType::Void
                },

                Ops::Assign => {
                    
                    if let Some(ChildrenType::Binary (l_node, r_node)) = &mut expression.children {

                        // Assume the operands vector has only two elements: the left operand and the right operand

                        // Only allow assignment to a symbol or a dereference
                        if !matches!(l_node.item.value, TokenKind::Value(Value::Symbol { .. }) | TokenKind::Op(Ops::Deref)) {
                            error::type_error(expression.item.unit_path, operator.allowed_types(0), &l_node.data_type, l_node.item.token.line_number(), l_node.item.token.column, &source[l_node.item.token.line_index()], format!("Invalid type {:?} for left operand of operator {:?}. Expected a symbol or a dereference.", l_node.data_type, operator).as_str());
                        }

                        // Resolve the types of the operands
                        resolve_expression_types(l_node, scope_id, outer_function_return, symbol_table, source);
                        resolve_expression_types(r_node, scope_id, outer_function_return, symbol_table, source);

                        // TODO: Check if the left operand is mutable. Add a "initialized" field for the Symbol struct in the symbol table

                        // Check if the symbol type and the expression type are compatible (the same or implicitly castable)
                        let r_value = r_node.item.value.literal_value();
                        if !r_node.data_type.is_implicitly_castable_to(&l_node.data_type, r_value) {
                            error::type_error(expression.item.unit_path, &[l_node.data_type.name()], &r_node.data_type, r_node.item.token.line_number(), r_node.item.token.column, &source[r_node.item.token.line_index()], format!("Invalid type {:?} for right operand of operator {:?}. Expected {:?}.", r_node.data_type, operator, l_node.data_type).as_str());
                        }
                    } else {
                        unreachable!()
                    };
                    
                    // An assignment is not an expression, so it does not have a type
                    DataType::Void
                },

                Ops::ArrayIndexOpen => {
                    // The data type of an array subscription operation is the type of the array elements

                    let data_type: DataType;

                    if let Some(ChildrenType::Binary (array_node, index_node )) = &mut expression.children {

                        resolve_expression_types(array_node, scope_id, outer_function_return, symbol_table, source);
                        resolve_expression_types(index_node, scope_id, outer_function_return, symbol_table, source);

                        if let DataType::Array(element_type) = &array_node.data_type {
                            data_type = *element_type.clone();
                        } else {
                            error::type_error(expression.item.unit_path, &["array-like"], &array_node.data_type, array_node.item.token.line_number(), array_node.item.token.column, &source[array_node.item.token.line_index()], format!("Type must be an array-like expression. Cannot index {:?}", array_node.data_type).as_str());
                        }

                        // Assert that the array index is an unsigned integer
                        if !matches!(&index_node.data_type, unsigned_integer_pattern!()) {
                            error::type_error(expression.item.unit_path, &["unsigned integer"], &index_node.data_type, index_node.item.token.line_number(), index_node.item.token.column, &source[index_node.item.token.line_index()], "Array index must strictly be an unsigned integer.");
                        }
                    } else {
                        unreachable!()
                    }

                    data_type
                }

                _ => {
                    // Assert that the unmatched operator does indeed not return a value
                    assert!(!operator.returns_a_value(), "Operator {:?} from expression {:?} returns a value. This is a bug.", operator, expression);

                    DataType::Void
                }
            }
        },

        TokenKind::As => if let Some(ChildrenType::TypeCast { data_type, expr }) = &mut expression.children {

            // Resolve the type of the expression to be cast
            resolve_expression_types(expr, scope_id, outer_function_return, symbol_table, source);

            // Check if the expression type can be cast to the specified type
            if !data_type.is_castable_to(&expr.data_type) {
                error::type_error(expression.item.unit_path, &[data_type.name()], &expr.data_type, expr.item.token.line_number(), expr.item.token.column, &source[expr.item.token.line_index()], format!("Invalid type {:?} for type cast. Expected {:?}.", expr.data_type, data_type).as_str());
            }

            // Evaluates to the data type of the type cast
            data_type.clone()
        } else {
            unreachable!("Operator {:?} from expression {:?} should have children of type ChildrenType::TypeCast, but the expression has children of type {:?} instead. This is a bug.", TokenKind::As, expression, expression.children)
        },

        TokenKind::Value(value) => match value {

            Value::Literal { value } => value.data_type(),

            Value::Symbol { id } => symbol_table.get(scope_id, id)
                .unwrap_or_else(|| error::symbol_undefined(expression.item.unit_path, id, expression.item.token.line_number(), expression.item.token.column, &source[expression.item.token.line_index()], "This symbol is not defined in this scope."))
                .data_type.clone(),
        },

        TokenKind::Fn => {
            // Resolve the types inside the function body

            if let Some(ChildrenType::Function { return_type, body, .. }) = &mut expression.children {
                resolve_scope_types(body, Some(return_type), symbol_table, source);
            }

            // Function declaration does not have any type
            DataType::Void
        },

        TokenKind::ArrayOpen => {
            // Recursively resolve the types of the array elements.
            // The array element type is the type of the first element.
            // Check if the array elements have the same type.
            // The array element type is void if the array is empty. A void array can be used as a generic array by assignment operators.

            let (data_type, is_literal_array, element_type) = if let Some(ChildrenType::List(elements)) = &mut expression.children {

                if elements.is_empty() {
                    (DataType::Array(Box::new(DataType::Void)), true, DataType::Void)
                } else {

                    let mut element_type: Option<&DataType> = None;

                    let mut is_literal_array = true;
                    for element in elements {
                        
                        // Resolve the element type
                        resolve_expression_types(element, scope_id, outer_function_return, symbol_table, source);
                        let expr_type = &element.data_type;

                        if let Some(et) = element_type {
                            if et != expr_type {
                                error::type_error(element.item.unit_path, &[et.name()], expr_type, element.item.token.line_number(), element.item.token.column, &source[element.item.token.line_index()], format!("Array elements have different types {:?} and {:?}.", et, expr_type).as_str());
                            }
                        } else {
                            // The first element of the array determines the array type
                            element_type = Some(expr_type);
                        }

                        // Check if the array elements are literals
                        if !matches!(element.item.value, TokenKind::Value(Value::Literal { .. })) {
                            is_literal_array = false;
                        }
                    }

                    (DataType::Array(Box::new(element_type.unwrap().clone())), is_literal_array, element_type.unwrap().clone())
                }
            } else { 
                unreachable!();
            };

            if is_literal_array {
                // If all the array elements are literals, the whole array is a literal array
                // Change this token to a literal array
                let elements = expression.children.take().map(|children| if let ChildrenType::List(elements) = children { elements } else { unreachable!("ArrayOpen node should have children of type ChildrenType::List, but the expression {:?} has children of type {:?} instead. This is a bug.", expression, expression.children) }).unwrap();
                let mut literal_items: Vec<LiteralValue> = Vec::with_capacity(elements.len());
                for element in elements {
                    literal_items.push(if let TokenKind::Value(Value::Literal { value }) = element.item.value { value } else { unreachable!("ArrayOpen node should have children of type ChildrenType::List, but the expression {:?} has children of type {:?} instead. This is a bug.", expression, expression.children) });
                }
                expression.item.value = TokenKind::Value(Value::Literal { value: LiteralValue::Array { element_type, items: literal_items } });
            }

            data_type
        },

        TokenKind::ScopeOpen => {
            // Recursively resolve the types of the children statements
            // Determine the type of the scope based on the type of the last statement
            // If the scope is empty (has no children), it evaluates to void

            if let Some(ChildrenType::Block(inner_block)) = &mut expression.children {
                resolve_scope_types(inner_block, outer_function_return, symbol_table, source);
                
                assert!(inner_block.statements.last().is_some(), "Scope blocks must have at least one statement. This is a bug.");
                let last_statement = inner_block.statements.last().unwrap();

                assert!(last_statement.last_node().is_some(), "Statements cannot be empty. This is a bug.");
                last_statement.last_node().unwrap().data_type.clone()
            } else {
                DataType::Void
            }
        },

        _ => unreachable!("Unexpected syntax node during expression and symbol type resolution: {:?}. This is a bug.", expression)
    };
}


/// Resolve and check the types of symbols and expressions.
fn resolve_scope_types(block: &mut ScopeBlock, outer_function_return: Option<&DataType>, symbol_table: &SymbolTable, source: &IRCode) {
    // Perform a depth-first traversal of the scope tree to determine the types in a top-to-bottom order (relative to the source code).
    // For every node in every scope, determine the node data type and check if it matches the expected type.

    for statement in &mut block.statements {

        let mut node_ptr = statement.first;
        while let Some(node) = unsafe { node_ptr.as_mut() } {

            resolve_expression_types(node, block.scope_id, outer_function_return, symbol_table, source);

            node_ptr = node.right;
        }

    }
}


/// Build an abstract syntax tree from a flat list of tokens and create a symbol table.
pub fn build_ast<'a>(mut tokens: TokenTree<'a>, source: &IRCode) -> (ScopeBlock<'a>, SymbolTable) {

    parse_scope_hierarchy(&mut tokens);

    let mut symbol_table = SymbolTable::new();

    let mut outer_block = divide_statements(tokens, &mut symbol_table, None);

    println!("Statements after division:\n{}", outer_block);

    parse_block_hierarchy(&mut outer_block, &mut symbol_table, source);

    println!("\n\nStatement hierarchy:\n{}", outer_block);

    resolve_scope_types(&mut outer_block, None, &symbol_table, source);

    println!("\n\nAfter symbol resolution:\n{}", outer_block);

    // 

    (outer_block, symbol_table)
}

