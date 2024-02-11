use std::rc::Rc;

use rusty_vm_lib::ir::SourceCode;

use crate::data_types::DataType;
use crate::error;
use crate::operations::Ops;
use crate::token::{TokenKind, Value};
use crate::symbol_table::{NameType, ScopeID, Symbol, SymbolTable, SymbolValue};
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


fn parse_block_hierarchy<'a>(block: UnparsedScopeBlock<'a>, symbol_table: &mut SymbolTable<'a>, source: &SourceCode) -> ScopeBlock<'a> {
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
                    },
    
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

                TokenKind::TypeDef => {
                    // Syntax: typedef <name> = <definition>

                    let name_node = extract_right!().unwrap_or_else(
                        || error::expected_argument(&op_node.item, source, "Missing name in type definition.")
                    );
                    let type_name = match_or!(TokenKind::Value(Value::Symbol { name, .. }) = name_node.item.value, name,
                        error::invalid_argument(&op_node.item.value, &name_node.item, source, "Invalid type name in type definition.")
                    );

                    let assign_node = extract_right!().unwrap_or_else(
                        || error::expected_argument(&op_node.item, source, "Missing assignment operator after type name in type declaration.")
                    );
                    if !matches!(assign_node.item.value, TokenKind::Op(Ops::Assign)) {
                        error::invalid_argument(&op_node.item.value, &assign_node.item, source, "Expected an assignment operator after type name in type definition.")
                    }

                    let definition_node = extract_right!().unwrap_or_else(
                        || error::expected_argument(&op_node.item, source, "Missing data type after assignment operator in type definition.")
                    );
                    let data_type: Rc<DataType> = match_or!(TokenKind::DataType(data_type) = definition_node.item.value, data_type,
                        error::invalid_argument(&op_node.item.value, &definition_node.item, source, "Expected a data type after assignment operator in type definition.")
                    );

                    let res = symbol_table.define_type(type_name.to_string(), block.scope_id, data_type.clone(), op_node.item.token.clone());
                    if let Some(shadow) = res.err() {
                        error::already_defined(&op_node.item.token, &shadow.token, source, format!("{type_name} is defined multiple times in the same scope.").as_str())
                    }

                    // These children aren't really useful, but here they are to keep the code more uniform
                    // Maybe in the future they will be used to evaluate constant expressions in type definitions (e.g. array size)
                    op_node.children = Some(ChildrenType::TypeDef { name: type_name, definition: data_type })
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
                    );

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
                },
    
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
                    let data_type = match after_name.item.value {

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

                            DataType::Unspecified.into()
                        },

                        _ => error::invalid_argument(&op_node.item.value, &after_name.item, source, "Expected a colon or an equal sign after variable name in variable declaration.")
                    };

                    // Declare the new symbol in the local scope
                    let (discriminant, res) = symbol_table.declare_symbol(
                        symbol_name.to_string(),
                        Symbol::new_uninitialized(
                            data_type, 
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

                    // At this stage, Symbol tokens can either be custom types or actual symbols
                    match symbol_table.get_name_type(name, block.scope_id) {

                        Some(NameType::Symbol(current_discriminant)) => {
                            // The name is a symbol
                            let scope_discriminant = match_unreachable!(TokenKind::Value(Value::Symbol { name: _, scope_discriminant }) = &mut op_node.item.value, scope_discriminant);
                            *scope_discriminant = current_discriminant;
                        },

                        Some(NameType::Type(data_type)) => {
                            // The name is a custom type
                            op_node.item.value = TokenKind::DataType(data_type);

                        },

                        _ => {
                            // Undefined symbols will be catched later. Function parameters, for example, would result in an error here.
                        }
                    }
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
                        return_type = match_or!(TokenKind::DataType(data_type) = return_type_node.item.value, data_type,
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
                        Symbol::new_function(
                            signature.clone(), 
                            op_node.item.token.clone(),
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
                                );

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
                    
                    let array_type = DataType::Array(element_type).into();
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
                    let ref_type = match element_type.as_ref() {
                        DataType::RawString { length } => {
                            if mutable {
                                error::invalid_argument(&op_node.item.value, &element_type_node.item, source, "Raw strings cannot be mutable.")
                            }
                            DataType::StringRef { length: *length }.into()
                        },
                        _ => {
                            let element_type = match_unreachable!(TokenKind::DataType(data_type) = element_type_node.item.value, data_type);
                            DataType::Ref { target: element_type, mutable }.into()
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

                    op_node.children = Some(ChildrenType::TypeCast { data_type, expr });
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
fn extract_list_like_delimiter_contents<'a>(tokens: &mut TokenTree<'a>, start_delimiter: *mut TokenNode<'a>, operator: &TokenKind<'_>, delimiter: &TokenKind<'_>, source: &SourceCode) -> Vec<TokenNode<'a>> {
    
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


/// Build an abstract syntax tree from a flat list of tokens
pub fn build_ast<'a>(mut tokens: TokenTree<'a>, source: &'a SourceCode, symbol_table: &mut SymbolTable<'a>) -> ScopeBlock<'a> {

    parse_scope_hierarchy(&mut tokens);

    let unparsed_outer = divide_statements(tokens, symbol_table, None);

    println!("Statements after division:\n\n{:?}", unparsed_outer);

    let parsed_outer = parse_block_hierarchy(unparsed_outer, symbol_table, source);

    println!("\n\nStatement hierarchy:\n{:?}", parsed_outer);

    parsed_outer
}

