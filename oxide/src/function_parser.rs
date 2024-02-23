use std::rc::Rc;

use crate::cli_parser::OptimizationFlags;
use crate::operations::Ops;
use crate::{binary_operators, match_or, match_unreachable, unary_operators};
use crate::token_tree::{ChildrenType, ScopeBlock, TokenNode};
use crate::token::{TokenKind, Value};
use crate::symbol_table::{ScopeID, SymbolTable, SymbolValue};
use crate::data_types::{DataType, LiteralValue};
use crate::error;

use rusty_vm_lib::ir::SourceCode;



pub struct Function<'a> {

    pub name: &'a str,
    pub code: ScopeBlock<'a>,
    pub signature: Rc<DataType>,
    pub parent_scope: ScopeID,
    /// Whether the function has non-local side effects (e.g. modifying a global variable, I/O, etc.)
    pub has_side_effects: bool

}

impl Function<'_> {

    pub fn new<'a>(name: &'a str, body: ScopeBlock<'a>, signature: Rc<DataType>, parent_scope: ScopeID) -> Function<'a> {
        Function {
            name,
            code: body,
            signature,
            parent_scope,
            has_side_effects: false
        }
    }

    pub fn return_type(&self) -> Rc<DataType> {
        match_unreachable!(DataType::Function { return_type, .. } = self.signature.as_ref(), return_type).clone()
    }

}

impl std::fmt::Debug for Function<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {:?}:\n{:?}", self.name, self.signature, self.code)
    }
}


/// Convert the source code trees into a list of functions.
/// Functions declared inside other functions are extracted and added to the list.
/// Functions from inner scopes will not be accessible from outer scopes by default thanks to symbol table scoping, so it's ok to keep them in a linear vector.
fn extract_functions<'a>(block: &mut ScopeBlock<'a>, inside_function: bool, function_parent_scope: ScopeID, symbol_table: &mut SymbolTable, source: &SourceCode) -> Vec<Function<'a>> {

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
                let inner_functions = extract_functions(&mut body, true, block.scope_id, symbol_table, source);
                functions.extend(inner_functions);

                functions.push(Function::new(
                    name,
                    body,
                    signature,
                    block.scope_id
                ));

                DO_EXTRACT
            },

            TokenKind::Static => {
                // Evaluate the constant expression and add it to the symbol table

                let (name, static_data_type, mut definition) = match_unreachable!(Some(ChildrenType::Static { name, data_type, definition }) = statement.children.take(), (name, data_type, definition));

                resolve_expression_types(&mut definition, block.scope_id, None, function_parent_scope, symbol_table, source);
                evaluate_constants(&mut definition, source, block.scope_id, symbol_table);

                let literal_value = match std::mem::replace(&mut definition.item.value, TokenKind::Comma) { // Replace with a small random TokenKind to avoid cloning the LiteralValue

                    // Allow initializing statics with literal values
                    TokenKind::Value(Value::Literal { value }) => value,

                    // Allow initializing statics with initialized constants and other initialized statics
                    TokenKind::Value(Value::Symbol { name, scope_discriminant }) => {
                        let symbol = symbol_table.get_symbol(block.scope_id, name, scope_discriminant).unwrap().borrow();
                        
                        match &symbol.value {

                            SymbolValue::Constant(value) |
                            SymbolValue::Static { mutable: _, init_value: value }
                             => value.clone(),

                            _ => error::not_a_constant(&definition.item, source, "Static definition must be a literal value or a constant expression.")
                        }
                    },
                    
                    _ => error::not_a_constant(&definition.item, source, "Static definition must be a literal value or a constant expression.")
                };

                let value_type = literal_value.data_type(symbol_table);
                if !value_type.is_implicitly_castable_to(&static_data_type, Some(&literal_value)) {
                    error::type_error(&definition.item, &[&static_data_type.name()], &value_type, source, "Mismatched data type in static declaration.");
                }
                let final_value = LiteralValue::from_cast(literal_value, &value_type, &static_data_type);
           
                let res = symbol_table.define_static(name, block.scope_id, final_value);
                if res.is_err() {
                    error::compile_time_operation_error(&statement.item, source, format!("Could not define static \"{name}\".").as_str());
                }

                DO_EXTRACT
            }

            TokenKind::Const => {
                // Evaluate the constant expression and add it to the symbol table

                let (name, const_data_type, mut definition) = match_unreachable!(Some(ChildrenType::Const { name, data_type, definition }) = statement.children.take(), (name, data_type, definition));
                
                resolve_expression_types(&mut definition, block.scope_id, None, function_parent_scope, symbol_table, source);
                evaluate_constants(&mut definition, source, block.scope_id, symbol_table);

                let literal_value = match std::mem::replace(&mut definition.item.value, TokenKind::Comma) { // Replace with a small random TokenKind to avoid cloning the LiteralValue

                    // Allow initializing constants with literal values
                    TokenKind::Value(Value::Literal { value }) => value,

                    // Allow initializing constants with other initialized constants
                    TokenKind::Value(Value::Symbol { name, scope_discriminant }) => {
                        let symbol = symbol_table.get_symbol(block.scope_id, name, scope_discriminant).unwrap().borrow();
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
                
                let res = symbol_table.define_constant(name, block.scope_id, final_value);
                if res.is_err() {
                    error::compile_time_operation_error(&statement.item, source, format!("Could not define constant \"{name}\".").as_str());
                }

                DO_EXTRACT
            },

            TokenKind::TypeDef => {
                // Type was already defined
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


fn resolve_functions_types(functions: &mut [Function], symbol_table: &mut SymbolTable, source: &SourceCode) {
    for function in functions {
        let return_type = function.return_type();
        resolve_scope_types(&mut function.code, Some(return_type), function.parent_scope, symbol_table, source);
    }
}


fn evaluate_constants_functions(functions: &mut [Function], symbol_table: &mut SymbolTable, source: &SourceCode) {
    for function in functions {
        evaluate_constants_block(&mut function.code, source, symbol_table);
    }
}



/// Resolve and check the types of symbols and expressions.
fn resolve_scope_types(block: &mut ScopeBlock, outer_function_return: Option<Rc<DataType>>, function_parent_scope: ScopeID, symbol_table: &mut SymbolTable, source: &SourceCode) {
    // Perform a depth-first traversal of the scope tree to determine the types in a top-to-bottom order (relative to the source code).
    // For every node in every scope, determine the node data type and check if it matches the expected type.

    for statement in &mut block.statements {

        resolve_expression_types(statement, block.scope_id, outer_function_return.clone(), function_parent_scope, symbol_table, source);

    }
}


fn warn_unused_symbols(block: &ScopeBlock, symbol_table: &SymbolTable, source: &SourceCode) {
    for (name, symbol) in symbol_table.get_unread_symbols(block.scope_id) {
        let symbol = symbol.borrow();

        // Some symbols are used internally like the main() function and thus may not be used in the source code
        if let ("main", DataType::Function { .. }) = (name, symbol.data_type.as_ref()) {
            continue;
        }

        let token = &symbol.token;
        error::warn(token, source, format!("Symbol \"{name}\" is declared but never used.\nDeclaration occurs at {}:{}:\n\n{}\n", token.line_number(), token.column, &source[token.line_index()]).as_str());
    }
}


/// Reduce the operations down the node by evaluating constant expressions.
/// 
/// Return whether the node can be removed because it has no effect.
fn evaluate_constants(node: &mut TokenNode, source: &SourceCode, scope_id: ScopeID, symbol_table: &mut SymbolTable) -> bool {

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
                    
                _ => false
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

        TokenKind::DoWhile => {
            let (condition, body) = match_unreachable!(Some(ChildrenType::While { condition, body }) = &mut node.children, (condition, body));

            evaluate_constants_block(body, source, symbol_table);
            evaluate_constants(condition, source, scope_id, symbol_table);

            if let Some(condition_value) = condition.item.value.literal_value() {
                let bool_value = match_unreachable!(LiteralValue::Bool(v) = condition_value, v);
                if *bool_value {
                    // The condition is always true, so the body will always be executed
                    // Downgrade the do-while loop to a unconditional loop

                    error::warn(&condition.item.token, source, "Do-while loop condition is always true. This loop will be converted to a unconditional loop.");

                    node.item.value = TokenKind::Loop;
                    node.children = Some(ChildrenType::ParsedBlock(
                        match_unreachable!(Some(ChildrenType::While { body, .. }) = node.children.take(), body)
                    ));
                } else {
                    // The condition is always false, so the body will only be executed once
                    // Downgrate the do-while loop to a simple block

                    error::warn(&condition.item.token, source, "Do-while loop condition is always false. This loop will be converted to a simple block.");

                    node.item.value = TokenKind::ScopeOpen;
                    node.children = Some(ChildrenType::ParsedBlock(
                        match_unreachable!(Some(ChildrenType::While { body, .. }) = node.children.take(), body)
                    ));
                }
            }
        },

        TokenKind::While => {
            let (condition, body) = match_unreachable!(Some(ChildrenType::While { condition, body }) = &mut node.children, (condition, body));

            evaluate_constants(condition, source, scope_id, symbol_table);

            // Cannot remove the while loop if the condition has side effects
            if condition.has_side_effects {
                evaluate_constants_block(body, source, symbol_table);
                return SHOULD_NOT_BE_REMOVED;
            }

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

                    return SHOULD_NOT_BE_REMOVED;
                } 

                // Condition is always false, so the body will never be executed
                // Remove the while loop entirely
                return SHOULD_BE_REMOVED;
            }
        },

        TokenKind::As => {
            let (target_type, expr) = match_unreachable!(Some(ChildrenType::TypeCast { expr, target_type }) = &mut node.children, (target_type, expr));

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

            let (target_type, expr) = match_unreachable!(Some(ChildrenType::TypeCast { target_type, expr }) = node.children.take(), (target_type, expr));
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
                assert!(matches!(inner_block.return_type().as_ref(), DataType::Void));
                return SHOULD_BE_REMOVED;
            }

            // If the scope doesn't return anything and has no side effects, it can be removed
            if matches!(inner_block.return_type().as_ref(), DataType::Void) && !inner_block.has_side_effects {
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
fn evaluate_constants_block(block: &mut ScopeBlock, source: &SourceCode, symbol_table: &mut SymbolTable) {
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


/// Recursively resolve the type of this expression and check if its children have the correct types.
fn resolve_expression_types(expression: &mut TokenNode, scope_id: ScopeID, outer_function_return: Option<Rc<DataType>>, function_parent_scope: ScopeID, symbol_table: &mut SymbolTable, source: &SourceCode) {

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

                    resolve_expression_types(operand, scope_id, outer_function_return, function_parent_scope, symbol_table, source);

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

                    resolve_expression_types(operand, scope_id, outer_function_return, function_parent_scope, symbol_table, source);

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

                    resolve_expression_types(op1, scope_id, outer_function_return.clone(), function_parent_scope, symbol_table, source);
                    resolve_expression_types(op2, scope_id, outer_function_return, function_parent_scope, symbol_table, source);

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

                    resolve_expression_types(operand, scope_id, outer_function_return, function_parent_scope, symbol_table, source);

                    require_initialized!(operand);

                    if !operator.is_allowed_type(&operand.data_type, 0) {
                        error::type_error(&operand.item, operator.allowed_types(0), &operand.data_type, source, format!("Data type is not allowed for operator {}.", operator).as_str())
                    }

                    DataType::Bool.into()
                },

                // Unary operators whose return type is the same as the operand type
                Ops::BitwiseNot => {
                    let operand = match_unreachable!(Some(ChildrenType::Unary(operand)) = &mut expression.children, operand);

                    resolve_expression_types(operand, scope_id, outer_function_return, function_parent_scope, symbol_table, source);

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

                    resolve_expression_types(op1, scope_id, outer_function_return.clone(), function_parent_scope, symbol_table, source);
                    resolve_expression_types(op2, scope_id, outer_function_return, function_parent_scope, symbol_table, source);

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
                    resolve_expression_types(callable, scope_id, outer_function_return.clone(), function_parent_scope, symbol_table, source);

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
                        resolve_expression_types(arg, scope_id, outer_function_return.clone(), function_parent_scope, symbol_table, source);

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

                        resolve_expression_types(return_expr, scope_id, outer_function_return, function_parent_scope, symbol_table, source);

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
                    resolve_expression_types(l_node, scope_id, outer_function_return.clone(), function_parent_scope, symbol_table, source);
                    resolve_expression_types(r_node, scope_id, outer_function_return, function_parent_scope, symbol_table, source);

                    require_initialized!(r_node);
                
                    // Assert that the symbol or dereference can be assigned to (mutable or uninitialized)
                    match &mut l_node.item.value {
                        TokenKind::Value(Value::Symbol { name, scope_discriminant }) => {

                            // Unwrap is safe because symbols have already been checked to be valid
                            let mut symbol = symbol_table.get_symbol(scope_id, name, *scope_discriminant).unwrap().borrow_mut();
                            
                            if symbol.initialized {
                                if !symbol.is_mutable() {
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

                            } else if let DataType::Array { element_type: _, size: None } = l_node.data_type.as_ref() {
                                // The array size was not specified, so it is inferred from the right operand
                                symbol.data_type = r_node.data_type.clone();
                                l_node.data_type = r_node.data_type.clone();
                            }
                        },

                        TokenKind::Op(Ops::Deref{ mutable }) => {
                            // The dereference must be mutable
                            if !*mutable {
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

                    resolve_expression_types(array_node, scope_id, outer_function_return.clone(), function_parent_scope, symbol_table, source);
                    resolve_expression_types(index_node, scope_id, outer_function_return, function_parent_scope, symbol_table, source);

                    require_initialized!(array_node);
                    require_initialized!(index_node);

                    let data_type = match_or!(DataType::Array { element_type, size: _ } = array_node.data_type.as_ref(), element_type.clone(),
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
            let (target_type, expr) = match_unreachable!(Some(ChildrenType::TypeCast { target_type, expr }) = &mut expression.children, (target_type, expr));

            // Resolve the type of the expression to be cast
            resolve_expression_types(expr, scope_id, outer_function_return, function_parent_scope, symbol_table, source);

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
                let (symbol, outside_function_boundary) = symbol_table.get_symbol_warn_if_outside_function(scope_id, name, *scope_discriminant, function_parent_scope);
                
                let mut symbol = symbol.unwrap_or_else(
                    || error::symbol_undefined(&expression.item, name, source, if let Some(symbol) = symbol_table.get_unreachable_symbol(name) { let symbol = symbol.borrow(); format!("Symbol \"{name}\" is declared in a different scope at {}:{}:\n{}.", symbol.line_number(), symbol.token.column, source[symbol.token.line_index]) } else { format!("Symbol \"{name}\" is not declared in any scope.") }.as_str())
                ).borrow_mut();

                // Disallow caputuring symbols from outsize the function boundary, unless they are constants, statics, or functions
                if outside_function_boundary && !matches!(symbol.value, SymbolValue::Constant(_) | SymbolValue::Function | SymbolValue::Static { .. }) {
                    error::illegal_symbol_capture(&expression.item, source, format!("Cannot capture dynamic environment (symbol \"{}\") inside a function.\n Symbol declared at line {}:{}:\n\n{}", symbol.token.string, symbol.token.line_number(), symbol.token.column, &source[symbol.token.line_index()]).as_str());
                }

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

            resolve_scope_types(body, Some(return_type.clone()), function_parent_scope, symbol_table, source);

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

            let array_size = elements.len();
            
            let (data_type, is_literal_array, element_type) = if elements.is_empty() {
                (DataType::Array { element_type: DataType::Void.into(), size: Some(0) }, true, DataType::Void.into())
            } else {

                let mut element_type: Option<Rc<DataType>> = None;

                let mut is_literal_array = true;
                for element in elements {
                    
                    // Resolve the element type
                    resolve_expression_types(element, scope_id, outer_function_return.clone(), function_parent_scope, symbol_table, source);

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

                (DataType::Array { element_type: element_type.as_ref().unwrap().clone(), size: Some(array_size) }, is_literal_array, element_type.unwrap())
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
                resolve_scope_types(inner_block, outer_function_return, function_parent_scope, symbol_table, source);
                inner_block.return_type()
            }
        },

        TokenKind::If => {
            // Recursively resolve the types of the if-else chain
            // The return type of the chain is the return type of the conditional blocks

            let mut chain_return_type: Option<Rc<DataType>> = None;

            let (if_chain, else_block) = match_unreachable!(Some(ChildrenType::IfChain { if_chain, else_block }) = &mut expression.children, (if_chain, else_block));

            for if_block in if_chain {
                resolve_expression_types(&mut if_block.condition, scope_id, outer_function_return.clone(), function_parent_scope, symbol_table, source);
                resolve_scope_types(&mut if_block.body, outer_function_return.clone(), function_parent_scope, symbol_table, source);

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
                resolve_scope_types(else_block, outer_function_return, function_parent_scope, symbol_table, source);

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

            resolve_expression_types(condition_node, scope_id, outer_function_return.clone(), function_parent_scope, symbol_table, source);
            resolve_scope_types(body_block, outer_function_return, function_parent_scope, symbol_table, source);

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


/// Recursively calculate the side effects of the nodes and return whether the operation has global side effects (outside of the function)
fn calculate_side_effects(node: &mut TokenNode, scope_id: ScopeID, symbol_table: &SymbolTable) -> bool {
    /*
        A statement has side effects if:
        - performs I/O operations
        - performs an assignment
        - calls a function with side effects
        - contains a control flow operation
        - contains an operation with side effects

        A function has side effects if:
        - performs I/O operations
        - performs an assignment to a static variable
        - has at least one mutable reference in its parameters (conservative approach, may be improved later)
        - calls a function with global side effects
        - takes a mutable reference to a static variable (conservative)


        In a statement, side effects propagate from the children to the parent.
        For instance:
        `
            let a = 0; <-- no side effects
            let b = a + 1; <-- no side effects
            a = b; <-- local side effects
            let c = { a = 0; 2 } + 9; <-- local side effects
        `

    */

    // Leave this intentionally uninitialized so that the compiler can warn if it is not set explicitly
    let mut function_side_effects: bool;

    const HAS_LOCAL_SIDE_EFFECTS: bool = true;
    const NO_LOCAL_SIDE_EFFECTS: bool = false;

    node.has_side_effects = match &node.item.value {

        TokenKind::Op(op) => match op {
            Ops::Add |
            Ops::Sub |
            Ops::Mul |
            Ops::Div |
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
            Ops::BitwiseXor |
            Ops::ArrayIndexOpen |
            Ops::Mod 
             => {
                let (l_node, r_node) = match_unreachable!(Some(ChildrenType::Binary (l_node, r_node)) = &mut node.children, (l_node, r_node));

                function_side_effects = calculate_side_effects(l_node, scope_id, symbol_table);
                function_side_effects |= calculate_side_effects(r_node, scope_id, symbol_table);

                // Propagate the side effects of the children to the parent
                l_node.has_side_effects | r_node.has_side_effects
            },

            Ops::LogicalNot |
            Ops::BitwiseNot
             => {
                let operand = match_unreachable!(Some(ChildrenType::Unary (operand)) = &mut node.children, operand);

                function_side_effects = calculate_side_effects(operand, scope_id, symbol_table);

                // Propagate the side effects of the children to the parent
                operand.has_side_effects
            },

            Ops::Assign => {
                let (l_node, r_node) = match_unreachable!(Some(ChildrenType::Binary (l_node, r_node)) = &mut node.children, (l_node, r_node));

                function_side_effects = calculate_side_effects(l_node, scope_id, symbol_table);
                function_side_effects |= calculate_side_effects(r_node, scope_id, symbol_table);

                // Check if the left operand is a static variable or a dereference of a static variable

                let l_value_node = if let TokenKind::Op(Ops::Deref { mutable: _ }) = &l_node.item.value {
                    match_unreachable!(Some(ChildrenType::Unary (operand)) = &mut l_node.children, operand)
                } else {
                    l_node
                };

                if let TokenKind::Value(Value::Symbol { name, scope_discriminant }) = l_value_node.item.value {
                    let symbol = symbol_table.get_symbol(scope_id, name, scope_discriminant).unwrap().borrow();
                    if matches!(symbol.value, SymbolValue::Static { .. }) {
                        // An assignment to a static variable always has non-local side effects
                        function_side_effects = true;
                    }
                }

                // An assignment operation always has local side effects
                HAS_LOCAL_SIDE_EFFECTS
            },

            Ops::Ref { mutable } => {
                let operand = match_unreachable!(Some(ChildrenType::Unary (operand)) = &mut node.children, operand);

                function_side_effects = calculate_side_effects(operand, scope_id, symbol_table);

                // If the operand is a static symbol and the reference is mutable, the reference operation has global side effects (conservative approach, the ref may not be used to mutate the static symbol)
                if let TokenKind::Value(Value::Symbol { name, scope_discriminant }) = &operand.item.value {
                    let symbol = symbol_table.get_symbol(scope_id, name, *scope_discriminant).unwrap().borrow();
                    if matches!(symbol.value, SymbolValue::Static { .. }) && *mutable {
                        function_side_effects = true;
                    }
                }

                operand.has_side_effects
            }, 

            Ops::Deref { mutable: _ } => {
                let operand = match_unreachable!(Some(ChildrenType::Unary (operand)) = &mut node.children, operand);

                function_side_effects = calculate_side_effects(operand, scope_id, symbol_table);

                operand.has_side_effects
            },

            Ops::FunctionCallOpen => {
                let (callable, args) = match_unreachable!(Some(ChildrenType::Call { callable, args }) = &mut node.children, (callable, args));

                let mut has_local_side_effects = false;

                function_side_effects = calculate_side_effects(callable, scope_id, symbol_table);
                has_local_side_effects |= callable.has_side_effects;

                for arg in args {
                    function_side_effects |= calculate_side_effects(arg, scope_id, symbol_table);
                    has_local_side_effects |= arg.has_side_effects;
                }

                has_local_side_effects
            },

            Ops::Return => {
                if let Some(children) = &mut node.children {
                    let return_node = match_unreachable!(ChildrenType::Unary(x) = children, x);

                    function_side_effects = calculate_side_effects(return_node, scope_id, symbol_table);
                } else {
                    function_side_effects = false;
                }

                // Unconditional control flow always has local side effects
                HAS_LOCAL_SIDE_EFFECTS
            },

            Ops::Continue |
            Ops::Break
             => {
                // No function side effects for unconditional control flow operations
                function_side_effects = false;

                // Unconditional control flow operations always have local side effects
                HAS_LOCAL_SIDE_EFFECTS
            },
        },

        TokenKind::If => {
            let (if_chain, else_block) = match_unreachable!(Some(ChildrenType::IfChain { if_chain, else_block }) = &mut node.children, (if_chain, else_block));

            function_side_effects = false;
            for if_block in if_chain {
                function_side_effects |= calculate_side_effects(&mut if_block.condition, scope_id, symbol_table);
                function_side_effects |= calculate_side_effects_block(&mut if_block.body, symbol_table);
            }

            if let Some(else_block) = else_block {
                function_side_effects |= calculate_side_effects_block(else_block, symbol_table);
            }

            NO_LOCAL_SIDE_EFFECTS
        },

        TokenKind::While |
        TokenKind::DoWhile
         => {
            let (condition_node, body_block) = match_unreachable!(Some(ChildrenType::While { condition, body }) = &mut node.children, (condition, body));

            function_side_effects = calculate_side_effects(condition_node, scope_id, symbol_table);
            function_side_effects |= calculate_side_effects_block(body_block, symbol_table);

            NO_LOCAL_SIDE_EFFECTS
        
        },

        TokenKind::Loop => {
            let inner_block = match_unreachable!(Some(ChildrenType::ParsedBlock(inner_block)) = &mut node.children, inner_block);

            function_side_effects = calculate_side_effects_block(inner_block, symbol_table);

            NO_LOCAL_SIDE_EFFECTS
        },

        TokenKind::ArrayOpen => {
            // Has side effects if any of its elements has side effects

            let elements = match_unreachable!(Some(ChildrenType::List(elements)) = &mut node.children, elements);

            function_side_effects = false;
            let mut has_local_side_effects = false;
            for element in elements {
                function_side_effects |= calculate_side_effects(element, scope_id, symbol_table);
                has_local_side_effects |= element.has_side_effects;
            }

            has_local_side_effects
        },

        TokenKind::ScopeOpen => {
            let inner_block = match_unreachable!(Some(ChildrenType::ParsedBlock(inner_block)) = &mut node.children, inner_block);

            function_side_effects = calculate_side_effects_block(inner_block, symbol_table);

            // The local side effects internal to the block don't make it to the parent.
            NO_LOCAL_SIDE_EFFECTS
        },

        TokenKind::As => {
            let (_target_type, expr) = match_unreachable!(Some(ChildrenType::TypeCast { target_type, expr }) = &mut node.children, (target_type, expr));

            function_side_effects = calculate_side_effects(expr, scope_id, symbol_table);

            expr.has_side_effects
        },
        
        // A value has no side effects. Side effects may arise when the value is used in an operation.
        TokenKind::Value(_) => {
            function_side_effects = false;
            NO_LOCAL_SIDE_EFFECTS
        },
        
        _ => unreachable!("Unexpected node during side effects calculation: {:?}. This is a bug.", node.item.value)
    };

    function_side_effects
}


fn calculate_side_effects_block(block: &mut ScopeBlock, symbol_table: &SymbolTable) -> bool {

    let mut function_side_effects = false;

    for statement in block.statements.iter_mut() {
        function_side_effects |= calculate_side_effects(statement, block.scope_id, symbol_table);
    }

    block.has_side_effects = function_side_effects;
    function_side_effects
}


fn calculate_side_effects_function(function: &mut Function, symbol_table: &mut SymbolTable) {

    // Conservative estimate of side effects. If a mutable reference is passed in, the function is assumed to have side effects.
    let arg_types = match_unreachable!(DataType::Function { params, return_type: _ } = function.signature.as_ref(), params);
    for arg in arg_types {
        if let DataType::Ref { target: _, mutable: true } = arg.as_ref() {
            function.has_side_effects = true;
            break;
        }
    }

    function.has_side_effects |= calculate_side_effects_block(&mut function.code, symbol_table);
}


fn calculate_side_effects_functions(functions: &mut [Function], symbol_table: &mut SymbolTable) {
    for function in functions {
        calculate_side_effects_function(function, symbol_table);
    }
}


pub fn parse_functions<'a>(mut block: ScopeBlock<'a>, optimization_flags: &OptimizationFlags, symbol_table: &mut SymbolTable, source: &SourceCode, verbose: bool) -> Vec<Function<'a>> {

    let scope_id = block.scope_id;
    let mut functions = extract_functions(&mut block, false, scope_id, symbol_table, source);

    if verbose {
        println!("\n\nFunctions:\n{:#?}\n", functions);
    }

    resolve_functions_types(&mut functions, symbol_table, source);

    if verbose {
        println!("\n\nAfter symbol resolution:\n{:?}", functions);
    }
    
    warn_unused_symbols(&block, symbol_table, source);

    calculate_side_effects_functions(&mut functions, symbol_table);

    if optimization_flags.evaluate_constants {
        evaluate_constants_functions(&mut functions, symbol_table, source);
        if verbose {
            println!("\n\nAfter constant expression evaluation:\n{:?}", functions);
        }
    }
    
    functions
}

