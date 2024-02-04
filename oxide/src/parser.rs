use std::rc::Rc;

use rust_vm_lib::ir::IRCode;

use crate::data_types::{DataType, LiteralValue};
use crate::error::warn;
use crate::{binary_operators, error, unary_operators};
use crate::operations::Ops;
use crate::token::{TokenKind, Value};
use crate::symbol_table::{ScopeID, Symbol, SymbolTable, SymbolValue};
use crate::token_tree::{ChildrenType, FunctionParam, IfBlock, ScopeBlock, TokenNode, TokenTree, UnparsedScopeBlock};
use crate::{match_or, match_unreachable};


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
fn divide_statements<'a>(mut tokens: TokenTree<'a>, symbol_table: &mut SymbolTable, parent_scope: Option<ScopeID>) -> UnparsedScopeBlock<'a> {

    let scope_id = symbol_table.add_scope(parent_scope);
    let mut block = UnparsedScopeBlock::new(scope_id);
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
                node_ref.children = if let Some(ChildrenType::Tree(children_tree)) = node_ref.children.take() {
                    Some(ChildrenType::UnparsedBlock(divide_statements(children_tree, symbol_table, Some(scope_id))))
                } else {
                    // Empty scope
                    Some(ChildrenType::ParsedBlock(ScopeBlock::new(ScopeID::placeholder())))
                };

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


fn parse_block_hierarchy<'a>(block: UnparsedScopeBlock<'a>, symbol_table: &mut SymbolTable<'a>, source: &IRCode) -> ScopeBlock<'a> {
    // Recursively parse the statements' hierarchy
    // Do not check the types of the operators yet. This will be done in the next pass when the symbol table is created.

    let mut parsed_scope_block = ScopeBlock::new(block.scope_id);

    for mut statement in block.statements {
        
        #[allow(unused_unsafe)] // A bug in the linter causes the below unsafe block to be marked as unnecessary, but removing it causes a compiler error
        while let Some(op_node) = find_highest_priority(&statement)
            .and_then(|node_ptr| unsafe { node_ptr.as_mut() }) // Convert the raw pointer to a mutable reference
        {

            if op_node.item.priority == 0 {
                // No more operations to parse
                break;
            }
            // Set the priority to 0 so that the node is not visited again
            op_node.item.priority = 0;
    
            // Useful macros to get tokens without forgetting that the token pointers of extracted tokens are invalidated
            macro_rules! extract_left {
                () => {
                    statement.extract_node(op_node.left)
                };
            }
            macro_rules! extract_right {
                () => {
                    statement.extract_node(op_node.right)
                };
            }
    
            // Satisfy the operator requirements
            match &op_node.item.value {
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
                            || error::expected_argument(&op_node.item, source, format!("Missing left argument for operator {}.", op).as_str())
                        );
                    
                        let right = extract_right!().unwrap_or_else(
                            || error::expected_argument(&op_node.item, source, format!("Missing right argument for operator {}.", op).as_str())
                        );
                    
                        op_node.children = Some(ChildrenType::Binary(left, right));
                    },
    
                    // Unary operators with argument to the right:
                    Ops::Deref { .. } |
                    Ops::LogicalNot |
                    Ops::BitwiseNot
                    => {
                        let right = extract_right!().unwrap_or_else(
                            || error::expected_argument(&op_node.item, source, format!("Missing right argument for operator {}.", op).as_str())
                        );
                        op_node.children = Some(ChildrenType::Unary(right));
                    },
                    
                    Ops::Ref { .. }
                     => {
                        let right = extract_right!().unwrap_or_else(
                            || error::expected_argument(&op_node.item, source, format!("Missing right argument for operator {}.", op).as_str())
                        );

                        let (mutable, right) = if let TokenKind::Mut = right.item.value {
                            let right = extract_right!().unwrap_or_else(
                                || error::expected_argument(&op_node.item, source, format!("Missing right argument for operator {}.", op).as_str())
                            );
                            (true, right)
                        } else {
                            (false, right)
                        };

                        let mutable_field = match_unreachable!(TokenKind::Op(Ops::Ref { mutable }) = &mut op_node.item.value, mutable);
                        *mutable_field = mutable;
                        
                        op_node.children = Some(ChildrenType::Unary(right));
                    },

                    Ops::Return => {
                        // Syntax: return <expression>
                        // Syntax: return

                        extract_right!().map(
                            |expr| if !is_expression(&expr.item.value) {
                                error::invalid_argument(&op_node.item.value, &expr.item, source, format!("Invalid expression {:?} after return operator.", expr.item.value).as_str());
                            } else { 
                                op_node.children = Some(ChildrenType::Unary(expr));
                            }
                        );
                    },
    
                    // No operands
                    Ops::Break |
                    Ops::Continue => {
                        // Syntax: break
                        // Syntax: continue

                        // Check that this token isn't followed by any other token
                        if let Some(next_node) = extract_right!() {
                            error::invalid_argument(&op_node.item.value, &next_node.item, source, format!("Unexpected token {:?} after operator {}.", next_node.item.value, op).as_str());
                        }
                    }
    
                    // Other operators:
                    Ops::FunctionCallOpen => {
                        // Functin call is a list of tokens separated by commas enclosed in parentheses
                        // Statements inside the parentheses have already been parsed into single top-level tokens because of their higher priority
                        
                        let callable = extract_left!().unwrap_or_else(
                            || error::expected_argument(&op_node.item, source, "Missing function name before function call operator.")
                        );
                        // Check for expression instead of only a symbol because expressions can evaluate to function pointers
                        if !is_expression(&callable.item.value) {
                            error::invalid_argument(&op_node.item.value, &callable.item, source, "Invalid function name or function-returning expression before function call operator.");
                        }

                        let args = extract_list_like_delimiter_contents(&mut statement, op_node, &op_node.item.value, &TokenKind::ParClose, source);
                        // Check if every call argument is a valid expression
                        for arg_node in &args {
                            if !is_expression(&arg_node.item.value) {
                                error::invalid_argument(&op_node.item.value, &arg_node.item, source, "Invalid argument in function call. Arguments must be expressions.");
                            }
                        }
                        
                        op_node.children = Some(ChildrenType::Call { callable, args });
                    },

                    Ops::ArrayIndexOpen => {
                        // Syntax: <expression>[<expression>]

                        let array_expression = extract_left!().unwrap(); // Unwrap because the tokenizer interprets `[` as array index only if the previous token is an expression
                        if !is_expression(&array_expression.item.value) {
                            error::invalid_argument(&op_node.item.value, &array_expression.item, source, "Expected an array-like expression before an array subscription operator.");
                        }

                        let index_expression = extract_right!().unwrap(); // Must have a right node for brackets to be balanced
                        if !is_expression(&index_expression.item.value) {
                            error::invalid_argument(&op_node.item.value, &index_expression.item, source, "Invalid argument in array subscript operator, expected an expression.");
                        }
                        
                        // Extract closing bracket
                        extract_right!().map(|node| if !matches!(node.item.value, TokenKind::SquareClose) {
                            error::expected_argument(&node.item, source, "Expected closing square bracket after expression in array subscription.")
                        }).unwrap(); // Unwrap because delimiters are guaranteed to be balanced by the tokenizer

                        op_node.children = Some(ChildrenType::Binary(array_expression, index_expression));
                    }
                    
                },
    
                TokenKind::ParOpen => {
                    // Syntax: (<expression>)
                    // Substitute the parenthesis node with its contents

                    // Extract the next token (either an expression or a closing parenthesis)
                    match extract_right!() {

                        Some(next_node) => match &next_node.item.value {

                            // Empty parentheses are not allowed, as they would evaluate to a void value
                            TokenKind::ParClose => error::expected_argument(&op_node.item, source, "Empty parentheses are not allowed because they would evaluate to a void value."),
                                
                            tk if is_expression(tk) => {
                                // next_node is here guaranteed to be a value or an expression

                                // Check if the next token is a closing parenthesis (because the parentheses contain only one top-level node)
                                let closing_parenthesis_node =extract_right!().unwrap();
                                if !matches!(closing_parenthesis_node.item.value, TokenKind::ParClose) {
                                    error::invalid_argument(&op_node.item.value, &closing_parenthesis_node.item, source, "Expected a closing parenthesis ).")
                                }

                                // Transform this parenthesis node into its inner expression
                                op_node.substitute(*next_node)
                            },

                            _ => error::invalid_argument(&op_node.item.value, &next_node.item, source, "Invalid token in parentheses. Expected an expression.")
                        },

                        None => unreachable!("Parenthesis open token has no right token. Tokenizer should have already caught this. This is a bug."),
                    }
                },
    
                TokenKind::ArrayOpen => {
                    // Extract the nodes within the square brackets and check if they are valid expressions
                    let inner_nodes = extract_list_like_delimiter_contents(&mut statement, op_node, &op_node.item.value, &TokenKind::SquareClose, source).into_iter().map(
                        |inner_node| if is_expression(&inner_node.item.value) {
                            inner_node
                        } else {
                            error::invalid_argument(&op_node.item.value, &inner_node.item, source, "Invalid token inside literal array. Expected an expression.");
                        }
                    ).collect();
                    
                    op_node.children = Some(ChildrenType::List(inner_nodes));
                },
    
                TokenKind::ScopeOpen => {
                    // Parse the children statements of the scope
                    // The children have already been extracted and divided into separate statements.
                    
                    let parsed_block = match op_node.children.take().unwrap() { // Assume children are present (if not, this is a bug)
                        ChildrenType::UnparsedBlock(statements) => parse_block_hierarchy(statements, symbol_table, source),
                        // If it's alreaady ParsedBlock, then it's empty. In this case, don't bother parsing it.
                        ChildrenType::ParsedBlock(statements) => statements,
                        _ => unreachable!()
                    };
                    
                    op_node.children = Some(ChildrenType::ParsedBlock(parsed_block));
                },

                TokenKind::Const => {
                    // Syntax: const <name>: <type> = <expression>
                    // Explicit data type is required and cannot be mutable (of course)

                    let name_node = extract_right!().unwrap_or_else(
                        || error::expected_argument(&op_node.item, source, "Missing name in constant declaration.")
                    );
                    let symbol_name: &str = match_or!(TokenKind::Value(Value::Symbol { name, .. }) = &name_node.item.value, name,
                        error::invalid_argument(&op_node.item.value, &name_node.item, source, "Invalid constant name in constant declaration.")
                    );

                    let colon_node = extract_right!().unwrap_or_else(
                        || error::expected_argument(&op_node.item, source, "Missing colon after constant name in constant declaration.")
                    );
                    if !matches!(colon_node.item.value, TokenKind::Colon) {
                        error::invalid_argument(&op_node.item.value, &colon_node.item, source, "Expected a colon after constant name in constant declaration.")
                    }

                    let data_type_node = extract_right!().unwrap_or_else(
                        || error::expected_argument(&op_node.item, source, "Missing data type after constant name in constant declaration.")
                    );
                    let data_type: Rc<DataType> = match_or!(TokenKind::DataType(data_type) = data_type_node.item.value, data_type,
                        error::invalid_argument(&op_node.item.value, &data_type_node.item, source, "Invalid data type in constant declaration.")
                    ).into();

                    let assign_node = extract_right!().unwrap_or_else(
                        || error::expected_argument(&op_node.item, source, "Missing assignment operator after constant data type in constant declaration.")
                    );
                    if !matches!(assign_node.item.value, TokenKind::Op(Ops::Assign)) {
                        error::invalid_argument(&op_node.item.value, &assign_node.item, source, "Expected an assignment operator after constant data type in constant declaration.")
                    }

                    let definition_node = extract_right!().unwrap_or_else(
                        || error::expected_argument(&op_node.item, source, "Missing expression after assignment operator in constant declaration.")
                    );
                    if !is_expression(&definition_node.item.value) {
                        error::invalid_argument(&op_node.item.value, &definition_node.item, source, "Invalid expression after assignment operator in constant declaration.")
                    }

                    let (discriminant, res) = symbol_table.declare_symbol(
                        symbol_name.to_string(),
                        Symbol::new_uninitialized(
                            data_type.clone(),
                            op_node.item.token.clone(),
                            SymbolValue::UninitializedConstant,
                        ),
                        block.scope_id
                    );
                    if let Some(warning) = res.warning() {
                        error::warn(&name_node.item.token, source, warning);
                    }

                    op_node.children = Some(ChildrenType::Const { 
                        name: symbol_name,
                        discriminant,
                        data_type,
                        definition: definition_node,
                    });
                }
    
                TokenKind::Let => {
                    // Syntax: let [mut] <name>: <type> 
                    // Syntax: let [mut] <name>: = <typed expression>
    
                    // This node can either be the symbol name or the mut keyword
                    let next_node = extract_right!().unwrap_or_else(
                        || error::expected_argument(&op_node.item, source, "Missing variable name after let in variable declaration.")
                    );
    
                    let (mutable, mut name_token) = if matches!(next_node.item.value, TokenKind::Mut) {
                        let name = extract_right!().unwrap_or_else(
                            || error::expected_argument(&op_node.item, source, "Missing variable name after let in variable declaration.")
                        );
                        (true, name)
                    } else {
                        (false, next_node)
                    };

                    let symbol_name: &str = match_or!(TokenKind::Value(Value::Symbol { name, .. }) = &name_token.item.value, name, 
                        error::invalid_argument(&op_node.item.value, &name_token.item, source, "Invalid variable name in declaration.")
                    );
                    
                    // Use unsafe to get around the borrow checker not recognizing that the immutable borrow ends before op_node is borrowed mutably at the last line
                    let after_name = unsafe { op_node.right.as_ref() }.unwrap_or_else(
                        || error::expected_argument(&op_node.item, source, "Missing colon or equal sign after variable name in variable declaration.")
                    );

                    // The data type is either specified now after a colon or inferred later from the assigned value
                    let data_type: DataType = match after_name.item.value {

                        TokenKind::Colon => {

                            extract_right!().unwrap(); // Unwrap is safe because because of the previous check

                            // The data type should be a single top-level token because of its higher priority
                            let data_type_node = extract_right!().unwrap_or_else(
                                || error::expected_argument(&op_node.item, source, "Missing data type after colon in variable declaration.")
                            );
                            match_or!(TokenKind::DataType(data_type) = data_type_node.item.value, data_type,
                                error::invalid_argument(&op_node.item.value, &data_type_node.item, source, "Token is not a valid data type.")
                            )
                        },

                        TokenKind::Op(Ops::Assign) => {
                            // Data type is not specified in the declaration, it will be inferred later upon assignment

                            DataType::Unspecified
                        },

                        _ => error::invalid_argument(&op_node.item.value, &after_name.item, source, "Expected a colon or an equal sign after variable name in variable declaration.")
                    };

                    // Declare the new symbol in the local scope
                    let (discriminant, res) = symbol_table.declare_symbol(
                        symbol_name.to_string(),
                        Symbol::new_uninitialized(
                            Rc::new(data_type), 
                            op_node.item.token.clone(),
                            if mutable { SymbolValue::Mutable } else { SymbolValue::Immutable(None) }, 
                        ),
                        block.scope_id
                    );
                    if let Some(warning) = res.warning() {
                        error::warn(&name_token.item.token, source, warning);
                    }

                    if let TokenKind::Value(Value::Symbol { name: _, scope_discriminant }) = &mut name_token.item.value {
                        *scope_discriminant = discriminant;
                    }
    
                    // Transform this node into a symbol node (the declatataor let is no longer needed since the symbol is already declared)
                    op_node.substitute(*name_token);
                },

                TokenKind::Value(Value::Symbol { name, scope_discriminant: _ }) => {

                    if let Some(current_discriminant) = symbol_table.get_current_discriminant(name, block.scope_id) {
                        let scope_discriminant = match_unreachable!(TokenKind::Value(Value::Symbol { name: _, scope_discriminant }) = &mut op_node.item.value, scope_discriminant);
                        *scope_discriminant = current_discriminant;
                    }
                    // Undefined symbols will be catched later. Function parameters, for example, would result in an error here.

                },
    
                TokenKind::Fn => {
                    // Function declaration syntax:
                    // fn <name>(<arguments>) -> <return type> { <body> }
                    // fn <name>(argumenta>) { <body> }

                    // Default return type is void, unless specified by the arrow ->
                    let mut return_type = Rc::new(DataType::Void);
    
                    let name_node = extract_right!().unwrap_or_else(
                        || error::expected_argument(&op_node.item, source, "Missing function name after fn in function declaration.")
                    );
                    let function_name: &str = match_or!(TokenKind::Value(Value::Symbol { name, .. }) = name_node.item.value, name,
                        error::invalid_argument(&op_node.item.value, &name_node.item, source, "Invalid function name in function declaration.")
                    );
    
                    // The parameters should be a single top-level token because of its higher priority
                    let params = extract_right!().unwrap_or_else(
                        || error::expected_argument(&op_node.item, source, "Missing arguments after function name in function declaration.")
                    );
                    let params: Vec<FunctionParam> = if matches!(params.item.value, TokenKind::FunctionParamsOpen) {
                        match_unreachable!(Some(ChildrenType::FunctionParams(params)) = params.children, params)
                    } else {
                        error::invalid_argument(&op_node.item.value, &params.item, source, "Expected a list of arguments enclosed in parentheses after function name in function declaration.");
                    };
                    
                    // Extract the arrow -> or the function body
                    let node_after_params = extract_right!().unwrap_or_else(
                        || error::expected_argument(&op_node.item, source, "Missing arrow or function body after function parameters in function declaration.")
                    );

                    // Check if it's an arrow -> or a function body, return the function body
                    let body_node = if matches!(node_after_params.item.value, TokenKind::Arrow) {
                        // Extract the function return type
                        let return_type_node = extract_right!().unwrap_or_else(
                            || error::expected_argument(&op_node.item, source, "Missing return type after arrow in function declaration.")
                        );
                        return_type = match_or!(TokenKind::DataType(data_type) = return_type_node.item.value, data_type.into(),
                            error::invalid_argument(&op_node.item.value, &return_type_node.item, source, "Invalid return type in function declaration.")
                        );

                        // The body node is the one after the return type
                        extract_right!()
                    } else {
                        // If there is no arrow ->, the node_after_params is the function body
                       Some(node_after_params)
                    };

                    // The body is one top-level scope token because it gets parsed first
                    let body_node = body_node.unwrap_or_else(
                        || error::expected_argument(&op_node.item, source, "Missing body after return type in function declaration.")
                    );
                    let body: ScopeBlock = if matches!(body_node.item.value, TokenKind::ScopeOpen) {
                        match_unreachable!(Some(ChildrenType::ParsedBlock(body)) = body_node.children, body) // Require the body to be already parsed
                    } else {
                        error::invalid_argument(&op_node.item.value, &body_node.item, source, "Expected a function body enclosed in curly braces.");
                    };
                    
                    let signature: Rc<DataType> = DataType::Function { 
                        params: params.iter().map(|param| param.data_type.clone()).collect(), // Take only the data type of the parameter
                        return_type: return_type.clone() // Here we clone the Rc pointer, not the DataType value
                    }.into();

                    // Declare the function parameter names in the function's scope
                    for param in params {
                        let (_discriminant, res) = symbol_table.declare_symbol(
                            param.name, 
                            Symbol::new_uninitialized(
                                param.data_type,
                                op_node.item.token.clone(),
                                if param.mutable { SymbolValue::Mutable } else { SymbolValue::Immutable(None) },
                            ), 
                            body.scope_id
                        );
                        if let Some(warning) = res.warning() {
                            error::warn(&name_node.item.token, source, warning);
                        }
                    }

                    let (_discriminant, res) = symbol_table.declare_symbol(
                        function_name.to_string(),
                        Symbol::new_uninitialized(
                            signature.clone(), 
                            op_node.item.token.clone(),
                            SymbolValue::Immutable(None), 
                        ),
                        block.scope_id
                    );
                    if let Some(warning) = res.warning() {
                        error::warn(&name_node.item.token, source, warning);
                    }

                    op_node.children = Some(ChildrenType::Function { name: function_name, signature, body });
                },

                TokenKind::FunctionParamsOpen => {
                    // Syntax: (<name>: <type>, <name>: <type>, ...)

                    let mut params: Vec<FunctionParam> = Vec::new();

                    let mut expected_comma: bool = false;
                    let mut mutable: bool = false;
                    loop {
                        let param_node = extract_right!().unwrap_or_else(
                            || error::expected_argument(&op_node.item, source, format!("Missing parameter or closing delimiter for operator {:?}.", op_node.item.value).as_str())
                        );

                        match param_node.item.value {

                            TokenKind::ParClose => break,

                            TokenKind::Comma => if expected_comma {
                                // Set to false because you cannot have two adjacent commas
                                expected_comma = false;
                            } else {
                                error::unexpected_token(&param_node.item, source, "Did you add an extra comma?")
                            },

                            TokenKind::Mut => {
                                if mutable {
                                    error::syntax_error(&param_node.item, source, "Cannot have two adjacent mut keywords in function declaration.");
                                }
                                mutable = true;
                            },

                            TokenKind::Value(Value::Symbol { name, .. }) => {

                                let name = name.to_string();

                                // Extract the colon
                                let colon_node = extract_right!().unwrap_or_else(
                                    || error::expected_argument(&op_node.item, source, format!("Missing colon after parameter name {:?} in function declaration.", name).as_str())
                                );
                                if !matches!(colon_node.item.value, TokenKind::Colon) {
                                    error::invalid_argument(&op_node.item.value, &colon_node.item, source, "Expected a semicolon after parameter name")
                                }
                                
                                let data_type_node = extract_right!().unwrap_or_else(
                                    || error::expected_argument(&op_node.item, source, "Missing data type after colon in function declaration.")
                                );
                                let data_type: Rc<DataType> = match_or!(TokenKind::DataType(data_type) = data_type_node.item.value, data_type,
                                    error::invalid_argument(&op_node.item.value, &data_type_node.item, source, "Invalid data type in function declaration.")
                                ).into();

                                params.push(FunctionParam { name, data_type, mutable });

                                // A comma is expected after each argument except the last one
                                expected_comma = true;
                                mutable = false;
                            },

                            _ => unreachable!("Invalid token kind during statement hierarchy parsing: {:?}. This token kind shouldn't have children.", param_node.item.value)
                        }
                    }

                    op_node.children = Some(ChildrenType::FunctionParams(params));
                },

                TokenKind::ArrayTypeOpen => {
                    // TODO: an array slice may be implemented at a later date. this array type would then require a size (or infer it from the context)
                    // Syntax: [<type>]

                    let element_type_node = extract_right!().unwrap_or_else(
                        || error::expected_argument(&op_node.item, source, "Missing data type after array type open bracket in array declaration.")
                    );
                    let element_type = match_or!(TokenKind::DataType(data_type) = element_type_node.item.value, data_type,
                        error::invalid_argument(&op_node.item.value, &element_type_node.item, source, "Invalid data type in array declaration.")
                    );
                    
                    // Extract the closing square bracket ]
                    let closing_bracket = extract_right!().unwrap_or_else(
                        || error::expected_argument(&op_node.item, source, "Missing array type close bracket after array type in array declaration.")
                    );
                    if !matches!(closing_bracket.item.value, TokenKind::SquareClose) {
                        error::invalid_argument(&op_node.item.value, &closing_bracket.item, source, "Expected closing square bracket ].");
                    }
                    
                    let array_type = DataType::Array(Rc::new(element_type));
                    // Transform this node into a data type node
                    op_node.item.value = TokenKind::DataType(array_type);
                },  

                TokenKind::RefType => {
                    // Syntax: &<type>
                    // Syntax: &mut <type>

                    let next_node = extract_right!().unwrap_or_else(
                        || error::expected_argument(&op_node.item, source, "Missing data type or mutability after reference symbol &.")
                    );

                    let (mutable, element_type_node) = if let TokenKind::Mut = next_node.item.value {
                        // This is a mutable reference
                        let element_type_node = extract_right!().unwrap_or_else(
                            || error::expected_argument(&op_node.item, source, "Missing data type after reference symbol &.")
                        );
                        (true, element_type_node)
                    } else {
                        // This is an immutable reference
                        (false, next_node)
                    };
                    
                    let element_type = match_or!(TokenKind::DataType(data_type) = &element_type_node.item.value, data_type,
                        error::invalid_argument(&op_node.item.value, &element_type_node.item, source, "Expected a data type after reference symbol &.")
                    );

                    // Some data types should be merged together
                    let ref_type = match element_type {
                        DataType::RawString { length } => {
                            if mutable {
                                error::invalid_argument(&op_node.item.value, &element_type_node.item, source, "Raw strings cannot be mutable.")
                            }
                            DataType::StringRef { length: *length }
                        },
                        _ => {
                            let element_type = match_unreachable!(TokenKind::DataType(data_type) = element_type_node.item.value, data_type);
                            DataType::Ref { target: element_type.into(), mutable }
                        }
                    };
                    
                    // Transform this node into a data type node
                    op_node.item.value = TokenKind::DataType(ref_type);
                },

                TokenKind::As => {
                    // Syntax: <expression> as <type>

                    let expr = extract_left!().unwrap_or_else(
                        || error::expected_argument(&op_node.item, source, format!("Missing expression before operator {:?}.", op_node.item.value).as_str())
                    );
                    if !is_expression(&expr.item.value) {
                        error::invalid_argument(&op_node.item.value, &expr.item, source, "Expected an expression.")
                    }
                    
                    let data_type_node = extract_right!().unwrap_or_else(
                        || error::expected_argument(&op_node.item, source, "Missing data type after type cast operator in type cast.")
                    );
                    let data_type = match_or!(TokenKind::DataType(data_type) = data_type_node.item.value, data_type,
                        error::invalid_argument(&op_node.item.value, &data_type_node.item, source, "Expected a data type after type cast operator.")
                    );

                    op_node.children = Some(ChildrenType::TypeCast { data_type: data_type.into(), expr });
                },

                TokenKind::If => {
                    // Syntax: if <condition> { <body> } [else if <condition> { <body> }]... [else { <body> }]

                    let mut if_chain: Vec<IfBlock> = Vec::new();
                    let mut else_block: Option<ScopeBlock> = None;

                    // The if operator node that is currently being parsed. Used for displaying correct error messages.
                    let mut reference_if_operator: Option<Box<TokenNode>> = None;

                    // Parse the if-else chain
                    loop {

                        let condition_node = extract_right!().unwrap_or_else(
                            || error::expected_argument(&reference_if_operator.as_ref().map(|node| node.as_ref()).unwrap_or(op_node).item, source, "Missing condition after if operator.")
                        );
                        if !is_expression(&condition_node.item.value) {
                            error::type_error(&condition_node.item, &[&DataType::Bool.name()], &condition_node.data_type, source, "Expected a boolean condition after if.");
                        }

                        let mut if_body_node = extract_right!().unwrap_or_else(
                            || error::expected_argument(&reference_if_operator.as_ref().map(|node| node.as_ref()).unwrap_or(op_node).item, source, "Missing body after condition in if statement.")
                        );
                        if !matches!(if_body_node.item.value, TokenKind::ScopeOpen) {
                            error::invalid_argument(&reference_if_operator.as_ref().map(|node| node.as_ref()).unwrap_or(op_node).item.value, &if_body_node.item, source, "Expected a body enclosed in curly braces after condition in if statement.");
                        }

                        if_chain.push(
                            IfBlock {
                                condition: *condition_node,
                                body: match_unreachable!(Some(ChildrenType::ParsedBlock(body)) = if_body_node.children.take(), body)
                            }
                        );

                        // Check for further else-if blocks 
                        // Use unsafe to circumvent the borrow checker not recognizing that the borrow ends right after the condition is checked
                        if !matches!(unsafe { op_node.right.as_ref() }.map(|node| &node.item.value), Some(TokenKind::Else)) {
                            // Next node is not an else branch, stop parsing the if-else chain
                            break;
                        }
                        let else_node = extract_right!().unwrap(); // Unwrap is guaranteed to succeed because of the previous check

                        let mut after_else_node = extract_right!().unwrap_or_else(
                            || error::expected_argument(&else_node.item, source, "Missing body after else.")
                        );

                        match after_else_node.item.value {
                            TokenKind::If => {
                                // Continue parsing the if-else chain
                                // Update the reference if to this if node (for displaying correct error messages)
                                reference_if_operator = Some(after_else_node);
                            },
                            TokenKind::ScopeOpen => {
                                // if-else chain is finished
                                else_block = Some(match_unreachable!(Some(ChildrenType::ParsedBlock(body)) = after_else_node.children.take(), body));
                                break;
                            },
                            _ => error::invalid_argument(&else_node.item.value, &after_else_node.item, source, "Expected an if or a body after else.")
                        }
                    }

                    op_node.children = Some(ChildrenType::IfChain { if_chain, else_block })
                },

                TokenKind::Loop => {
                    // Syntax: loop { <body> }

                    let mut body_node = extract_right!().unwrap_or_else(
                        || error::expected_argument(&op_node.item, source, "Missing body after loop.")
                    );
                    if !matches!(body_node.item.value, TokenKind::ScopeOpen) {
                        error::invalid_argument(&op_node.item.value, &body_node.item, source, "Expected a body enclosed in curly braces after loop.");
                    }

                    let scope_block = match_unreachable!(Some(ChildrenType::ParsedBlock(scope_block)) = body_node.children.take(), scope_block);
                    op_node.children = Some(ChildrenType::ParsedBlock(scope_block));
                },

                TokenKind::While => {
                    // Syntax: while <condition> { <body> }

                    let condition_node = extract_right!().unwrap_or_else(
                        || error::expected_argument(&op_node.item, source, "Missing condition after while operator.")
                    );
                    if !is_expression(&condition_node.item.value) {
                        error::type_error(&condition_node.item, &[&DataType::Bool.name()], &condition_node.data_type, source, "Expected a boolean condition after while.");
                    }

                    let mut body_node = extract_right!().unwrap_or_else(
                        || error::expected_argument(&op_node.item, source, "Missing body after condition in while loop.")
                    );
                    if !matches!(body_node.item.value, TokenKind::ScopeOpen) {
                        error::invalid_argument(&op_node.item.value, &body_node.item, source, "Expected a body enclosed in curly braces after condition in while loop.");
                    }

                    let scope_block = match_unreachable!(Some(ChildrenType::ParsedBlock(scope_block)) = body_node.children.take(), scope_block);
                    op_node.children = Some(ChildrenType::While { condition: condition_node, body: scope_block });
                },

                _ => unreachable!("Invalid token kind during statement hierarchy parsing: {:?}.", op_node.item.value)
            }
        }

        while let Some(top_node) = statement.extract_first() {
            parsed_scope_block.statements.push(*top_node);
        }

    }

    parsed_scope_block
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

        let arg_node = tokens.extract_node(start_delimiter.right).unwrap_or_else(
            || error::expected_argument(&start_delimiter.item, source, format!("Missing argument or closing delimiter for operator {:?}.", operator).as_str())
        );

        match &arg_node.item.value {

            t if std::mem::discriminant(t) == std::mem::discriminant(delimiter) => break,

            TokenKind::Comma => if expected_comma {
                // Set to false because you cannot have two adjacent commas
                expected_comma = false;
            } else {
                error::unexpected_token(&arg_node.item, source, "Did you add an extra comma?");
            },

            _ => {
                // The token type will be checked later
                arguments.push(*arg_node);
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
fn resolve_expression_types(expression: &mut TokenNode, scope_id: ScopeID, outer_function_return: Option<Rc<DataType>>, symbol_table: &mut SymbolTable, source: &IRCode) {

    /// Assert that, if the node is a symbol, it is initialized.
    /// Not all operators require their operands to be initialized (l_value of assignment, ref)
    macro_rules! require_initialized {
        ($x:expr) => {
            if let TokenKind::Value(Value::Symbol { name, scope_discriminant }) = $x.item.value {
                let symbol = symbol_table.get_symbol(scope_id, name, scope_discriminant).unwrap().borrow();
                if !symbol.initialized {
                    error::use_of_uninitialized_value(&$x.item, &$x.data_type, source, format!("Cannot use uninitialized value \"{name}\".\nType of \"{name}\": {}.\n{name} declared at {}:{}:\n{}", symbol.data_type, symbol.line_number(), symbol.token.column, source[symbol.token.line_index]).as_str());
                }
            }
        };
    }

    expression.data_type = match &expression.item.value {

        TokenKind::Op(operator) => {
            // Resolve and check the types of the operands first
            // Based on the operand values, determine the type of the operator
            
            match operator {

                Ops::Break |
                Ops::Continue
                 => DataType::Void.into(),

                Ops::Deref { .. } => {
                    let operand = match_unreachable!(Some(ChildrenType::Unary(operand)) = &mut expression.children, operand);

                    resolve_expression_types(operand, scope_id, outer_function_return, symbol_table, source);

                    if let DataType::Ref { mutable, target } = operand.data_type.as_ref() {
                        let mutable_field = match_unreachable!(TokenKind::Op(Ops::Deref { mutable }) = &mut expression.item.value, mutable);
                        *mutable_field = *mutable;
                        target.clone()
                    } else {
                        error::type_error(&operand.item, &[&DataType::Ref { target: DataType::Unspecified.into(), mutable: false }.name()], &operand.data_type, source, "Can only dereference a reference")
                    }
                },

                Ops::Ref { mutable } => {
                    let operand = match_unreachable!(Some(ChildrenType::Unary(operand)) = &mut expression.children, operand);

                    resolve_expression_types(operand, scope_id, outer_function_return, symbol_table, source);

                    if let TokenKind::Value(Value::Symbol { name, scope_discriminant }) = &operand.item.value {
                        let symbol = symbol_table.get_symbol(scope_id, name, *scope_discriminant).unwrap().borrow();
                        // Mutable borrows of immutable symbols are not allowed
                        if !symbol.is_mutable() && *mutable {
                            error::illegal_mutable_borrow(&operand.item, source, format!("Cannot borrow \"{name}\" as mutable because it was declared as immutable.\nType of \"{name}\": {}.\n{name} declared at {}:{}:\n{}", symbol.data_type, symbol.line_number(), symbol.token.column, source[symbol.token.line_index]).as_str())
                        }
                    }

                    DataType::Ref { target: operand.data_type.clone(), mutable: *mutable }.into()
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
                 => {
                    let (op1, op2) = match_unreachable!(Some(ChildrenType::Binary(op1, op2)) = &mut expression.children, (op1, op2));

                    resolve_expression_types(op1, scope_id, outer_function_return.clone(), symbol_table, source);
                    resolve_expression_types(op2, scope_id, outer_function_return, symbol_table, source);

                    require_initialized!(op1);
                    require_initialized!(op2);

                    if !operator.is_allowed_type(&op1.data_type, 0) {
                        error::type_error(&op1.item, operator.allowed_types(0), &op1.data_type, source, format!("Data type is not allowed for operator {}.", operator).as_str())
                    }
                    if !operator.is_allowed_type(&op2.data_type, 1) {
                        error::type_error(&op2.item, operator.allowed_types(1), &op2.data_type, source, format!("Data type is not allowed for operator {}.", operator).as_str())
                    }

                    // Operands must have the same type
                    if op1.data_type != op2.data_type {
                        error::type_error(&op2.item, &[&op1.data_type.name()], &op2.data_type, source, format!("Operator {:?} has operands of different types {:?} and {:?}.", operator, op1.data_type, op2.data_type).as_str());
                    }

                    DataType::Bool.into()
                },

                // Unary operators that return a boolean
                Ops::LogicalNot => {
                    let operand = match_unreachable!(Some(ChildrenType::Unary(operand)) = &mut expression.children, operand);

                    resolve_expression_types(operand, scope_id, outer_function_return, symbol_table, source);

                    require_initialized!(operand);

                    if !operator.is_allowed_type(&operand.data_type, 0) {
                        error::type_error(&operand.item, operator.allowed_types(0), &operand.data_type, source, format!("Data type is not allowed for operator {}.", operator).as_str())
                    }

                    DataType::Bool.into()
                },

                // Unary operators whose return type is the same as the operand type
                Ops::BitwiseNot => {
                    let operand = match_unreachable!(Some(ChildrenType::Unary(operand)) = &mut expression.children, operand);

                    resolve_expression_types(operand, scope_id, outer_function_return, symbol_table, source);

                    require_initialized!(operand);

                    if !operator.is_allowed_type(&operand.data_type, 0) {
                        error::type_error(&operand.item, operator.allowed_types(0), &operand.data_type, source, format!("Data type is not allowed for operator {}.", operator).as_str())
                    }

                    operand.data_type.clone()
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
                 => {
                    let (op1, op2) = match_unreachable!(Some(ChildrenType::Binary(op1, op2)) = &mut expression.children, (op1, op2));

                    resolve_expression_types(op1, scope_id, outer_function_return.clone(), symbol_table, source);
                    resolve_expression_types(op2, scope_id, outer_function_return, symbol_table, source);

                    require_initialized!(op1);
                    require_initialized!(op2);

                    if !operator.is_allowed_type(&op1.data_type, 0) {
                        error::type_error(&op1.item, operator.allowed_types(0), &op1.data_type, source, format!("Data type is not allowed for operator {}.", operator).as_str())
                    }
                    if !operator.is_allowed_type(&op2.data_type, 1) {
                        error::type_error(&op2.item, operator.allowed_types(1), &op2.data_type, source, format!("Data type is not allowed for operator {}.", operator).as_str())
                    }

                    // Check if the operands have the same type
                    if op1.data_type != op2.data_type {
                        // Here ot.clone() is acceptable because the program will exit after this error
                        error::type_error(&op2.item, &[&op1.data_type.name()], &op2.data_type, source, format!("Operator {:?} has operands of different types {:?} and {:?}.", operator, op1.data_type, op2.data_type).as_str());
                    }

                    op1.data_type.clone()
                },

                Ops::FunctionCallOpen => {
                    let (callable, args) = match_unreachable!(Some(ChildrenType::Call { callable, args }) = &mut expression.children, (callable, args));
                    
                    // Resolve the type of the callable operand
                    resolve_expression_types(callable, scope_id, outer_function_return.clone(), symbol_table, source);

                    require_initialized!(callable);

                    // Check if the callable operand is indeed callable (a function symbol or a function pointer)
                    let (param_types, return_type) = match callable.data_type.as_ref() {

                        DataType::Function { params, return_type } => (params, return_type.clone()),
                        DataType::Ref { target, .. } if matches!(**target, DataType::Function { .. }) => if let DataType::Function { params, return_type } = &**target {
                            (params, return_type.clone())
                        } else {
                            unreachable!("Invalid data type during expression type resolution: {:?}. This is a bug.", target)
                        },

                        _ => error::type_error(&callable.item, &[&DataType::Function { params: Vec::new(), return_type: DataType::Void.into() }.name()], &callable.data_type, source, "Expected a function name or a function pointer.")
                    };

                    // Check if the number of arguments matches the number of parameters
                    // Check this before resolving the types of the arguments to avoid unnecessary work
                    if args.len() != param_types.len() {
                        error::mismatched_call_arguments(&expression.item, param_types.len(), args.len(), source, "Invalid number of arguments for function call.");
                    }                    

                    // Resolve the types of the arguments and check if they match the function parameters
                    for (arg, expected_type) in args.iter_mut().zip(param_types) {
                        resolve_expression_types(arg, scope_id, outer_function_return.clone(), symbol_table, source);

                        if arg.data_type != *expected_type {
                            error::type_error(&arg.item, &[&expected_type.name()], &arg.data_type, source, "Argument type does not match function signature.");
                        }
                    }

                    // The return type of the function call is the return type of the function
                    return_type
                },

                Ops::Return => {

                    // A return statement is only allowed inside a function
                    let return_type = outer_function_return.as_ref().unwrap_or_else(
                        || error::syntax_error(&expression.item, source, "Return statement is not allowed outside a function.")
                    ).clone();

                    // Resolve the type of the return value, if any
                    if let Some(children) = &mut expression.children {

                        let return_expr = if let ChildrenType::Unary (children) = children { children } else { unreachable!(); };

                        resolve_expression_types(return_expr, scope_id, outer_function_return, symbol_table, source);

                        require_initialized!(return_expr);
                        
                        // Check if the return type matches the outer function return type
                        if return_expr.data_type != return_type {
                            error::type_error(&return_expr.item, &[&return_type.name()], &return_expr.data_type, source, "The returned expression type does not match function signature.");
                        }
                    } else if !matches!(*return_type, DataType::Void) {
                        // If the function doesn't return void, return statements must have a return value
                        error::type_error(&expression.item, &[&return_type.name()], &DataType::Void, source, "Missing return value for function that does not return void.");
                    }

                    // A return statement evaluates to void
                    DataType::Void.into()
                },

                Ops::Assign => {
                    
                    let (l_node, r_node) = match_unreachable!(Some(ChildrenType::Binary (l_node, r_node)) = &mut expression.children, (l_node, r_node));
                    
                    // Only allow assignment to a symbol or a dereference
                    if !matches!(l_node.item.value, TokenKind::Value(Value::Symbol { .. }) | TokenKind::Op(Ops::Deref { .. })) {
                        error::type_error(&l_node.item, Ops::Assign.allowed_types(0), &l_node.data_type, source, "Invalid left operand for assignment operator.");
                    }

                    // Resolve the types of the operands
                    resolve_expression_types(l_node, scope_id, outer_function_return.clone(), symbol_table, source);
                    resolve_expression_types(r_node, scope_id, outer_function_return, symbol_table, source);

                    require_initialized!(r_node);
                
                    // Assert that the symbol or dereference can be assigned to (mutable or uninitialized)
                    match &l_node.item.value {
                        TokenKind::Value(Value::Symbol { name, scope_discriminant }) => {

                            // Unwrap is safe because symbols have already been checked to be valid
                            let mut symbol = symbol_table.get_symbol(scope_id, name, *scope_discriminant).unwrap().borrow_mut();
                            
                            if symbol.initialized {
                                if matches!(symbol.value, SymbolValue::Immutable(_)) {
                                    // Symbol is immutable and already initialized, so cannot assign to it again
                                    error::immutable_change(&l_node.item, &l_node.data_type, source, "Cannot assign to an immutable symbol.");
                                }
                            } else {
                                // Symbol was not initialized, so set it as initialized now
                                symbol.initialized = true;
                            }

                            // The data type must be inferred now if it wasn't specified earlier
                            if matches!(*symbol.data_type, DataType::Unspecified) {
                                symbol.data_type = r_node.data_type.clone();
                                l_node.data_type = r_node.data_type.clone();
                            }
                        },

                        TokenKind::Op(Ops::Deref{ mutable }) => {
                            // The dereference must be mutable
                            if !mutable {
                                error::immutable_change(&l_node.item, &l_node.data_type, source, "Cannot assign to an immutable dereference.");
                            }
                        },

                        _ => unreachable!("Invalid token kind during expression type resolution: {:?}. This is a bug.", l_node.item.value)
                    }

                    // Check if the symbol type and the expression type are compatible (the same or implicitly castable)
                    let r_value = r_node.item.value.literal_value();
                    if !r_node.data_type.is_implicitly_castable_to(&l_node.data_type, r_value) {
                        error::type_error(&r_node.item, &[&l_node.data_type.name()], &r_node.data_type, source, "Mismatched right operand type for assignment operator.");
                    }
                    
                    // An assignment is not an expression, so it does not have a type
                    DataType::Void.into()
                },

                Ops::ArrayIndexOpen => {
                    // The data type of an array subscription operation is the type of the array elements

                    let (array_node, index_node) = match_unreachable!(Some(ChildrenType::Binary (array_node, index_node )) = &mut expression.children, (array_node, index_node));

                    resolve_expression_types(array_node, scope_id, outer_function_return.clone(), symbol_table, source);
                    resolve_expression_types(index_node, scope_id, outer_function_return, symbol_table, source);

                    require_initialized!(array_node);
                    require_initialized!(index_node);

                    let data_type = match_or!(DataType::Array(element_type) = array_node.data_type.as_ref(), element_type.clone(),
                        error::type_error(&array_node.item, Ops::ArrayIndexOpen.allowed_types(0), &array_node.data_type, source, "Can only index arrays.")
                    );

                    // Assert that the array index is an unsigned integer
                    if !Ops::ArrayIndexOpen.is_allowed_type( &index_node.data_type, 1) {
                        error::type_error(&index_node.item, Ops::ArrayIndexOpen.allowed_types(1), &index_node.data_type, source, "Array index must strictly be an unsigned integer.");
                    }
                    
                    data_type
                }
            }
        },

        TokenKind::As => {
            let (target_type, expr) = match_unreachable!(Some(ChildrenType::TypeCast { data_type, expr }) = &mut expression.children, (data_type, expr));

            // Resolve the type of the expression to be cast
            resolve_expression_types(expr, scope_id, outer_function_return, symbol_table, source);

            require_initialized!(expr);

            if expr.data_type == *target_type {
                
                error::warn(&expression.item.token, source, "Redundant type cast. Expression is already of the specified type.")

            } else {
                    // Check if the expression type can be cast to the specified type
                if !expr.data_type.is_castable_to(target_type) {
                    error::type_error(&expr.item, &[&target_type.name()], &expr.data_type, source, format!("Type {:?} cannot be cast to {:?}.", expr.data_type, target_type).as_str());
                }

                // Evaluates to the data type of the type cast
            }

            target_type.clone()
        },

        TokenKind::Value(value) => match value {

            Value::Literal { value } => value.data_type(symbol_table).into(),

            Value::Symbol { name, scope_discriminant } => {
                let mut symbol = symbol_table.get_symbol(scope_id, name, *scope_discriminant).unwrap_or_else(
                    || error::symbol_undefined(&expression.item, name, source, if let Some(symbol) = symbol_table.get_unreachable_symbol(name) { let symbol = symbol.borrow(); format!("Symbol \"{name}\" is declared in a different scope at {}:{}:\n{}.", symbol.line_number(), symbol.token.column, source[symbol.token.line_index]) } else { format!("Symbol \"{name}\" is not declared in any scope.") }.as_str())
                ).borrow_mut();

                // The symbol has beed used in an expression, so it has been read from.
                // If the symbol is being assigned to instead, the Ops::Assign branch will set this flag to false later.
                // This is not an ideal design choice, but it works.
                symbol.read_from = true;

                symbol.data_type.clone()
            }
        },

        TokenKind::Fn => {
            // Resolve the types inside the function body
            
            let (signature, body) = match_unreachable!(Some(ChildrenType::Function { signature, body, .. }) = &mut expression.children, (signature, body));
            let return_type = match_unreachable!(DataType::Function { return_type, .. } = signature.as_ref(), return_type);

            resolve_scope_types(body, Some(return_type.clone()), symbol_table, source);

            // Check return type
            let return_value = body.return_value_literal();
            if !body.return_type().is_implicitly_castable_to(return_type.as_ref(), return_value) {
                error::type_error(&expression.item, &[&return_type.name()], &body.return_type(), source, "Mismatched return type in function declaration.");
            }

            // Function declaration does not have any type
            DataType::Void.into()
        },

        TokenKind::ArrayOpen => {
            // Recursively resolve the types of the array elements.
            // The array element type is the type of the first element.
            // Check if the array elements have the same type.
            // The array element type is void if the array is empty. A void array can be used as a generic array by assignment operators.

            let elements = match_unreachable!(Some(ChildrenType::List(elements)) = &mut expression.children, elements);
            
            let (data_type, is_literal_array, element_type) = if elements.is_empty() {
                (DataType::Array(DataType::Void.into()), true, DataType::Void.into())
            } else {

                let mut element_type: Option<Rc<DataType>> = None;

                let mut is_literal_array = true;
                for element in elements {
                    
                    // Resolve the element type
                    resolve_expression_types(element, scope_id, outer_function_return.clone(), symbol_table, source);

                    require_initialized!(element);

                    let expr_type = element.data_type.clone();

                    if let Some(expected_element_type) = &element_type {
                        if *expected_element_type != expr_type {
                            error::type_error(&element.item, &[&expected_element_type.name()], &expr_type, source, "Array elements have different types.");
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

                (DataType::Array(element_type.as_ref().unwrap().clone()), is_literal_array, element_type.unwrap())
            };

            if is_literal_array {
                // If all the array elements are literals, the whole array is a literal array
                // Change this token to a literal array
                let elements = expression.children.take().map(|children| if let ChildrenType::List(elements) = children { elements } else { unreachable!("ArrayOpen node should have children of type ChildrenType::List, but the expression {:?} has children of type {:?} instead. This is a bug.", expression, expression.children) }).unwrap();
                let mut literal_items: Vec<LiteralValue> = Vec::with_capacity(elements.len());
                for element in elements {
                    literal_items.push(if let TokenKind::Value(Value::Literal { value }) = element.item.value { value } else { unreachable!("All items must be literals") });
                }
                expression.item.value = TokenKind::Value(Value::Literal { value: LiteralValue::Array { element_type, items: literal_items } });
            }

            data_type.into()
        },

        TokenKind::ScopeOpen => {
            // Recursively resolve the types of the children statements
            // Determine the type of the scope based on the type of the last statement
            // If the scope is empty, it evaluates to void

            let inner_block = match_unreachable!(Some(ChildrenType::ParsedBlock(inner_block)) = &mut expression.children, inner_block);

            if inner_block.statements.is_empty() {
                DataType::Void.into()
            } else {
                resolve_scope_types(inner_block, outer_function_return, symbol_table, source);
                inner_block.return_type()
            }
        },

        TokenKind::If => {
            // Recursively resolve the types of the if-else chain
            // The return type of the chain is the return type of the conditional blocks

            let mut chain_return_type: Option<Rc<DataType>> = None;

            let (if_chain, else_block) = match_unreachable!(Some(ChildrenType::IfChain { if_chain, else_block }) = &mut expression.children, (if_chain, else_block));

            for if_block in if_chain {
                resolve_expression_types(&mut if_block.condition, scope_id, outer_function_return.clone(), symbol_table, source);
                resolve_scope_types(&mut if_block.body, outer_function_return.clone(), symbol_table, source);

                require_initialized!(if_block.condition);

                // Check if the return types match
                if let Some(return_type) = &chain_return_type {
                    if if_block.body.return_type() != *return_type {
                        // If the body is not empty, use its last statement as the culprit of the type mismatch. Otherwise, use the if condition.
                        let culprit_token = if let Some(last_statement) = if_block.body.statements.last() {
                            &last_statement.item
                        } else {
                            &expression.item
                        };
                        error::type_error(culprit_token, &[&return_type.name()], &if_block.body.return_type(), source, "Mismatched return type in if-else chain.");
                    }
                } else {
                    chain_return_type = Some(if_block.body.return_type().clone());
                }
            }

            if let Some(else_block) = else_block {
                resolve_scope_types(else_block, outer_function_return, symbol_table, source);

                // Check if the return types match
                // Unwrap is safe because the else block is guaranteed to be preceeded by an if block, which sets the chain_return_type
                if else_block.return_type() != *chain_return_type.as_ref().unwrap() {
                    // If the body is not empty, use its last statement as the culprit of the type mismatch. Otherwise, use the if condition.
                    let culprit_token = if let Some(last_statement) = else_block.statements.last() {
                        &last_statement.item
                    } else {
                        &expression.item
                    };
                    error::type_error(culprit_token, &[&chain_return_type.unwrap().name()], &else_block.return_type(), source, "Mismatched return type in if-else chain.");
                }
            }

            // Unwrap is safe because the if-else chain is guaranteed to have at least one if block, which sets the chain_return_type
            chain_return_type.unwrap()
        },

        TokenKind::While => {
            
            let (condition_node, body_block) = match_unreachable!(Some(ChildrenType::While { condition, body }) = &mut expression.children, (condition, body));

            resolve_expression_types(condition_node, scope_id, outer_function_return.clone(), symbol_table, source);
            resolve_scope_types(body_block, outer_function_return, symbol_table, source);

            require_initialized!(condition_node);

            // Assert that the condition is a boolean
            if !matches!(*condition_node.data_type, DataType::Bool) {
                error::type_error(&condition_node.item, &[&DataType::Bool.name()], &condition_node.data_type, source, "While loop condition must be a boolean.");
            }

            DataType::Void.into()
        }

        _ => unreachable!("Unexpected syntax node during expression and symbol type resolution: {:?}. This is a bug.", expression)
    };
}


/// Resolve and check the types of symbols and expressions.
fn resolve_scope_types(block: &mut ScopeBlock, outer_function_return: Option<Rc<DataType>>, symbol_table: &mut SymbolTable, source: &IRCode) {
    // Perform a depth-first traversal of the scope tree to determine the types in a top-to-bottom order (relative to the source code).
    // For every node in every scope, determine the node data type and check if it matches the expected type.

    for statement in &mut block.statements {

        resolve_expression_types(statement, block.scope_id, outer_function_return.clone(), symbol_table, source);

    }
}


fn warn_unused_symbols(block: &ScopeBlock, symbol_table: &SymbolTable, source: &IRCode) {
    for (name, symbol) in symbol_table.get_unread_symbols(block.scope_id) {
        let token = &symbol.borrow().token;
        warn(token, source, format!("Symbol \"{name}\" is declared but never used.\nDeclaration occurs at {}:{}:\n\n{}\n", token.line_number(), token.column, &source[token.line_index()]).as_str());
    }
}


/// Reduce the operations down the node by evaluating constant expressions.
/// 
/// Return whether the node can be removed because it has no effect.
fn evaluate_constants(node: &mut TokenNode, source: &IRCode, scope_id: ScopeID, symbol_table: &mut SymbolTable) -> bool {

    macro_rules! extract_constant_value {
        ($node:expr) => {
            match $node.item.value {
                TokenKind::Value(Value::Literal { value }) => Some(value),

                TokenKind::Value(Value::Symbol { name, scope_discriminant }) => {
                    let symbol = symbol_table.get_symbol(scope_id, name, scope_discriminant).unwrap();
                    symbol.borrow().get_value().cloned()
                }
                
                _ => unreachable!()
            }
        };
    }

    macro_rules! has_constant_value {
        ($node:expr) => {
            match &$node.item.value {
                TokenKind::Value(Value::Literal { .. }) => true,

                TokenKind::Value(Value::Symbol { name, scope_discriminant })
                 => symbol_table.get_symbol(scope_id, name, *scope_discriminant).unwrap().borrow().get_value().is_some(),
                    
                _ => unreachable!()
            }
        };
    }

    const SHOULD_BE_REMOVED: bool = true;
    const SHOULD_NOT_BE_REMOVED: bool = false;

    match node.item.value {
        TokenKind::Op(op) => match op {

            // These operators are only allowed in runtime for obvious reasons
            Ops::Break |
            Ops::Continue
             => return SHOULD_NOT_BE_REMOVED,

            Ops::ArrayIndexOpen => {
                let (op1, op2) = match_unreachable!(Some(ChildrenType::Binary(op1, op2)) = &mut node.children, (op1, op2));

                evaluate_constants(op1, source, scope_id, symbol_table);
                evaluate_constants(op2, source, scope_id, symbol_table);

                // TODO: implement compile-time array indexing for literal arrays and initialized immutable arrays
            },

            Ops::Assign => {
                let (l_node, r_node) = match_unreachable!(Some(ChildrenType::Binary(l_node, r_node)) = &mut node.children, (l_node, r_node));

                evaluate_constants(l_node, source, scope_id, symbol_table);
                evaluate_constants(r_node, source, scope_id, symbol_table);

                // If the left operand is a plain symbol and the right operand is known, perform the assignment statically
                if let (TokenKind::Value(Value::Symbol { name, scope_discriminant }), true) = (&l_node.item.value, has_constant_value!(r_node)) {
                    
                    let mut l_symbol = symbol_table.get_symbol(scope_id, name, *scope_discriminant).unwrap().borrow_mut();

                    // Only allow to statically initialize immutable symbols
                    if matches!(l_symbol.value, SymbolValue::Immutable(_)) {

                        let r_node = match_unreachable!(Some(ChildrenType::Binary(_l_node, r_node)) = node.children.take(), r_node);
                        let r_value = extract_constant_value!(r_node).unwrap();

                        l_symbol.initialize_immutable(r_value);

                        // The assignment has just been performed statically, so the assignment operation can be removed (assignment operation has no side effect and is not an expression)
                        return SHOULD_BE_REMOVED;
                    }
                }
            },

            #[allow(unreachable_patterns)] // Allow to keep the code concise. Some binary operators are handled differently.
            binary_operators!() => {
                let (op1, op2) = match_unreachable!(Some(ChildrenType::Binary(op1, op2)) = &mut node.children, (op1, op2));

                evaluate_constants(op1, source, scope_id, symbol_table);
                evaluate_constants(op2, source, scope_id, symbol_table);

                if !op.is_allowed_at_compile_time() || (op1.item.value.literal_value().is_none() && op2.item.value.literal_value().is_none()) {
                    return SHOULD_NOT_BE_REMOVED;
                }

                // .take() is ok because the children will be dropped after the operation
                let (op1, op2) = match_unreachable!(Some(ChildrenType::Binary(op1, op2)) = node.children.take(), (op1, op2));
                
                let (op1_value, op2_value) = if let (Some(v1), Some(v2)) = (extract_constant_value!(op1), extract_constant_value!(op2)) {
                    (v1, v2)
                } else {
                    return SHOULD_NOT_BE_REMOVED;
                };
                
                let res = match op.execute(&[op1_value, op2_value]) {
                    Ok(res) => res,
                    Err(err) => error::compile_time_operation_error(&node.item, source, err)
                };

                node.item.value = TokenKind::Value(Value::Literal { value: res });
            },

            unary_operators!() => {
                let operand = match_unreachable!(Some(ChildrenType::Unary(operand)) = &mut node.children, operand);

                evaluate_constants(operand, source, scope_id, symbol_table);

                if !op.is_allowed_at_compile_time() || operand.item.value.literal_value().is_none() {
                    return SHOULD_NOT_BE_REMOVED;
                }

                // .take() is ok because the child will be dropped after the operation
                let operand = match_unreachable!(Some(ChildrenType::Unary(operand)) = node.children.take(), operand);
                
                let operand_value = if let Some(v) = extract_constant_value!(operand) {
                    v
                } else {
                    return SHOULD_NOT_BE_REMOVED;
                };

                let res = match op.execute(&[operand_value]) {
                    Ok(res) => res,
                    Err(err) => error::compile_time_operation_error(&node.item, source, err)
                };

                node.item.value = TokenKind::Value(Value::Literal { value: res });
            },

            Ops::Return => if let Some(expr) = &mut node.children {
                let expr = match_unreachable!(ChildrenType::Unary(expr) = expr, expr);

                evaluate_constants(expr, source, scope_id, symbol_table);
            },
            
            Ops::FunctionCallOpen => {

                // TODO: evaluate const functions

                let (callable, args) = match_unreachable!(Some(ChildrenType::Call { callable, args }) = &mut node.children, (callable, args));

                evaluate_constants(callable, source, scope_id, symbol_table);

                for arg in args {
                    evaluate_constants(arg, source, scope_id, symbol_table);
                }
            },
        },

        TokenKind::While => {
            let (condition, body) = match_unreachable!(Some(ChildrenType::While { condition, body }) = &mut node.children, (condition, body));

            evaluate_constants(condition, source, scope_id, symbol_table);

            if let Some(condition_value) = condition.item.value.literal_value() {
                let bool_value = match_unreachable!(LiteralValue::Bool(v) = condition_value, v);
                if *bool_value {
                    // The condition is always true, so the body will always be executed
                    // Downgrade the while loop to a unconditional loop

                    error::warn(&condition.item.token, source, "While loop condition is always true. This loop will be converted to an unconditional loop.");

                    evaluate_constants_block(body, source, symbol_table);

                    node.item.value = TokenKind::Loop;
                    node.children = Some(ChildrenType::ParsedBlock(
                        match_unreachable!(Some(ChildrenType::While { body, .. }) = node.children.take(), body)
                    ));
                } else {
                    // Condition is always false, so the body will never be executed
                    // Remove the while loop entirely
                    // Don't worry about side effects in the condition, since expressions with side effects are not evaluated at compile-time
                    return SHOULD_BE_REMOVED;
                }
            }
        },

        TokenKind::As => {
            let (target_type, expr) = match_unreachable!(Some(ChildrenType::TypeCast { expr, data_type }) = &mut node.children, (data_type, expr));

            evaluate_constants(expr, source, scope_id, symbol_table);

            if expr.data_type == *target_type {
                // No need to perform the cast
                // Return directly the expression

                let expr = match_unreachable!(Some(ChildrenType::TypeCast { expr, .. }) = node.children.take(), expr);

                node.substitute(*expr);

                return SHOULD_NOT_BE_REMOVED;
            }
            // The cast is to be performed

            // Cannot cast at compile-time if the expression value is unknown (not a literal)
            if expr.item.value.literal_value().is_none() {
                return SHOULD_NOT_BE_REMOVED;
            }

            let (target_type, expr) = match_unreachable!(Some(ChildrenType::TypeCast { data_type, expr }) = node.children.take(), (data_type, expr));
            let value = match_unreachable!(TokenKind::Value(Value::Literal { value }) = expr.item.value, value);

            let new_value = LiteralValue::from_cast(value, &expr.data_type, &target_type);

            node.item.value = TokenKind::Value(Value::Literal { value: new_value });
            node.data_type = expr.data_type;
        },

        TokenKind::Fn => {
            let body = match_unreachable!(Some(ChildrenType::Function { body, .. }) = &mut node.children, body);

            evaluate_constants_block(body, source, symbol_table);
        },

        TokenKind::If => {
            let (if_chain, else_block) = match_unreachable!(Some(ChildrenType::IfChain { if_chain, else_block }) = &mut node.children, (if_chain, else_block));

            for if_block in if_chain {
                evaluate_constants(&mut if_block.condition, source, scope_id, symbol_table);
                evaluate_constants_block(&mut if_block.body, source, symbol_table);
            }

            if let Some(else_block) = else_block {
                evaluate_constants_block(else_block, source, symbol_table);
            }
        },

        TokenKind::ArrayOpen => {
            let elements = match_unreachable!(Some(ChildrenType::List(elements)) = &mut node.children, elements);

            for element in elements {
                evaluate_constants(element, source, scope_id, symbol_table);
            }
        },

        TokenKind::ScopeOpen => {
            let inner_block = match_unreachable!(Some(ChildrenType::ParsedBlock(inner_block)) = &mut node.children, inner_block);

            evaluate_constants_block(inner_block, source, symbol_table);

            if inner_block.statements.is_empty() {
                // Empty scopes can be removed
                return SHOULD_BE_REMOVED;
            }
        },

        TokenKind::Value(_) => {
            // Values are already reduced to the minimum
        },

        _ => unreachable!("{:?} should have been removed from the tree.", node.item.value)
    }

    // By default, the node should not be removed
    SHOULD_NOT_BE_REMOVED
}


/// Reduce the number of operations by evaluating constant expressions
fn evaluate_constants_block(block: &mut ScopeBlock, source: &IRCode, symbol_table: &mut SymbolTable) {
    // Depth-first traversal to evaluate constant expressions and remove unnecessary operations

    let mut i: usize = 0;
    while let Some(statement) = block.statements.get_mut(i) {

        if evaluate_constants(statement, source, block.scope_id, symbol_table) {
            // Remove the useless node 
            block.statements.remove(i);
        } else {
            i += 1;
        }

    }
}


struct Function<'a> {

    name: &'a str,
    code: ScopeBlock<'a>,
    signature: Rc<DataType>

}

impl std::fmt::Debug for Function<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {:?}", self.name, self.signature)
    }
}


/// Convert the source code trees into a list of functions.
/// Functions declared inside other functions are extracted and added to the list.
/// Functions from inner scopes will not be accessible from outer scopes by default thanks to symbol table scoping, so it's ok to keep them in a linear vector.
fn extract_functions<'a>(block: &mut ScopeBlock<'a>, inside_function: bool, symbol_table: &mut SymbolTable, source: &IRCode) -> Vec<Function<'a>> {

    const DO_EXTRACT: bool = false;
    const DO_NOT_EXTRACT: bool = true;

    let mut functions = Vec::new();

    // Pre-allocate the maximum capacity that can possibly be needed
    let mut old_statements: Vec<TokenNode> = Vec::with_capacity(block.statements.len());
    // Swap the two vectors so that block.statements is the vector statements will get appended to
    // Perform the swap now because rust doesn't like moving out values without replacing them immediately
    // Move would occur in the iterations below
    std::mem::swap(&mut old_statements, &mut block.statements);

    for mut statement in old_statements {

        let keep: bool = match statement.item.value {

            TokenKind::Fn => {
                let (name, signature, mut body) = match_unreachable!(Some(ChildrenType::Function { name, signature, body }) = statement.children.take(), (name, signature, body));
                
                // Recursively extract functions from the body
                let inner_functions = extract_functions(&mut body, true, symbol_table, source);
                functions.extend(inner_functions);

                functions.push(Function {
                    name,
                    code: body,
                    signature
                });

                DO_EXTRACT
            },

            TokenKind::Const => {
                // Evaluate the constant expression and add it to the symbol table

                let (name, discriminant, const_data_type, mut definition) = match_unreachable!(Some(ChildrenType::Const { name, discriminant, data_type, definition }) = statement.children.take(), (name, discriminant, data_type, definition));
                
                resolve_expression_types(&mut definition, block.scope_id, None, symbol_table, source);
                evaluate_constants(&mut definition, source, block.scope_id, symbol_table);

                let literal_value = match &definition.item.value {

                    // Allow initializing constants with literal values
                    TokenKind::Value(Value::Literal { value }) => value.clone(), // TODO: find a way to avoid cloning literal types (maybe use a Rc?)

                    // Allow initializing constants with other initialized constatns
                    TokenKind::Value(Value::Symbol { name, scope_discriminant }) => {
                        let symbol = symbol_table.get_symbol(block.scope_id, name, *scope_discriminant).unwrap().borrow();
                        match &symbol.value {
                            SymbolValue::Constant(value) => value.clone(),
                            _ => error::not_a_constant(&definition.item, source, "Constant definition must be a literal value or a constant expression.")
                        }
                    },
                    
                    _ => error::not_a_constant(&definition.item, source, "Constant definition must be a literal value or a constant expression.")
                };

                let value_type = literal_value.data_type(symbol_table);
                if !value_type.is_implicitly_castable_to(&const_data_type, Some(&literal_value)) {
                    error::type_error(&definition.item, &[&const_data_type.name()], &value_type, source, "Mismatched constant type.");
                }
                let final_value = LiteralValue::from_cast(literal_value, &value_type, &const_data_type);
                
                let res = symbol_table.define_constant(name, discriminant, block.scope_id, final_value);
                if res.is_err() {
                    error::compile_time_operation_error(&statement.item, source, format!("Could not define constant \"{name}\".").as_str());
                }

                DO_EXTRACT
            },

            _ => {
                if !inside_function {
                    error::syntax_error(&statement.item, source, "Cannot be a top-level statement.");
                }

                DO_NOT_EXTRACT
            }

        };

        if keep {
            block.statements.push(statement)
        }
    }

    functions
}


/// Build an abstract syntax tree from a flat list of tokens and create a symbol table.
pub fn build_ast<'a>(mut tokens: TokenTree<'a>, source: &'a IRCode, optimize: bool, symbol_table: &mut SymbolTable<'a>) -> ScopeBlock<'a> {

    parse_scope_hierarchy(&mut tokens);

    let unparsed_outer = divide_statements(tokens, symbol_table, None);

    println!("Statements after division:\n\n{:?}", unparsed_outer);

    let mut parsed_outer = parse_block_hierarchy(unparsed_outer, symbol_table, source);

    println!("\n\nStatement hierarchy:\n{:?}", parsed_outer);

    let functions = extract_functions(&mut parsed_outer, false, symbol_table, source);

    println!("\n\nFunctions:\n{:#?}\n", functions);

    // resolve_scope_types(&mut outer_block, None, symbol_table, source);

    // println!("\n\nAfter symbol resolution:\n{}", outer_block);
    
    // warn_unused_symbols(&parsed_outer, symbol_table, source);

    // if optimize {
    //     reduce_operations_block(&mut outer_block, source, symbol_table);
    //     println!("\n\nAfter constant expression evaluation:\n{}", outer_block);
    // }
    
    // outer_block

    todo!()
}

