use std::rc::Rc;

use rusty_vm_lib::ir::SourceCode;

use super::{FunctionParam, ScopeBlock, SyntaxNode, SyntaxNodeValue, UnparsedScopeBlock, RuntimeOp, IfBlock};

use crate::lang::data_types::DataType;
use crate::lang::error;
use crate::tokenizer::{Token, TokenKind, TokenPriority, Value, TokenParsingList, TokenParsingNode, TokenParsingNodeValue};
use crate::symbol_table::{NameType, ScopeDiscriminant, ScopeID, Symbol, SymbolTable, SymbolValue};
use crate::match_or;


fn find_next_scope(tokens: *mut TokenParsingNode) -> Option<*mut TokenParsingNode> {
    let mut node = tokens;

    while !node.is_null() {

        let token = unsafe { (*node).assume_lex_token() };
        
        if matches!(token.value, TokenKind::ScopeOpen { .. }) {
            return Some(node);
        }

        node = unsafe { (*node).right() };
    }

    None
}


fn find_scope_end(tokens: *mut TokenParsingNode) -> *mut TokenParsingNode {

    let mut scope_depth: usize = 1;

    let mut node = tokens;
    while !node.is_null() {

        let token = unsafe { (*node).assume_lex_token() };

        match token.value {
            TokenKind::ScopeOpen { .. } => scope_depth += 1,
            TokenKind::ScopeClose => scope_depth -= 1,
            _ => (),
        }

        if scope_depth == 0 {
            return node;
        }

        node = unsafe { (*node).right() };
    }

    unreachable!("Scope end not found. This is a bug.");
}


/// Recursively parse the tokens into a hierarchical tree structure based on nested scopes.
/// 
/// The contents of the scopes are moved into the children of the scope tokens.
fn parse_scope_hierarchy(tokens: &mut TokenParsingList<'_>) {
    // This function shouldn't fail because the tokenizer has already checked that the scopes are balanced

    let mut node = unsafe { tokens.head() };

    while !node.is_null() {

        if let Some(scope_node) = find_next_scope(node) {
            let first_scope_element = unsafe { (*scope_node).right() };
            let scope_end = find_scope_end(first_scope_element);

            // The node in the next iteration will be the node after the scope end
            node = unsafe { (*scope_end).right() };
            
            // Don't parse empty scopes
            if first_scope_element == scope_end {
                // Remove the closing scope token
                unsafe { tokens.extract_node(scope_end) };

                // Set the scope node to an empty scope for consistency
                let scope_node = unsafe { scope_node.as_mut().unwrap() };

                scope_node.value = TokenParsingNodeValue::RawScope {
                    inner_tokens: TokenParsingList::new(),
                    token: scope_node.extract_value().assume_lex_token(),
                };

                continue;
            }

            let mut inner_scope = unsafe { tokens.extract_slice(first_scope_element, scope_end) };
            // Remove the closing scope token
            inner_scope.drop_last();
            
            // Recursively parse the inner scope hierarchy
            parse_scope_hierarchy(&mut inner_scope);

            let scope_node = unsafe { scope_node.as_mut().unwrap() };
            
            scope_node.value = TokenParsingNodeValue::RawScope {
                inner_tokens: inner_scope,
                token: scope_node.extract_value().assume_lex_token(),
            };

        } else {
            node = unsafe { (*node).right() };
        }

    }
}


/// Divide the scope token tree into a list of separate statements based on semicolons
fn divide_statements_of_scope<'a>(mut tokens: TokenParsingList<'a>, symbol_table: &mut SymbolTable, parend_scope: Option<ScopeID>) -> UnparsedScopeBlock<'a> {

    let scope_id = symbol_table.add_scope(parend_scope);

    let mut block = UnparsedScopeBlock::new(scope_id);
    let mut node_ptr = unsafe { tokens.head() };

    while let Some(node) = unsafe { node_ptr.as_mut() } {

        match &node.value {
            
            TokenParsingNodeValue::LexToken(token) => if let TokenKind::Semicolon = token.value {
                // End of statement
                let mut statement = unsafe { tokens.extract_slice(tokens.head(), node_ptr) };

                // Ignore empty statements (only has one semicolon token)
                if !statement.has_one_item() {
                    // Drop the semicolon token because it is not needed anymore
                    // The last token is guaranteed to be a semicolon because of it's the current node
                    statement.drop_last();
                    block.statements.push(statement);
                }

                node_ptr = unsafe { tokens.head() };
            } else {
                node_ptr = unsafe { (*node_ptr).right() }
            },

            TokenParsingNodeValue::RawScope { inner_tokens, token } => {
                // Recursively parse the nested scope into separate statements

                if inner_tokens.is_empty() {
                    // Still declare the empty scope in the symbol table because it will be needed to calculate the scope size
                    let scope_id = symbol_table.add_scope(Some(scope_id));
                    node.value = TokenParsingNodeValue::SyntaxToken(
                        SyntaxNode::new(
                            SyntaxNodeValue::Scope(ScopeBlock::new(scope_id)),
                            token.source_token.clone()
                        )
                    )
                } else {
                    // The scope is not empty, so divide its inner statements recursively

                    let (inner_tokens, token) = node.extract_value().assume_raw_scope();

                    let unparsed_inner_block = divide_statements_of_scope(inner_tokens, symbol_table, Some(scope_id));

                    node.value = TokenParsingNodeValue::UnparsedScope { 
                        statements: unparsed_inner_block,
                        token,
                    };
                }

                // Scopes are not their own statements: they are treated as expressions like in Rust.

                node_ptr = unsafe { (*node_ptr).right() };
            },

            TokenParsingNodeValue::UnparsedScope { .. } => unreachable!("Unparsed scopes are not allowed at this stage. This is a bug."),
            TokenParsingNodeValue::SyntaxToken(_) => unreachable!("Syntax nodes are not allowed at this stage. This is a bug."),
            TokenParsingNodeValue::Placeholder => unreachable!("Invalid node {:#?}: Placeholder nodes are not allowed at this stage. This is a bug.", node),
        }
    }

    // Add any remaining tokens as the last statement
    if !tokens.is_empty() {
        block.statements.push(tokens);
    }
    
    block
}


fn parse_block_hierarchy<'a>(block: UnparsedScopeBlock<'a>, symbol_table: &mut SymbolTable<'a>, source: &SourceCode) -> ScopeBlock<'a> {
    // Recursively parse the statements' hierarchy
    // Do not check the types of the operators yet. This will be done in the next pass when the symbol table is created.

    let mut parsed_scope_block = ScopeBlock::new(block.scope_id);

    let mut unreachable_code = false;

    for mut statement in block.statements {

        if unreachable_code {
            // If the code is unreachable, don't bother parsing it
            let prev_statement = parsed_scope_block.statements.last().unwrap().token.as_ref();
            error::warn(statement.iter().next().unwrap().value.source_token(), source, format!("This statement is unreachable and will be ignored.\nThe previous statement at line {}:{} passed control to another branch:\n{}\n", prev_statement.line_number(), prev_statement.column, &source[prev_statement.line_index()]).as_str());
            break;
        }
        
        #[allow(unused_unsafe)] // A bug in the linter causes the below unsafe block to be marked as unnecessary, but removing it causes a compiler error
        while let Some(op_node) = find_highest_priority(&mut statement)
            .and_then(|node_ptr| unsafe { node_ptr.as_mut() }) // Convert the raw pointer to a mutable reference
        {

            if node_priority(op_node) == 0 {
                // No more operations to parse
                break;
            }
            // Set the priority to 0 so that the node is not visited again
            zero_node_priority(op_node);
    
            // Useful macros to get tokens without forgetting that the token pointers of extracted tokens are invalidated
            macro_rules! extract_left {
                () => {
                    statement.extract_node(unsafe { op_node.left() })
                };
            }
            macro_rules! extract_right {
                () => {
                    statement.extract_node(unsafe { op_node.right() })
                };
            }
    
            // Satisfy the operator requirements
            op_node.value = match op_node.extract_value() {

                TokenParsingNodeValue::LexToken(token) => {
                    
                    macro_rules! binary_satisfy_to {
                        ($sn:ident) => {
                            {
                                let left = unsafe { extract_left!() }.unwrap_or_else(
                                    || error::expected_argument(&token, source, format!("Missing left argument for operator {}.", token).as_str())
                                ).syntax_node_extract_value()
                                    .unwrap_or_else(
                                    |left| error::invalid_argument(&token.value, &left.source_token(), source, format!("Invalid left argument for operator {}.", token).as_str())
                                );
                
                                let right = unsafe { extract_right!() }.unwrap_or_else(
                                    || error::expected_argument(&token, source, format!("Missing right argument for operator {}.", token).as_str())
                                ).syntax_node_extract_value()
                                    .unwrap_or_else(
                                    |right| error::invalid_argument(&token.value, &right.source_token(), source, format!("Invalid right argument for operator {}.", token).as_str())
                                );
                                
                                TokenParsingNodeValue::SyntaxToken(SyntaxNode::new(
                                    SyntaxNodeValue::RuntimeOp(RuntimeOp::$sn { left: Box::new(left), right: Box::new(right) }),
                                    token.source_token.clone()
                                ))
                            }
                        };
                    }
                    macro_rules! unary_rigth_satisfy_to {
                        ($sn:ident) => {
                            {
                                let right = unsafe { extract_right!() }.unwrap_or_else(
                                    || error::expected_argument(&token, source, format!("Missing right argument for operator {}.", token).as_str())
                                ).syntax_node_extract_value()
                                    .unwrap_or_else(
                                    |right| error::invalid_argument(&token.value, &right.source_token(), source, format!("Invalid right argument for operator {}.", token).as_str())
                                );
                                
                                TokenParsingNodeValue::SyntaxToken(SyntaxNode::new(
                                    SyntaxNodeValue::RuntimeOp(RuntimeOp::$sn(Box::new(right))),
                                    token.source_token.clone()
                                ))
                            }
                        };
                    }
                    macro_rules! control_flow_no_args {
                        ($sn:ident) => {
                            {
                                // Check that this token isn't followed by any other token
                                if let Some(next_node) = unsafe { extract_right!() } {
                                    error::invalid_argument(&token.value, &next_node.source_token(), source, format!("Unexpected token {:?} after operator {}. This operator takes 0 arguments.", next_node.source_token(), token).as_str());
                                }
                
                                TokenParsingNodeValue::SyntaxToken(SyntaxNode::new(
                                    SyntaxNodeValue::RuntimeOp(RuntimeOp::$sn),
                                    token.source_token.clone()
                                ))
                            }
                        };
                    }

                    match token.value {

                        // Generic binary operators:
                        TokenKind::Add => binary_satisfy_to!(Add),
                        TokenKind::Sub => binary_satisfy_to!(Sub),
                        TokenKind::Mul => binary_satisfy_to!(Mul),
                        TokenKind::Div => binary_satisfy_to!(Div),
                        TokenKind::Mod => binary_satisfy_to!(Mod),
                        TokenKind::Equal => binary_satisfy_to!(Equal),
                        TokenKind::NotEqual => binary_satisfy_to!(NotEqual),
                        TokenKind::Greater => binary_satisfy_to!(Greater),
                        TokenKind::Less => binary_satisfy_to!(Less),
                        TokenKind::GreaterEqual => binary_satisfy_to!(GreaterEqual),
                        TokenKind::LessEqual => binary_satisfy_to!(LessEqual),
                        TokenKind::LogicalAnd => binary_satisfy_to!(LogicalAnd),
                        TokenKind::LogicalOr => binary_satisfy_to!(LogicalOr),
                        TokenKind::BitShiftLeft => binary_satisfy_to!(BitShiftLeft),
                        TokenKind::BitShiftRight => binary_satisfy_to!(BitShiftRight),
                        TokenKind::BitwiseOr => binary_satisfy_to!(BitwiseOr),
                        TokenKind::BitwiseAnd => binary_satisfy_to!(BitwiseAnd),
                        TokenKind::Assign => binary_satisfy_to!(Assign),
                        TokenKind::BitwiseXor => binary_satisfy_to!(BitwiseXor),

    
                        // Generic unary operators with argument to the right:
                        TokenKind::LogicalNot => unary_rigth_satisfy_to!(LogicalNot),
                        TokenKind::BitwiseNot => unary_rigth_satisfy_to!(BitwiseNot),

                        TokenKind::Deref => {
                            let right = unsafe { extract_right!() }.unwrap_or_else(
                                || error::expected_argument(&token, source, format!("Missing right argument for operator {}.", token).as_str())
                            ).syntax_node_extract_value()
                                .unwrap_or_else(
                                |arg| error::invalid_argument(&token.value, arg.source_token(), source, format!("Invalid right argument for operator {}.", token).as_str())
                            );
                            
                            TokenParsingNodeValue::SyntaxToken(SyntaxNode::new(
                                SyntaxNodeValue::RuntimeOp(RuntimeOp::Deref { 
                                    mutable: false, // Set to `false` node, it will be changed later if needed when the expression types are resolved
                                    expr: Box::new(right)
                                }),
                                token.source_token.clone()
                            ))
                        },
                        
                        TokenKind::Ref => {
                            let right = unsafe { extract_right!() }.unwrap_or_else(
                                || error::expected_argument(&token, source, format!("Missing right argument for operator {}.", token).as_str())
                            );
    
                            let (mutable, right) = match right.value {

                                TokenParsingNodeValue::LexToken(tok)
                                    if matches!(tok.value, TokenKind::Mut)
                                => {
                                    let right = unsafe { extract_right!() }.unwrap_or_else(
                                        || error::expected_argument(&token, source, format!("Missing right argument for operator {}.", token).as_str())
                                    );
                                    (true, right)
                                },

                                _ => (false, right)
                            };

                            let right = right.syntax_node_extract_value()
                                .unwrap_or_else(
                                    |right| error::invalid_argument(&token.value, right.source_token(), source, "Invalid expression after ref operator.")
                                );
                            
                            TokenParsingNodeValue::SyntaxToken(SyntaxNode::new(
                                SyntaxNodeValue::RuntimeOp(RuntimeOp::Ref { 
                                    mutable, 
                                    expr: Box::new(right)
                                }),
                                token.source_token.clone()
                            ))
                        },
    
                        TokenKind::Return => {
                            // Syntax: return <expression>
                            // Syntax: return
    
                            let return_expr = unsafe { extract_right!() }
                                .map(|node| node.syntax_node_extract_value()
                                    .unwrap_or_else(
                                        |arg| error::invalid_argument(&token.value, arg.source_token(), source, "Invalid expression after return operator.")
                                    )
                                )
                                .map(
                                    |expr| if !expr.is_expression() {
                                        error::invalid_argument(&token.value, &expr.token, source, format!("Invalid expression {:?} after return operator.", expr.value).as_str());
                                    } else { 
                                        expr
                                    }
                            );

                            // Statements after a return statement are unreachable
                            unreachable_code = true;

                            TokenParsingNodeValue::SyntaxToken(SyntaxNode::new(
                                SyntaxNodeValue::RuntimeOp(RuntimeOp::Return(return_expr.map(Box::new))),
                                token.source_token.clone()
                            ))
                        },
        
                        // No operands
                        TokenKind::Break => control_flow_no_args!(Break),
                        TokenKind::Continue => control_flow_no_args!(Continue),
        
                        // Other operators:
                        TokenKind::FunctionCallOpen => {
                            // Functin call is a list of tokens separated by commas enclosed in parentheses
                            // Statements inside the parentheses have already been parsed into single top-level tokens because of their higher priority
                            
                            let callable = unsafe { extract_left!() }.unwrap_or_else(
                                || error::expected_argument(&token, source, "Missing function name before function call operator.")
                            ).syntax_node_extract_value()
                                .unwrap_or_else(
                                |arg| error::invalid_argument(&token.value, arg.source_token(), source, "Invalid function name before function call operator.")
                            );

                            // Check for expression instead of only a symbol because expressions can evaluate to function pointers
                            if !callable.is_expression() {
                                error::invalid_argument(&token.value, &callable.token, source, "Invalid function name or function-returning expression before function call operator.");
                            }

                            let args = extract_list_like_delimiter_contents(&mut statement, op_node, &token, &TokenKind::ParClose, source).into_iter().map(
                                |arg_node| {

                                    let arg_node = arg_node.syntax_node_extract_value()
                                        .unwrap_or_else(
                                            |arg| error::invalid_argument(&token.value, arg.source_token(), source, "Invalid argument in function call. Expected an expression.")
                                        );
                                    
                                    // Check if every call argument is a valid expression
                                    if !arg_node.is_expression() {
                                        error::invalid_argument(&token.value, &arg_node.token, source, "Invalid argument in function call. Arguments must be expressions.");
                                    }
                                    arg_node
                                }
                            ).collect();
                            
                            TokenParsingNodeValue::SyntaxToken(SyntaxNode::new(
                                SyntaxNodeValue::RuntimeOp(RuntimeOp::Call { 
                                    callable: Box::new(callable), 
                                    args 
                                }),
                                token.source_token.clone()
                            ))
                        },
    
                        TokenKind::ArrayIndexOpen => {
                            // Syntax: <expression>[<expression>]
    
                            let array_expression = unsafe { extract_left!() }.unwrap() // Unwrap because the tokenizer interprets `[` as array index only if the previous token is an expression
                                .syntax_node_extract_value()
                                .unwrap_or_else(
                                    |arg| error::invalid_argument(&token.value, arg.source_token(), source, "Expected an array-like expression before an array subscription operator.")
                                );

                            if !array_expression.is_expression() {
                                error::invalid_argument(&token.value, &array_expression.token, source, "Expected an array-like expression before an array subscription operator.");
                            }
    
                            let index_expression = unsafe { extract_right!() }.unwrap() // Must have a right node for brackets to be balanced
                                .syntax_node_extract_value()
                                .unwrap_or_else(
                                    |arg| error::invalid_argument(&token.value, arg.source_token(), source, "Expected an expression in array subscript operator.")
                                );
                            
                            if !index_expression.is_expression() {
                                error::invalid_argument(&token.value, &index_expression.token, source, "Invalid argument in array subscript operator, expected an expression.");
                            }
                            
                            // Extract closing bracket
                            unsafe { extract_right!() }.map(|node| node.lex_token_extract_value()
                                .unwrap_or_else(
                                    |arg| error::invalid_argument(&token.value, arg.source_token(), source, "Invalid token in array subscript operator. Expected a closing square bracket.")
                                )
                            ).map(|node| if !matches!(node.value, TokenKind::SquareClose) {
                                    error::expected_argument(&node, source, "Expected closing square bracket after expression in array subscription.")
                                }).unwrap(); // Unwrap because delimiters are guaranteed to be balanced by the tokenizer
                            
                            TokenParsingNodeValue::SyntaxToken(SyntaxNode::new(
                                SyntaxNodeValue::RuntimeOp(RuntimeOp::ArrayIndex { 
                                    array: Box::new(array_expression),
                                    index: Box::new(index_expression) 
                                }),
                                token.source_token.clone()
                            ))
                        },
                    
        
                        TokenKind::ParOpen => {
                            // Syntax: (<expression>)
                            // Substitute the parenthesis node with its contents
        
                            // Extract the next token (either an expression or a closing parenthesis)
                            match unsafe { extract_right!() }.unwrap().value {
                                
                                TokenParsingNodeValue::LexToken(next_token)
                                    if matches!(next_token.value, TokenKind::ParClose) 
                                => {
                                    error::expected_argument(&token, source, "Empty parentheses are not allowed because they would evaluate to a void value.")
                                },

                                TokenParsingNodeValue::SyntaxToken(content_node)
                                    if content_node.is_expression()
                                => {
                                    // Check if the next token is a closing parenthesis (because the parentheses contain only one top-level node)
                                    let closing_parenthesis_node = unsafe { extract_right!() }.unwrap()
                                        .lex_token_extract_value()
                                        .unwrap_or_else(
                                            |arg| error::invalid_argument(&token.value, arg.source_token(), source, "Invalid token in parentheses. Expected a closing parenthesis ).")
                                        );

                                    if !matches!(closing_parenthesis_node.value, TokenKind::ParClose) {
                                        error::invalid_argument(&token.value, &closing_parenthesis_node.source_token, source, "Expected a closing parenthesis ).")
                                    }

                                    // Transform this parenthesis node into its inner expression
                                    TokenParsingNodeValue::SyntaxToken(content_node)
                                },
                                
                                arg => error::invalid_argument(&token.value, arg.source_token(), source, "Invalid token in parentheses. Expected an expression.")
                            }
                        },
            
                        TokenKind::ArrayOpen => {
                            // Extract the nodes within the square brackets and check if they are valid expressions
                            let inner_nodes = extract_list_like_delimiter_contents(&mut statement, op_node, &token, &TokenKind::SquareClose, source).into_iter()
                                .map(|inner_node| inner_node.syntax_node_extract_value().unwrap_or_else(
                                    |arg| error::invalid_argument(&token.value, arg.source_token(), source, "Invalid token inside literal array. Expected an expression.")
                                )).map(
                                    |inner_node| if inner_node.is_expression() {
                                        inner_node
                                    } else {
                                        error::invalid_argument(&token.value, &inner_node.token, source, "Invalid token inside literal array. Expected an expression.");
                                    }
                                ).collect();
                            
                            TokenParsingNodeValue::SyntaxToken(SyntaxNode::new(
                                SyntaxNodeValue::RuntimeOp(RuntimeOp::MakeArray { elements: inner_nodes }),
                                token.source_token.clone()
                            ))
                        },
        
                        TokenKind::TypeDef => {
                            // Syntax: typedef <name> = <definition>
        
                            let name_node = unsafe { extract_right!() }.unwrap_or_else(
                                || error::expected_argument(&token, source, "Missing name in type definition.")
                            )
                                .syntax_node_extract_value()
                                .unwrap_or_else(
                                    |arg| error::invalid_argument(&token.value, arg.source_token(), source, "Invalid type name in type definition.")
                            );

                            let type_name = match_or!(SyntaxNodeValue::Symbol { name, .. } = name_node.value, name,
                                error::invalid_argument(&token.value, &name_node.token, source, "Invalid type name in type definition.")
                            );
        
                            let assign_node = unsafe { extract_right!() }.unwrap_or_else(
                                || error::expected_argument(&token, source, "Missing assignment operator after type name in type declaration.")
                            );

                            if !matches!(&assign_node.value, TokenParsingNodeValue::LexToken(t) if matches!(t.value, TokenKind::Assign)) {
                                error::invalid_argument(&token.value, assign_node.source_token(), source, "Expected an assignment operator after type name in type definition.")
                            }
        
                            let definition_node = unsafe { extract_right!() }.unwrap_or_else(
                                || error::expected_argument(&token, source, "Missing data type after assignment operator in type definition.")
                            ).syntax_node_extract_value()
                                .unwrap_or_else(
                                    |arg| error::invalid_argument(&token.value, arg.source_token(), source, "Invalid data type after assignment operator in type definition.")
                            );

                            let data_type: Rc<DataType> = match_or!(SyntaxNodeValue::DataType(data_type) = definition_node.value, data_type,
                                error::invalid_argument(&token.value, &definition_node.token, source, "Expected a data type after assignment operator in type definition.")
                            );
        
                            let res = symbol_table.define_type(type_name, block.scope_id, data_type.clone(), token.source_token.clone());
                            if let Some(shadow) = res.err() {
                                error::already_defined(&name_node.token, &shadow.token, source, format!("{type_name} is defined multiple times in the same scope.").as_str())
                            }
        
                            TokenParsingNodeValue::SyntaxToken(SyntaxNode::new(
                                SyntaxNodeValue::TypeDef { name: type_name, definition: data_type },
                                token.source_token.clone()
                            ))
                        },
        
                        TokenKind::Static => {
                            // Syntax: static [mut] <name>: <type> = <expression>
                            // Explicit data type is required
                            // Must be initialized
        
                            // This node can either be the symbol name or the mut keyword
                            let name_or_mut_node = unsafe { extract_right!() }.unwrap_or_else(
                                || error::expected_argument(&token, source, "Missing name in static declaration.")
                            );

                            let (mutable, name_node) = match name_or_mut_node.value {

                                TokenParsingNodeValue::LexToken(mut_token) => {

                                    if !matches!(mut_token.value, TokenKind::Mut) {
                                        error::invalid_argument(&token.value, &mut_token.source_token, source, "Invalid token in static declaration. Expected a name or the mut keyword.")
                                    }

                                    let name_node = unsafe { extract_right!() }.unwrap_or_else(
                                        || error::expected_argument(&token, source, "Missing name in static declaration.")
                                    ).syntax_node_extract_value()
                                        .unwrap_or_else(
                                            |arg| error::invalid_argument(&token.value, arg.source_token(), source, "Invalid name in static declaration.")
                                    );

                                    (true, name_node)
                                },

                                TokenParsingNodeValue::SyntaxToken(name_node) => (false, name_node),

                                _ => unreachable!("Invalid token type in static declaration {:?}.", name_or_mut_node),
                            };
            
                            let symbol_name: &str = match_or!(SyntaxNodeValue::Symbol { name, .. } = &name_node.value, name, 
                                error::invalid_argument(&token.value, &name_node.token, source, "Invalid name in static declaration.")
                            );
        
                            let colon_node = unsafe { extract_right!() }.unwrap_or_else(
                                || error::expected_argument(&token, source, "Missing colon after name in static declaration.")
                            );
                            if !matches!(&colon_node.value, TokenParsingNodeValue::LexToken(token) if matches!(token.value, TokenKind::Colon)) {
                                error::invalid_argument(&token.value, colon_node.source_token(), source, "Expected a colon after name in static declaration.")
                            }
        
                            let data_type_node = unsafe { extract_right!() }.unwrap_or_else(
                                || error::expected_argument(&token, source, "Missing data type after name in static declaration.")
                            )
                                .syntax_node_extract_value()
                                .unwrap_or_else(
                                    |arg| error::invalid_argument(&token.value, arg.source_token(), source, "Invalid data type in static declaration.")
                            );
                            let data_type: Rc<DataType> = match_or!(SyntaxNodeValue::DataType(data_type) = data_type_node.value, data_type,
                                error::invalid_argument(&token.value, &data_type_node.token, source, "Invalid data type in static declaration.")
                            );
        
                            let assign_node = unsafe { extract_right!() }.unwrap_or_else(
                                || error::expected_argument(&token, source, "Missing assignment operator after data type in static declaration.")
                            );
                            if !matches!(&assign_node.value, TokenParsingNodeValue::LexToken(t) if matches!(t.value, TokenKind::Assign)) {
                                error::invalid_argument(&token.value, assign_node.source_token(), source, "Expected an assignment operator after data type in static declaration.")
                            }
        
                            let definition_node = unsafe { extract_right!() }.unwrap_or_else(
                                || error::expected_argument(&token, source, "Missing expression after assignment operator in static declaration.")
                            )
                                .syntax_node_extract_value()
                                .unwrap_or_else(
                                    |arg| error::invalid_argument(&token.value, arg.source_token(), source, "Invalid expression after assignment operator in static declaration.")
                            );
                            if !definition_node.is_expression() {
                                error::invalid_argument(&token.value, &definition_node.token, source, "Invalid expression after assignment operator in static declaration.")
                            }
        
                            let old_def = symbol_table.declare_constant_or_static(
                                symbol_name,
                                Symbol::new_uninitialized(
                                    data_type.clone(),
                                    token.source_token.clone(),
                                    SymbolValue::UninitializedStatic { mutable },
                                ),
                                block.scope_id
                            );
                            if let Err(old_def) = old_def {
                                error::already_defined(&name_node.token, &old_def, source, "Cannot define a static multiple times in the same scope")
                            }

                            TokenParsingNodeValue::SyntaxToken(SyntaxNode::new(
                                SyntaxNodeValue::Static { 
                                    name: symbol_name, 
                                    data_type, 
                                    definition: Box::new(definition_node)
                                },
                                token.source_token.clone()
                            ))
                        },
        
                        TokenKind::Const => {
                            // Syntax: const <name>: <type> = <expression>
                            // Explicit data type is required and cannot be mutable (of course)
        
                            let name_node = unsafe { extract_right!() }.unwrap_or_else(
                                || error::expected_argument(&token, source, "Missing name in constant declaration.")
                            ).syntax_node_extract_value()
                                .unwrap_or_else(
                                    |arg| error::invalid_argument(&token.value, arg.source_token(), source, "Invalid name in constant declaration.")
                                );

                            let symbol_name: &str = match_or!(SyntaxNodeValue::Symbol { name, .. } = &name_node.value, name,
                                error::invalid_argument(&token.value, &name_node.token, source, "Invalid constant name in constant declaration.")
                            );
        
                            let colon_node = unsafe { extract_right!() }.unwrap_or_else(
                                || error::expected_argument(&token, source, "Missing colon after constant name in constant declaration.")
                            ).lex_token_extract_value()
                                .unwrap_or_else(
                                    |arg| error::invalid_argument(&token.value, arg.source_token(), source, "Expected a colon after constant name in constant declaration.")
                                );

                            if !matches!(colon_node.value, TokenKind::Colon) {
                                error::invalid_argument(&token.value, &colon_node.source_token, source, "Expected a colon after constant name in constant declaration.")
                            }
        
                            let data_type_node = unsafe { extract_right!() }.unwrap_or_else(
                                || error::expected_argument(&token, source, "Missing data type after constant name in constant declaration.")
                            ).syntax_node_extract_value()
                                .unwrap_or_else(
                                    |arg| error::invalid_argument(&token.value, arg.source_token(), source, "Invalid data type in constant declaration.")
                                );

                            let data_type: Rc<DataType> = match_or!(SyntaxNodeValue::DataType(data_type) = data_type_node.value, data_type,
                                error::invalid_argument(&token.value, &data_type_node.token, source, "Invalid data type in constant declaration.")
                            );
        
                            let assign_node = unsafe { extract_right!() }.unwrap_or_else(
                                || error::expected_argument(&token, source, "Missing assignment operator after constant data type in constant declaration.")
                            ).lex_token_extract_value()
                                .unwrap_or_else(
                                    |arg| error::invalid_argument(&token.value, arg.source_token(), source, "Expected an assignment operator after constant data type in constant declaration.")
                                );
                            if !matches!(assign_node.value, TokenKind::Assign) {
                                error::invalid_argument(&token.value, &assign_node.source_token, source, "Expected an assignment operator after constant data type in constant declaration.")
                            }
        
                            let definition_node = unsafe { extract_right!() }.unwrap_or_else(
                                || error::expected_argument(&token, source, "Missing expression after assignment operator in constant declaration.")
                            ).syntax_node_extract_value()
                                .unwrap_or_else(
                                    |arg| error::invalid_argument(&token.value, arg.source_token(), source, "Invalid expression after assignment operator in constant declaration.")
                                );

                            if !definition_node.is_expression() {
                                error::invalid_argument(&token.value, &definition_node.token, source, "Invalid expression after assignment operator in constant declaration.")
                            }
        
                            let old_def = symbol_table.declare_constant_or_static(
                                symbol_name,
                                Symbol::new_uninitialized(
                                    data_type.clone(),
                                    token.source_token.clone(),
                                    SymbolValue::UninitializedConstant,
                                ),
                                block.scope_id
                            );
                            if let Err(old_def) = old_def {
                                error::already_defined(&name_node.token, &old_def, source, "Cannot define a constant multiple times in the same scope")
                            }
                        
                            TokenParsingNodeValue::SyntaxToken(SyntaxNode::new(
                                SyntaxNodeValue::Const { 
                                    name: symbol_name, 
                                    data_type, 
                                    definition: Box::new(definition_node)
                                },
                                token.source_token.clone()
                            ))
                        },
            
                        TokenKind::Let => {
                            // Syntax: let [mut] <name>: <type> 
                            // Syntax: let [mut] <name> = <expression>
                            // Syntax: let [mut] <name>: <type> = <expression>
            
                            // This node can either be the symbol name or the mut keyword
                            let mut_or_name = unsafe { extract_right!() }.unwrap_or_else(
                                || error::expected_argument(&token, source, "Missing variable name after let in variable declaration.")
                            );

                            let (mutable, name_node) = if matches!(&mut_or_name.value, TokenParsingNodeValue::LexToken(token) if matches!(token.value, TokenKind::Mut)) {
                                let name = unsafe { extract_right!() }.unwrap_or_else(
                                    || error::expected_argument(&token, source, "Missing variable name after let in variable declaration.")
                                );
                                (true, name)
                            } else {
                                (false, mut_or_name)
                            };

                            let mut name_node = name_node.syntax_node_extract_value()
                                .unwrap_or_else(
                                    |arg| error::invalid_argument(&token.value, arg.source_token(), source, "Invalid variable name in variable declaration.")
                                );
        
                            let symbol_name: &str = match_or!(SyntaxNodeValue::Symbol { name, .. } = &name_node.value, name, 
                                error::invalid_argument(&token.value, &name_node.token, source, "Invalid variable name in declaration.")
                            );
                            
                            // Use unsafe to get around the borrow checker not recognizing that the immutable borrow ends before op_node is borrowed mutably at the last line
                            let colon_or_assign = unsafe { op_node.right().as_ref() }.unwrap_or_else(
                                || error::expected_argument(&token, source, "Missing colon or equal sign after variable name in variable declaration.")
                            );
        
                            // The data type is either specified now after a colon or inferred later from the assigned value
                            let data_type = match &colon_or_assign.value {
                                
                                TokenParsingNodeValue::LexToken(t)
                                    if matches!(t.value, TokenKind::Colon)
                                => {
                                    unsafe { extract_right!() }.unwrap(); // Unwrap is safe because because of the previous check
        
                                    // The data type should be a single top-level token because of its higher priority
                                    let data_type_node = unsafe { extract_right!() }.unwrap_or_else(
                                        || error::expected_argument(&token, source, "Missing data type after colon in variable declaration.")
                                    ).syntax_node_extract_value()
                                        .unwrap_or_else(
                                            |arg| error::invalid_argument(&token.value, arg.source_token(), source, "Invalid data type after colon in variable declaration.")
                                        );

                                    match_or!(SyntaxNodeValue::DataType(data_type) = data_type_node.value, data_type,
                                        error::invalid_argument(&token.value, &data_type_node.token, source, "Token is not a valid data type.")
                                    )
                                },
        
                                TokenParsingNodeValue::LexToken(t)
                                    if matches!(t.value, TokenKind::Assign) => {
                                        // Data type is not specified in the declaration, it will be inferred later upon assignment
            
                                        DataType::Unspecified.into()
                                },
        
                                _ => error::invalid_argument(&token.value, colon_or_assign.source_token(), source, "Expected a colon or an equal sign after variable name in variable declaration.")
                            };
        
                            // Declare the new symbol in the local scope
                            let (discriminant, prev_declaration) = symbol_table.declare_symbol(
                                symbol_name,
                                Symbol::new_uninitialized(
                                    data_type, 
                                    token.source_token.clone(),
                                    if mutable { SymbolValue::Mutable } else { SymbolValue::Immutable(None) }, 
                                ),
                                block.scope_id
                            );
                            if let Some(prev_declaration) = prev_declaration {
                                error::warn(&name_node.token, source, &format!("Symbol `{}` was already declared in this scope. This declaration will overshadow the previous one.\nPrevious declaration at line {}:{}:", symbol_name, prev_declaration.line_number(), prev_declaration.column));
                                error::print_source_context(source, prev_declaration.line_index(), prev_declaration.column);
                            }
        
                            if let SyntaxNodeValue::Symbol { name: _, scope_discriminant } = &mut name_node.value {
                                *scope_discriminant = discriminant;
                            }
            
                            // Transform this node into a symbol node (the declatataor let is no longer needed since the symbol is already declared)
                            TokenParsingNodeValue::SyntaxToken(name_node)
                        },
        
                        TokenKind::Value(Value::Symbol { name, scope_discriminant: _ }) => {
        
                            // At this stage, Symbol tokens can either be custom types or actual symbols
                            match symbol_table.get_name_type(name, block.scope_id) {
        
                                Some(NameType::Symbol(current_discriminant)) => {
                                    // The name is a symbol
                                    TokenParsingNodeValue::SyntaxToken(SyntaxNode::new(
                                        SyntaxNodeValue::Symbol { name, scope_discriminant: current_discriminant },
                                        token.source_token.clone()
                                    ))
                                },
        
                                Some(NameType::Type(data_type)) => {
                                    // The name is a custom type
                                    TokenParsingNodeValue::SyntaxToken(SyntaxNode::new(
                                        SyntaxNodeValue::DataType(data_type),
                                        token.source_token.clone()
                                    ))
                                },
        
                                _ => {
                                    // Undefined symbols will be catched later. Function parameters, for example, would result in an error here.
                                    TokenParsingNodeValue::SyntaxToken(SyntaxNode::new(
                                        SyntaxNodeValue::Symbol { name, scope_discriminant: ScopeDiscriminant(0) },
                                        token.source_token.clone()
                                    ))
                                }
                            }
                        },

                        TokenKind::Value(Value::Literal { value }) => {

                            TokenParsingNodeValue::SyntaxToken(SyntaxNode::new(
                                SyntaxNodeValue::Literal(value), // TODO: fix this clone with Rc<LiteralValue>
                                token.source_token.clone()
                            ))
                        },

                        TokenKind::DataType(data_type) => {
                            TokenParsingNodeValue::SyntaxToken(SyntaxNode::new(
                                SyntaxNodeValue::DataType(data_type),
                                token.source_token.clone()
                            ))
                        },
            
                        TokenKind::Fn => {
                            // Function declaration syntax:
                            // [const] fn <name>(<arguments>) [-> <return type>] { <body> }
        
                            let is_const_function: bool = if let Some(TokenParsingNodeValue::LexToken(Token { value: TokenKind::Const, .. })) = unsafe { op_node.left().as_ref().map(|node| &node.value)} {
                                // Syntax: const fn ...
                                unsafe { extract_left!() }.unwrap(); // Remove the const token
                                true
                            } else { 
                                false
                            };
        
                            // Default return type is void, unless specified by the arrow ->
                            let mut return_type = Rc::new(DataType::Void);
            
                            let name_node = unsafe { extract_right!() }.unwrap_or_else(
                                || error::expected_argument(&token, source, "Missing function name after fn in function declaration.")
                            ).syntax_node_extract_value()
                                .unwrap_or_else(
                                    |arg| error::invalid_argument(&token.value, arg.source_token(), source, "Invalid function name in function declaration.")
                                );
                            let function_name: &str = match_or!(SyntaxNodeValue::Symbol { name, .. } = name_node.value, name,
                                error::invalid_argument(&token.value, &name_node.token, source, "Invalid function name in function declaration.")
                            );
            
                            // The parameters should be a single top-level token because of its higher priority
                            let params_node = unsafe { extract_right!() }.unwrap_or_else(
                                || error::expected_argument(&token, source, "Missing arguments after function name in function declaration.")
                            ).syntax_node_extract_value()
                                .unwrap_or_else(
                                    |arg| error::invalid_argument(&token.value, arg.source_token(), source, "Invalid arguments after function name in function declaration.")
                                );
                                
                            let params: Vec<FunctionParam> = match_or!(SyntaxNodeValue::FunctionParams(params) = params_node.value, params, 
                                error::invalid_argument(&token.value, &params_node.token, source, "Expected a list of arguments enclosed in parentheses after function name in function declaration.")
                            );
                            
                            // Extract the arrow -> or the function body
                            let arrow_or_scope = unsafe { extract_right!() }.unwrap_or_else(
                                || error::expected_argument(&token, source, "Missing arrow or function body after function parameters in function declaration.")
                            );
        
                            // Check if it's an arrow -> or a function body, return the function body
                            let body_node = match arrow_or_scope.value {
                                
                                TokenParsingNodeValue::LexToken(t)
                                    if matches!(t.value, TokenKind::Arrow)
                                => {
                                    // Extract the function return type
                                    let return_type_node = unsafe { extract_right!() }.unwrap_or_else(
                                        || error::expected_argument(&token, source, "Missing return type after arrow in function declaration.")
                                    ).syntax_node_extract_value()
                                        .unwrap_or_else(
                                            |arg| error::invalid_argument(&token.value, arg.source_token(), source, "Invalid return type after arrow in function declaration.")
                                        );

                                    return_type = match_or!(SyntaxNodeValue::DataType(data_type) = return_type_node.value, data_type,
                                        error::invalid_argument(&token.value, &return_type_node.token, source, "Invalid return type in function declaration.")
                                    );
            
                                    // The body node is the one after the return type
                                    unsafe { extract_right!() }
                                },
                                
                                // If there is no arrow ->, the node_after_params is the function body
                                _ => Some(arrow_or_scope)
                            };
        
                            // The body is one top-level scope token because it gets parsed first
                            let body_node = body_node.unwrap_or_else(
                                || error::expected_argument(&token, source, "Missing body after return type in function declaration.")
                            ).syntax_node_extract_value()
                                .unwrap_or_else(
                                    |arg| error::invalid_argument(&token.value, arg.source_token(), source, "Invalid body after return type in function declaration.")
                                );
                            let body: ScopeBlock = if let SyntaxNodeValue::Scope(block) = body_node.value {
                                block
                            } else {
                                error::invalid_argument(&token.value, &body_node.token, source, "Expected a function body enclosed in curly braces.");
                            };
                            
                            let signature: Rc<DataType> = DataType::Function { 
                                params: params.iter().map(|param| param.data_type.clone()).collect(), // Take only the data type of the parameter
                                return_type: return_type.clone() // Here we clone the Rc pointer, not the DataType value
                            }.into();
        
                            // Declare the function parameter names in the function's scope
                            // At the same time, construct a list of the parameter names
                            let mut param_names: Vec<&str> = Vec::with_capacity(params.len());

                            for param in params {

                                param_names.push(param.token.string);

                                let (_discriminant, prev_declaration) = symbol_table.declare_symbol(
                                    param.token.string, 
                                    Symbol::new_uninitialized(
                                        param.data_type.clone(),
                                        param.token.clone(),
                                        if param.mutable { SymbolValue::Mutable } else { SymbolValue::Immutable(None) },
                                    ), 
                                    body.scope_id // Note that the parameters are declared in the function's body scope
                                );
                                if let Some(prev_declaration) = prev_declaration {
                                    error::warn(&name_node.token, source, &format!("Symbol {} was already declared in this scope. This declaration will overshadow the previous one.\nPrevious declaration at line {}:{}:", param.name(), prev_declaration.line_number(), prev_declaration.column));
                                    error::print_source_context(source, prev_declaration.line_index(), prev_declaration.column);
                                }
                            }
        
                            let old_def = symbol_table.declare_function(
                                function_name,
                                is_const_function,
                                signature.clone(), 
                                param_names.into_boxed_slice(),
                                token.source_token.clone(),
                                block.scope_id
                            );
                            if let Err(old_def) = old_def {
                                error::already_defined(&name_node.token, &old_def, source, "Cannot define a function multiple times in the same scope")
                            }
                            
                            TokenParsingNodeValue::SyntaxToken(SyntaxNode::new(
                                SyntaxNodeValue::Function { 
                                    name: function_name, 
                                    signature, 
                                    body 
                                },
                                token.source_token.clone()
                            ))
                        },
        
                        TokenKind::FunctionParamsOpen => {
                            // Syntax: (<name>: <type>, <name>: <type>, ...)
        
                            let mut params: Vec<FunctionParam> = Vec::new();
        
                            let mut expected_comma: bool = false;
                            let mut mutable: bool = false;
                            loop {
                                let param_node = unsafe { extract_right!() }.unwrap_or_else(
                                    || error::expected_argument(&token, source, format!("Missing parameter or closing delimiter for operator {:?}.", token.value).as_str())
                                );
        
                                match param_node.value {
        
                                    TokenParsingNodeValue::LexToken(param_token) 
                                        if matches!(param_token.value, TokenKind::ParClose)
                                    => break,
        
                                    TokenParsingNodeValue::LexToken(param_token) 
                                        if matches!(param_token.value, TokenKind::Comma)
                                    => if expected_comma {
                                        // Set to false because you cannot have two adjacent commas
                                        expected_comma = false;
                                    } else {
                                        error::unexpected_token(&param_token, source, "Did you add an extra comma?")
                                    },
        
                                    TokenParsingNodeValue::LexToken(param_token) 
                                        if matches!(param_token.value, TokenKind::Mut)
                                    => {
                                        if mutable {
                                            error::syntax_error(&param_token.source_token, source, "Cannot have two adjacent mut keywords in function declaration.");
                                        }
                                        mutable = true;
                                    },
        
                                    TokenParsingNodeValue::SyntaxToken(sn) => {
                                        
                                        let name = match_or!(SyntaxNodeValue::Symbol { name, .. } = sn.value, name,
                                            error::invalid_argument(&token.value, &sn.token, source, "Invalid parameter name in function declaration.")
                                        );

                                        // Extract the colon
                                        let colon_node = unsafe { extract_right!() }.unwrap_or_else(
                                            || error::expected_argument(&token, source, format!("Missing colon after parameter name {:?} in function declaration.", name).as_str())
                                        ).lex_token_extract_value()
                                            .unwrap_or_else(
                                                |arg| error::invalid_argument(&token.value, arg.source_token(), source, "Invalid token after parameter name in function declaration.")
                                            );
                                        if !matches!(colon_node.value, TokenKind::Colon) {
                                            error::invalid_argument(&token.value, &colon_node.source_token, source, "Expected a semicolon after parameter name")
                                        }
                                        
                                        let data_type_node = unsafe { extract_right!() }.unwrap_or_else(
                                            || error::expected_argument(&token, source, "Missing data type after colon in function declaration.")
                                        ).syntax_node_extract_value()
                                            .unwrap_or_else(
                                                |arg| error::invalid_argument(&token.value, arg.source_token(), source, "Invalid data type after colon in function declaration.")
                                            );
                                        let data_type: Rc<DataType> = match_or!(SyntaxNodeValue::DataType(data_type) = data_type_node.value, data_type,
                                            error::invalid_argument(&token.value, &data_type_node.token, source, "Invalid data type in function declaration.")
                                        );
        
                                        params.push(FunctionParam { token: sn.token, data_type, mutable });
        
                                        // A comma is expected after each argument except the last one
                                        expected_comma = true;
                                        mutable = false;
                                    },
        
                                    _ => unreachable!("Invalid token kind during statement hierarchy parsing: {:?}. This token kind shouldn't have children.", param_node)
                                }
                            }
        
                            TokenParsingNodeValue::SyntaxToken(SyntaxNode::new(
                                SyntaxNodeValue::FunctionParams(params),
                                token.source_token.clone()
                            ))
                        },
        
                        TokenKind::ArrayTypeOpen => {
                            // Syntax: [<type>]
                            // Syntax: [<type>, <size>]
        
                            let element_type_node = unsafe { extract_right!() }.unwrap_or_else(
                                || error::expected_argument(&token, source, "Missing data type after array type open bracket in array declaration.")
                            ).syntax_node_extract_value()
                                .unwrap_or_else(
                                    |arg| error::invalid_argument(&token.value, arg.source_token(), source, "Invalid data type after array type open bracket in array declaration.")
                                );
                            let element_type = match_or!(SyntaxNodeValue::DataType(data_type) = element_type_node.value, data_type,
                                error::invalid_argument(&token.value, &element_type_node.token, source, "Invalid data type in array declaration.")
                            );
        
                            // May be either a comma or a closing ]
                            let comma_or_square = unsafe { extract_right!() }.unwrap_or_else(
                                || error::expected_argument(&token, source, "Missing array type close bracket after array type in array declaration.")
                            ).lex_token_extract_value()
                                .unwrap_or_else(
                                    |arg| error::invalid_argument(&token.value, arg.source_token(), source, "Invalid token after array type in array declaration.")
                                );
                            match comma_or_square.value {
                                TokenKind::Comma => {
                                    // Extract the closing square bracket ]
                                    let closing_bracket = unsafe { extract_right!() }.unwrap_or_else(
                                        || error::expected_argument(&token, source, "Missing array type close bracket after array type in array declaration.")
                                    ).lex_token_extract_value()
                                        .unwrap_or_else(
                                            |arg| error::invalid_argument(&token.value, arg.source_token(), source, "Invalid token after array type in array declaration.")
                                        );
                                    if !matches!(closing_bracket.value, TokenKind::SquareClose) {
                                        error::invalid_argument(&token.value, &closing_bracket.source_token, source, "Expected closing square bracket ].");
                                    }
                                },
                                TokenKind::SquareClose => {},
                                _ => error::invalid_argument(&token.value, &comma_or_square.source_token, source, "Invalid token after array type in array declaration.")
                            }
                            
                            let array_type = DataType::Array { element_type, size: None }.into();

                            // Transform this node into a data type node
                            TokenParsingNodeValue::SyntaxToken(SyntaxNode::new(
                                SyntaxNodeValue::DataType(array_type),
                                token.source_token.clone()
                            ))
                        },  
        
                        TokenKind::RefType => {
                            // Syntax: &<type>
                            // Syntax: &mut <type>
        
                            let mut_or_type = unsafe { extract_right!() }.unwrap_or_else(
                                || error::expected_argument(&token, source, "Missing data type or mutability after reference symbol &.")
                            );
        
                            let (mutable, element_type_node) = match mut_or_type.value {
                                
                                TokenParsingNodeValue::LexToken(t)
                                    if matches!(t.value, TokenKind::Mut)
                                => {
                                    // This is a mutable reference
                                    let target_type_node = unsafe { extract_right!() }.unwrap_or_else(
                                        || error::expected_argument(&token, source, "Missing data type after reference symbol &.")
                                    );
                                    (true, target_type_node)
                                },
                                _ => {
                                    // This is an immutable reference
                                    (false, mut_or_type)
                                }
                            };

                            let target_type_node = element_type_node.syntax_node_extract_value()
                                .unwrap_or_else(
                                    |arg| error::invalid_argument(&token.value, arg.source_token(), source, "Invalid data type after reference symbol &.")
                                );
                                                    
                            let target_type = match_or!(SyntaxNodeValue::DataType(data_type) = &target_type_node.value, data_type,
                                error::invalid_argument(&token.value, &target_type_node.token, source, "Expected a data type after reference symbol &.")
                            );
        
                            // Some data types should be merged together
                            let ref_type = match target_type.as_ref() {
                                DataType::RawString { length } => {
                                    if mutable {
                                        error::invalid_argument(&token.value, &target_type_node.token, source, "Raw strings cannot be mutable.")
                                    }
                                    DataType::StringRef { length: *length }.into()
                                },
                                _ => {
                                    DataType::Ref { target: target_type.clone(), mutable }.into()
                                }
                            };
                            
                            // Transform this node into a data type node
                            TokenParsingNodeValue::SyntaxToken(SyntaxNode::new(
                                SyntaxNodeValue::DataType(ref_type),
                                token.source_token.clone()
                            ))
                        },
        
                        TokenKind::As => {
                            // Syntax: <expression> as <type>
        
                            let expr = unsafe { extract_left!() }.unwrap_or_else(
                                || error::expected_argument(&token, source, format!("Missing expression before operator {:?}.", token.value).as_str())
                            ).syntax_node_extract_value()
                                .unwrap_or_else(
                                    |arg| error::invalid_argument(&token.value, arg.source_token(), source, "Invalid expression before as operator.")
                                );
                            if !expr.is_expression() {
                                error::invalid_argument(&token.value, &expr.token, source, "Expected an expression.")
                            }
                            
                            let data_type_node = unsafe { extract_right!() }.unwrap_or_else(
                                || error::expected_argument(&token, source, "Missing data type after type cast operator in type cast.")
                            ).syntax_node_extract_value()
                                .unwrap_or_else(
                                    |arg| error::invalid_argument(&token.value, arg.source_token(), source, "Invalid data type after type cast operator.")
                                );
                            let data_type = match_or!(SyntaxNodeValue::DataType(data_type) = data_type_node.value, data_type,
                                error::invalid_argument(&token.value, &data_type_node.token, source, "Expected a data type after type cast operator.")
                            );
        
                            TokenParsingNodeValue::SyntaxToken(SyntaxNode::new(
                                SyntaxNodeValue::As { target_type: data_type, expr: Box::new(expr) },
                                token.source_token.clone()
                            ))
                        },
        
                        TokenKind::If => {
                            // Syntax: if <condition> { <body> } [else if <condition> { <body> }]... [else { <body> }]
        
                            let mut if_chain: Vec<IfBlock> = Vec::new();
                            let mut else_block: Option<ScopeBlock> = None;
        
                            // The if operator node that is currently being parsed. Used for displaying correct error messages.
                            let mut reference_if_operator: Option<Token> = None;
        
                            // Parse the if-else chain
                            loop {
                                
                                let condition_node = unsafe { extract_right!() }.unwrap_or_else(
                                    || error::expected_argument(reference_if_operator.as_ref().unwrap_or(&token), source, "Missing condition after if operator.")
                                ).syntax_node_extract_value()
                                    .unwrap_or_else(
                                        |arg| error::invalid_argument(&reference_if_operator.as_ref().unwrap_or(&token).value, arg.source_token(), source, "Invalid condition after if operator.")
                                    );
                                if !condition_node.is_expression() {
                                    error::type_error(&condition_node.token, &[&DataType::Bool.name()], &condition_node.data_type, source, "Expected a boolean condition after if.");
                                }
        
                                let if_body_node = unsafe { extract_right!() }.unwrap_or_else(
                                    || error::expected_argument(reference_if_operator.as_ref().unwrap_or(&token), source, "Missing body after condition in if statement.")
                                );
                                let if_body_node = if_body_node.syntax_node_extract_value()
                                    .unwrap_or_else(
                                        |arg| error::invalid_argument(&reference_if_operator.as_ref().unwrap_or(&token).value, arg.source_token(), source, "Invalid body after condition in if statement.")
                                    );
                                let body = match_or!(SyntaxNodeValue::Scope(body) = if_body_node.value, body,
                                    error::invalid_argument(&reference_if_operator.as_ref().unwrap_or(&token).value, &if_body_node.token, source, "Expected a body enclosed in curly braces after condition in if statement.")
                                );
        
                                if_chain.push(
                                    IfBlock {
                                        condition: condition_node,
                                        body
                                    }
                                );
        
                                // Check for further else-if blocks 
                                // Use unsafe to circumvent the borrow checker not recognizing that the borrow ends right after the condition is checked
                                if !matches!(
                                    unsafe { op_node.right().as_ref() }.map(|node| &node.value),
                                    Some(TokenParsingNodeValue::LexToken(t)) if matches!(t.value, TokenKind::Else)
                                ) {
                                    // Next node is not an else branch, stop parsing the if-else chain
                                    break;
                                }
                                let else_node = unsafe { 
                                    extract_right!() .unwrap() // Unwrap is guaranteed to succeed because of the previous check
                                    .assume_lex_token_extract_value()
                                };

                                let if_or_scope = unsafe { extract_right!() }.unwrap_or_else(
                                    || error::expected_argument(&else_node, source, "Missing body after else.")
                                );
        
                                match if_or_scope.value {

                                    TokenParsingNodeValue::LexToken(if_token)
                                        if matches!(if_token.value, TokenKind::If)
                                    => {
                                        // Continue parsing the if-else chain
                                        // Update the reference if to this if node (for displaying correct error messages)
                                        reference_if_operator = Some(if_token);
                                    },

                                    TokenParsingNodeValue::SyntaxToken(sn) => {

                                        let body = match_or!(SyntaxNodeValue::Scope(body) = sn.value, body,
                                            error::invalid_argument(&else_node.value, &sn.token, source, "Expected a body enclosed in curly braces after else.")
                                        );

                                        // if-else chain is finished
                                        else_block = Some(body);
                                        break;
                                    },

                                    _ => error::invalid_argument(&else_node.value, if_or_scope.source_token(), source, "Expected an if or a body after else.")
                                }
                            }
        
                            TokenParsingNodeValue::SyntaxToken(SyntaxNode::new(
                                SyntaxNodeValue::IfChain { 
                                    if_blocks: if_chain,
                                    else_block
                                },
                                token.source_token.clone()
                            ))
                        },
        
                        TokenKind::Loop => {
                            // Syntax: loop { <body> }
        
                            let body_node = unsafe { extract_right!() }.unwrap_or_else(
                                || error::expected_argument(&token, source, "Missing body after loop.")
                            ).syntax_node_extract_value()
                                .unwrap_or_else(
                                    |arg| error::invalid_argument(&token.value, arg.source_token(), source, "Invalid body after loop.")
                                );
                            
                            let body = match_or!(SyntaxNodeValue::Scope(body) = body_node.value, body,
                                error::invalid_argument(&token.value, &body_node.token, source, "Expected a body enclosed in curly braces after loop.")
                            );
        
                            TokenParsingNodeValue::SyntaxToken(SyntaxNode::new(
                                SyntaxNodeValue::Loop { body },
                                token.source_token.clone()
                            ))
                        },
        
                        TokenKind::While => {
                            // Syntax: while <condition> { <body> }
        
                            let condition_node = unsafe { extract_right!() }.unwrap_or_else(
                                || error::expected_argument(&token, source, "Missing condition after while operator.")
                            ).syntax_node_extract_value()
                                .unwrap_or_else(
                                    |arg| error::invalid_argument(&token.value, arg.source_token(), source, "Invalid condition after while operator.")
                                );
                            if !condition_node.is_expression() {
                                error::type_error(&condition_node.token, &[&DataType::Bool.name()], &condition_node.data_type, source, "Expected a boolean condition after while.");
                            }
        
                            let body_node = unsafe { extract_right!() }.unwrap_or_else(
                                || error::expected_argument(&token, source, "Missing body after condition in while loop.")
                            ).syntax_node_extract_value()
                                .unwrap_or_else(
                                    |arg| error::invalid_argument(&token.value, arg.source_token(), source, "Invalid body after condition in while loop.")
                                );
                            
                            let body = match_or!(SyntaxNodeValue::Scope(body) = body_node.value, body,
                                error::invalid_argument(&token.value, &body_node.token, source, "Expected a body enclosed in curly braces after condition in while loop.")
                            );
        
                            TokenParsingNodeValue::SyntaxToken(SyntaxNode::new(
                                SyntaxNodeValue::While { 
                                    condition: Box::new(condition_node), 
                                    body
                                },
                                token.source_token.clone()
                            ))
                        },
        
                        TokenKind::DoWhile => {
                            // Syntax: do { <body> } while <condition>
        
                            let body_node = unsafe { extract_right!() }.unwrap_or_else(
                                || error::expected_argument(&token, source, "Missing body after do operator.")
                            ).syntax_node_extract_value()
                                .unwrap_or_else(
                                    |arg| error::invalid_argument(&token.value, arg.source_token(), source, "Invalid body after do operator.")
                                );
                            
                            let body = match_or!(SyntaxNodeValue::Scope(body) = body_node.value, body,
                                error::invalid_argument(&token.value, &body_node.token, source, "Expected a body enclosed in curly braces after do operator.")
                            );
        
                            let while_node = unsafe { extract_right!() }.unwrap_or_else(
                                || error::expected_argument(&token, source, "Missing while operator after body in do-while loop.")
                            ).lex_token_extract_value()
                                .unwrap_or_else(
                                    |arg| error::invalid_argument(&token.value, arg.source_token(), source, "Expected a while operator after body in do-while loop.")
                                );
                            if !matches!(while_node.value, TokenKind::While) {
                                error::invalid_argument(&token.value, &while_node.source_token, source, "Expected a while operator after body in do-while loop.");
                            }
        
                            let condition_node = unsafe { extract_right!() }.unwrap_or_else(
                                || error::expected_argument(&token, source, "Missing condition after while operator in do-while loop.")
                            ).syntax_node_extract_value()
                                .unwrap_or_else(
                                    |arg| error::invalid_argument(&token.value, arg.source_token(), source, "Invalid condition after while operator in do-while loop.")
                                );
                            if !condition_node.is_expression() {
                                error::type_error(&condition_node.token, &[&DataType::Bool.name()], &condition_node.data_type, source, "Expected a boolean condition after while in do-while loop.");
                            }
        
                            TokenParsingNodeValue::SyntaxToken(SyntaxNode::new(
                                SyntaxNodeValue::DoWhile { 
                                    condition: Box::new(condition_node), 
                                    body
                                },
                                token.source_token.clone()
                            ))
                        }
        
                        _ => unreachable!("Invalid token kind during statement hierarchy parsing: {:?}.", token.value)
                    }
                },

                TokenParsingNodeValue::UnparsedScope { statements, token } => {
                    // Parse the children statements of the scope
                    // The children have already been extracted and divided into separate statements.
                    
                    let parsed_block = parse_block_hierarchy(statements, symbol_table, source);

                    TokenParsingNodeValue::SyntaxToken(
                        SyntaxNode::new(
                            SyntaxNodeValue::Scope(parsed_block),
                            token.source_token.clone()
                        )
                    )
                },

                TokenParsingNodeValue::SyntaxToken(_) => unreachable!("Syntax nodes are not allowed at this stage, they should have been skipped cause 0 priority. This is a bug."),
                TokenParsingNodeValue::RawScope { .. } => unreachable!("Raw scopes are not allowed at this stage, they should have been converted to UnparsedScope. This is a bug."),
                TokenParsingNodeValue::Placeholder => unreachable!("Placeholders are not allowed at this stage, they should have been removed. This is a bug."),
            }
                
        }

        while let Some(top_node) = statement.extract_first() {
            parsed_scope_block.statements.push(
                unsafe { top_node.assume_syntax_node_extract_value() }
            );
        }

    }

    parsed_scope_block
}


/// Extract comma-separated tokens within a delimiter (parentheses, square brackets, etc.).
/// 
/// Removes the closing delimiter from the token list without including it in the returned arguments.
fn extract_list_like_delimiter_contents<'a>(

    tokens: &mut TokenParsingList<'a>,
    start_delimiter_ptr: *mut TokenParsingNode<'a>,
    start_delimiter_token: &Token,
    delimiter: &TokenKind<'_>,
    source: &SourceCode

) -> Vec<TokenParsingNode<'a>> 
{

    let mut arguments = Vec::new();

    let start_delimiter = unsafe { &mut *start_delimiter_ptr };

    // Set to false because the first token in a collection can't be a comma
    let mut expected_comma: bool = false;
   
    // Extract the arguments within the delimiters
    loop {

        let arg_node = unsafe { tokens.extract_node(start_delimiter.right()) }.unwrap_or_else(
            || error::expected_argument(start_delimiter_token, source, format!("Missing argument or closing delimiter for operator {:?}.", start_delimiter_token.value).as_str())
        );

        match &arg_node.value {

            TokenParsingNodeValue::LexToken(token) => {
                match &token.value {

                    t if std::mem::discriminant(t) == std::mem::discriminant(delimiter) => break,
        
                    TokenKind::Comma => if expected_comma {
                        // Set to false because you cannot have two adjacent commas
                        expected_comma = false;
                    } else {
                        error::unexpected_token(token, source, "Did you add an extra comma?");
                    },
        
                    _ => unreachable!("Unexpected token kind during list extraction: {:?}. This is a bug.", token)
                }
            },

            TokenParsingNodeValue::SyntaxToken(_) => {
                // The token type will be checked later
                arguments.push(*arg_node);
                // A comma is expected after each argument except the last one
                expected_comma = true;
            },
            
            _ => unreachable!("Invalid token kind during list extraction: {:?}. This is a bug.", arg_node)
        }
    }

    arguments
}


fn node_priority(node: &TokenParsingNode) -> TokenPriority {
    match &node.value {
        TokenParsingNodeValue::LexToken(token) => token.priority,
        TokenParsingNodeValue::SyntaxToken(_) => 0,
        TokenParsingNodeValue::RawScope { token, .. } => token.priority,
        TokenParsingNodeValue::UnparsedScope { token, .. } => token.priority,
        TokenParsingNodeValue::Placeholder => unreachable!("Invalid node {:#?}: Placeholders are not allowed at this stage, they should have been removed. This is a bug.", node),
    }
}


fn zero_node_priority(node: &mut TokenParsingNode) {
    match &mut node.value {
        TokenParsingNodeValue::LexToken(token) => token.priority = 0,
        TokenParsingNodeValue::SyntaxToken(_) => {},
        TokenParsingNodeValue::RawScope { token, .. } => token.priority = 0,
        TokenParsingNodeValue::UnparsedScope { token, .. } => token.priority = 0,
        TokenParsingNodeValue::Placeholder => unreachable!("Invalid node {:#?}: Placeholders are not allowed at this stage, they should have been removed. This is a bug.", node),
    }
}


/// Find the token node with the highest priority in the uppermost layer of the tree.
fn find_highest_priority<'a>(tokens: &mut TokenParsingList<'a>) -> Option<*mut TokenParsingNode<'a>> {

    let mut highest_priority: Option<&mut TokenParsingNode> = None;

    for node in tokens.iter_mut() {
        if let Some(ref hp) = highest_priority {

            
            if node_priority(node) > node_priority(hp) {
                highest_priority = Some(node);
            }
        } else {
            highest_priority = Some(node);
        }
    }

    highest_priority.map(|node| node as *mut TokenParsingNode)
}


/// Build an abstract syntax tree from a flat list of tokens
pub fn build_ast<'a>(mut tokens: TokenParsingList<'a>, source: &SourceCode, symbol_table: &mut SymbolTable<'a>, verbose: bool) -> ScopeBlock<'a> {

    parse_scope_hierarchy(&mut tokens);

    let unparsed_outer = divide_statements_of_scope(tokens, symbol_table, None);

    if verbose {
        println!("Statements after division:\n\n{}", unparsed_outer);
    }

    let parsed_outer = parse_block_hierarchy(unparsed_outer, symbol_table, source);

    if verbose {
        println!("\n\nStatement hierarchy:\n{}", parsed_outer);
    }

    parsed_outer
}

