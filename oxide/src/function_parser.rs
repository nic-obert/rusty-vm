use std::mem;
use std::rc::Rc;

use crate::cli_parser::OptimizationFlags;
use crate::{match_or, match_unreachable};
use crate::ast::{RuntimeOp, ScopeBlock, SyntaxNode, SyntaxNodeValue};
use crate::symbol_table::{FunctionInfo, ScopeID, SymbolTable, SymbolValue};
use crate::lang::data_types::{DataType, LiteralValue};
use crate::lang::data_types::dt_macros::*;
use crate::lang::error;

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

    let mut functions = Vec::new();

    // Pre-allocate the maximum capacity that can possibly be needed
    // It will be shrunk to fit at the end
    let mut old_statements: Vec<SyntaxNode> = Vec::with_capacity(block.statements.len());
    // Swap the two vectors so that block.statements is the vector statements will get appended to
    // Perform the swap now because rust doesn't like moving out values without replacing them immediately
    // Move would occur in the iterations below
    std::mem::swap(&mut old_statements, &mut block.statements);

    for mut statement in old_statements {

        let keep_statement: Option<SyntaxNode> = match statement.value {

            SyntaxNodeValue::Function { name, signature, mut body } => {
                
                // Recursively extract functions from the body
                let inner_functions = extract_functions(&mut body, true, block.scope_id, symbol_table, source);
                functions.extend(inner_functions);

                functions.push(Function::new(
                    name,
                    body,
                    signature.clone(),
                    block.scope_id
                ));

                None
            },

            SyntaxNodeValue::Static { name, data_type, mut definition } => {
                // Evaluate the constant expression and add it to the symbol table

                resolve_expression_types(&mut definition, block.scope_id, None, function_parent_scope, symbol_table, source);
                evaluate_constants(&mut definition, source, block.scope_id, symbol_table);

                let literal_value = match definition.value {

                    // Allow initializing statics with literal values
                    SyntaxNodeValue::Literal(value) => value,

                    // Allow initializing statics with initialized constants and other initialized statics
                    SyntaxNodeValue::Symbol { name, scope_discriminant } => {
                        let symbol = symbol_table.get_symbol(block.scope_id, name, scope_discriminant).unwrap().borrow();
                        
                        match &symbol.value {

                            SymbolValue::Constant(value) |
                            SymbolValue::Static { mutable: _, init_value: value }
                             => value.clone(),

                            _ => error::not_a_constant(&definition.token, source, "Static definition must be a literal value or a constant expression.")
                        }
                    },
                    
                    _ => error::not_a_constant(&definition.token, source, "Static definition must be a literal value or a constant expression.")
                };

                let value_type = literal_value.data_type(symbol_table);
                if !value_type.is_implicitly_castable_to(&data_type, Some(&literal_value)) {
                    error::type_error(&definition.token, &[&data_type.name()], &value_type, source, "Mismatched data type in static declaration.");
                }
                let final_value = LiteralValue::from_cast(&literal_value, &value_type, &data_type);
           
                let res = symbol_table.define_static(name, block.scope_id, final_value);
                if res.is_err() {
                    error::compile_time_operation_error(&statement.token, source, format!("Could not define static \"{name}\".").as_str());
                }

                None
            }

            SyntaxNodeValue::Const { name, data_type, mut definition } => {
                // Evaluate the constant expression and add it to the symbol table

                resolve_expression_types(&mut definition, block.scope_id, None, function_parent_scope, symbol_table, source);
                evaluate_constants(&mut definition, source, block.scope_id, symbol_table);

                let literal_value = match definition.value {

                    // Allow initializing constants with literal values
                    SyntaxNodeValue::Literal(value) => value,

                    // Allow initializing constants with other strictly initialized constants
                    SyntaxNodeValue::Symbol { name, scope_discriminant } => {
                        let symbol = symbol_table.get_symbol(block.scope_id, name, scope_discriminant).unwrap().borrow();
                        match &symbol.value {
                            SymbolValue::Constant(value) => value.clone(),
                            _ => error::not_a_constant(&definition.token, source, "Constant definition must be a literal value or a constant expression.")
                        }
                    },
                    
                    _ => error::not_a_constant(&definition.token, source, "Constant definition must be a literal value or a constant expression.")
                };

                let value_type = literal_value.data_type(symbol_table);
                if !value_type.is_implicitly_castable_to(&data_type, Some(&literal_value)) {
                    error::type_error(&definition.token, &[&data_type.name()], &value_type, source, "Mismatched constant type.");
                }
                let final_value = LiteralValue::from_cast(&literal_value, &value_type, &data_type);
                
                let res = symbol_table.define_constant(name, block.scope_id, final_value);
                if res.is_err() {
                    error::compile_time_operation_error(&statement.token, source, format!("Could not define constant \"{name}\".").as_str());
                }

                None
            },

            SyntaxNodeValue::TypeDef { .. } => {
                // Type was already defined in the symbol table
                None
            },

            sn => {
                if !inside_function {
                    error::syntax_error(&statement.token, source, "Cannot be a top-level statement.");
                }

                statement.value = sn;
                Some(statement)
            }

        };

        if let Some(statement) = keep_statement {
            block.statements.push(statement)
        }
    }

    functions.shrink_to_fit();
    functions
}


fn resolve_functions_types(functions: &mut [Function], symbol_table: &mut SymbolTable, source: &SourceCode) {

    for function in functions {

        let return_type = function.return_type();
        resolve_scope_types(&mut function.code, Some(return_type), function.parent_scope, symbol_table, source);

        // Const functions

        // Reduce the constant function to the minimum
        // This will be useful when the function is called statically because it will be faster to evaluate.
        evaluate_constants_block(&mut function.code, source, symbol_table);
        
        let function_symbol = symbol_table.get_function(function.name, function.parent_scope).unwrap().borrow();
        let function_info = match_unreachable!(SymbolValue::Function (function_info) = &function_symbol.value, function_info);
        if function_info.is_const && function_info.has_side_effects {
            error::not_a_constant(&function_symbol.token, source, format!("Function `{}` is declared as const but has side effects.", function.name).as_str());
        }

        // The function will be evaluated upon calling. Here we should check if the function can be evaluated statically.
        if !check_block_is_constant(&function.code, function.code.scope_id, function_info, symbol_table) {
            error::not_a_constant(&function_symbol.token, source, format!("Function `{}` is declared as const but it cannot be evaluated statically.", function.name).as_str())
        }

        // TODO: store the function's constantness in the symbol table to avoid re-evaluating it every time it's called
    }
}


const IS_CONSTANT: bool = true;
const NOT_CONSTANT: bool = false;


fn check_node_is_constant(node: &SyntaxNode, scope_id: ScopeID, function_top_scope: ScopeID, function_info: &FunctionInfo, symbol_table: &SymbolTable) -> bool {

    match &node.value {

        SyntaxNodeValue::RuntimeOp(op) => match op {
            // A runtime operation is constant if all its operands are constant

            RuntimeOp::MakeArray { elements }
                => elements.iter()
                    .map(|elem| check_node_is_constant(elem, scope_id, function_top_scope, function_info, symbol_table))
                    .fold(IS_CONSTANT, |acc, result| acc & result),

            RuntimeOp::Add { left, right } |
            RuntimeOp::Sub { left, right } |
            RuntimeOp::Mul { left, right } |
            RuntimeOp::Div { left, right } |
            RuntimeOp::Mod { left, right } |
            RuntimeOp::Assign { left, right } |
            RuntimeOp::Equal { left, right } |
            RuntimeOp::NotEqual { left, right } |
            RuntimeOp::Greater { left, right } |
            RuntimeOp::Less { left, right } |
            RuntimeOp::GreaterEqual { left, right } |
            RuntimeOp::LessEqual { left, right } |
            RuntimeOp::LogicalAnd { left, right } |
            RuntimeOp::LogicalOr { left, right } |
            RuntimeOp::BitShiftLeft { left, right } |
            RuntimeOp::BitShiftRight { left, right } |
            RuntimeOp::BitwiseOr { left, right } |
            RuntimeOp::BitwiseAnd { left, right } |
            RuntimeOp::BitwiseXor { left, right } |
            RuntimeOp::ArrayIndex { array: left, index: right }
                => check_node_is_constant(left, scope_id, function_top_scope, function_info, symbol_table)
                    && check_node_is_constant(right, scope_id, function_top_scope, function_info, symbol_table),
            
            RuntimeOp::Deref { mutable: _, expr } |
            RuntimeOp::Ref { mutable: _, expr } |
            RuntimeOp::LogicalNot(expr) |
            RuntimeOp::BitwiseNot(expr) 
                => check_node_is_constant(expr, scope_id, function_top_scope, function_info, symbol_table),
            
            RuntimeOp::Return(expr)
                => expr.as_ref().map(|expr| check_node_is_constant(expr, scope_id, function_top_scope, function_info, symbol_table))
                    .unwrap_or(NOT_CONSTANT),

            RuntimeOp::Call { callable, args } => {
                /*
                    A function call is constant if:
                    - the function expression to call is constant
                    - all its arguments are constant
                    - the called function is constant
                */

                let mut is_constant = check_node_is_constant(callable, scope_id, function_top_scope, function_info, symbol_table);

                for arg in args {
                    is_constant &= check_node_is_constant(arg, scope_id, function_top_scope, function_info, symbol_table);
                }

                is_constant &= if let SyntaxNodeValue::Symbol { name, scope_discriminant } = &callable.value {

                    let function_symbol = symbol_table.get_symbol(scope_id, name, *scope_discriminant).unwrap().borrow();
                    let function_info = match_unreachable!(SymbolValue::Function (function_info) = &function_symbol.value, function_info);

                    // TODO: check if the function is constant even if it's not marked as const
                    function_info.is_const
                } else {
                    // We don't know which function is being called
                    NOT_CONSTANT
                };

                is_constant
            },

            RuntimeOp::Break |
            RuntimeOp::Continue
                => unreachable!("SyntaxNode {:?} should never be reached here since they can only be found inside a loop and loops are classified as non-constant", node)
        },

        SyntaxNodeValue::As { target_type: _, expr }
            => check_node_is_constant(expr, scope_id, function_top_scope, function_info, symbol_table), // A type cast is constant if the expression to cast is constant

        SyntaxNodeValue::IfChain { if_blocks, else_block } => {
            /*
                An if chain is constant if:
                - all its conditions are constant
                - all its bodies are constant
            */

            let mut is_constant = true;

            for if_block in if_blocks {
                is_constant &= check_node_is_constant(&if_block.condition, scope_id, function_top_scope, function_info, symbol_table);
                is_constant &= check_block_is_constant(&if_block.body, function_top_scope, function_info, symbol_table);
            }

            if let Some(else_block) = else_block {
                is_constant &= check_block_is_constant(else_block, function_top_scope, function_info, symbol_table);
            }

            is_constant
        },
        
        SyntaxNodeValue::DoWhile { .. } |
        SyntaxNodeValue::While { .. } |
        SyntaxNodeValue::Loop { .. }
            => NOT_CONSTANT, // Loops are assumed to be non-constant even though they might be. Determining if a loop is constant is tedious.

        SyntaxNodeValue::Scope(inner_block)
            => check_block_is_constant(inner_block, function_top_scope, function_info, symbol_table),

        SyntaxNodeValue::Symbol { name, scope_discriminant } => {
            /*
                A symbol is constant if:
                - it has a known value in the symbol table
                - it's a function parameter, which will be initialized upon calling

                Now, since the function `evaluate_constants_functions` has already been called, the symbol should
                already have been substituted with its literal value, if it has one.
                Because of this, we can assume that, if a symbol is encountered at this stage, it has no known value.
                This means that here we only have to check if the symbol is a function parameter.
            */

            if scope_discriminant.0 != 0 {
                /*
                    If the scope discriminant is not 0, this symbol is not the first symbol declared with its name in the scope.
                    Since the function parameters are the first symbols to be declared in a function body, this symbol is not a function parameter.
                */
                return NOT_CONSTANT;  
            }

            /*
                However, the this symbol may be an omonym of a function parameter in an inner scope like this:
                
                ```
                fn foo(x: i32) {
                    {
                        let x = 5; // This x is not the function parameter even though it has the same name and it would have a scope discriminant of 0
                    }
                }
                ```
                Because of this, we have to check if the symbol was declared in the function's top-level scope.
            */
            if !symbol_table.is_function_top_level_symbol(scope_id, function_top_scope, name, *scope_discriminant) {
                return NOT_CONSTANT;
            }

            /*
                Finally, when we know that the symbol was declared in the function's top scope and is the first symbol with its name,
                we can check if its name is in the list of function parameters.
            */
            for param_name in function_info.param_names.iter() {
                if name == param_name {
                    return IS_CONSTANT;
                }
            }

            // The symbol name does not match any function parameter name and its value is not known
            NOT_CONSTANT
        },

        SyntaxNodeValue::Literal(_)
            => IS_CONSTANT, // Literals are always constant
        
        SyntaxNodeValue::FunctionParams(_) |
        SyntaxNodeValue::DataType(_) |
        SyntaxNodeValue::Function { .. } |
        SyntaxNodeValue::Const { .. } |
        SyntaxNodeValue::Static { .. } |
        SyntaxNodeValue::TypeDef { .. } |
        SyntaxNodeValue::Placeholder
            => unreachable!("Unexpected SyntaxNode {:?}. This node should have been removed. This is a bug.", node),
    }
}


fn check_block_is_constant(block: &ScopeBlock, function_top_scope: ScopeID, function_info: &FunctionInfo, symbol_table: &SymbolTable) -> bool {
    /*
        A function can be declared as constant only if all its operations are constant expressions.
        Its parameters are the only symbols that are allowed to be undefined, as their values will be known upon calling.
    */

    let mut is_constant = true;

    for statement in block.statements.iter() {
        is_constant &= check_node_is_constant(statement, block.scope_id, function_top_scope, function_info, symbol_table);
    }

    is_constant
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
fn evaluate_constants(node: &mut SyntaxNode, source: &SourceCode, scope_id: ScopeID, symbol_table: &mut SymbolTable) -> bool {

    macro_rules! has_known_value {
        ($($node:expr),+) => {
            ($(
                match &$node.value {
                    SyntaxNodeValue::Literal (_) => true,

                    SyntaxNodeValue::Symbol { name, scope_discriminant }
                        => symbol_table.get_symbol(scope_id, name, *scope_discriminant).unwrap().borrow().get_value().is_some(),
                        
                    _ => false
                }
            )&&+)
        };
    }

    const SHOULD_BE_REMOVED: bool = true;
    const SHOULD_NOT_BE_REMOVED: bool = false;

    match &mut node.value {
        SyntaxNodeValue::RuntimeOp(op) => match op {

            // These operators are only allowed in runtime for obvious reasons
            RuntimeOp::Break |
            RuntimeOp::Continue
             => { /* Don't remove them */ },

            RuntimeOp::ArrayIndex { array, index } => {

                evaluate_constants(array, source, scope_id, symbol_table);
                evaluate_constants(index, source, scope_id, symbol_table);

                // TODO: implement compile-time array indexing for literal arrays and initialized immutable arrays
            },

            RuntimeOp::Assign { left: l_node, right: r_node } => {

                evaluate_constants(l_node, source, scope_id, symbol_table);
                evaluate_constants(r_node, source, scope_id, symbol_table);

                // If the left operand is a plain symbol and the right operand is known, perform the assignment statically
                if let (SyntaxNodeValue::Symbol { name, scope_discriminant }, true) = (&l_node.value, has_known_value!(r_node)) {
                    
                    let mut l_symbol = symbol_table.get_symbol(scope_id, name, *scope_discriminant).unwrap().borrow_mut();

                    // Only allow to statically initialize immutable symbols
                    if matches!(l_symbol.value, SymbolValue::Immutable(_)) {


                        let r_value = match r_node.extract_value() {
                            SyntaxNodeValue::Literal (value) => Some(value),
            
                            SyntaxNodeValue::Symbol { name, scope_discriminant } => {
                                let symbol = symbol_table.get_symbol(scope_id, name, scope_discriminant).unwrap();
                                symbol.borrow().get_value()
                            }
                            
                            _ => unreachable!()
                        }.unwrap();

                        l_symbol.initialize_immutable(r_value);

                        // The assignment has just been performed statically, so the assignment operation can be removed (assignment operation has no side effect and is not an expression)
                        return SHOULD_BE_REMOVED;
                    }
                }
            },

            RuntimeOp::Add { left, right } |
            RuntimeOp::Sub { left, right } |
            RuntimeOp::Mul { left, right } |
            RuntimeOp::Div { left, right } |
            RuntimeOp::Mod { left, right } |
            RuntimeOp::BitShiftLeft { left, right } |
            RuntimeOp::BitShiftRight { left, right } |
            RuntimeOp::BitwiseOr { left, right } |
            RuntimeOp::BitwiseAnd { left, right } |
            RuntimeOp::BitwiseXor { left, right } |
            RuntimeOp::Equal { left, right } |
            RuntimeOp::NotEqual { left, right } |
            RuntimeOp::Greater { left, right } |
            RuntimeOp::Less { left, right } |
            RuntimeOp::GreaterEqual { left, right } |
            RuntimeOp::LessEqual { left, right } |
            RuntimeOp::LogicalAnd { left, right } |
            RuntimeOp::LogicalOr { left, right }
            => {

                evaluate_constants(left, source, scope_id, symbol_table);
                evaluate_constants(right, source, scope_id, symbol_table);

                if !has_known_value!(left, right) {
                    return SHOULD_NOT_BE_REMOVED;
                }
                
                let res = match op.execute(scope_id, symbol_table) {
                    Ok(res) => res,
                    Err(err) => error::compile_time_operation_error(&node.token, source, err)
                };

                node.value = SyntaxNodeValue::Literal (res);
            },

            RuntimeOp::BitwiseNot(operand) |
            RuntimeOp::LogicalNot(operand)
            => {

                evaluate_constants(operand, source, scope_id, symbol_table);

                if !operand.has_literal_value() {
                    return SHOULD_NOT_BE_REMOVED;
                }

                let res = match op.execute(scope_id, symbol_table) {
                    Ok(res) => res,
                    Err(err) => error::compile_time_operation_error(&node.token, source, err)
                };

                node.value = SyntaxNodeValue::Literal (res);
            },

            RuntimeOp::Ref { mutable: _, expr } |
            RuntimeOp::Deref { mutable: _, expr }
            => {
                evaluate_constants(expr, source, scope_id, symbol_table);
            },

            RuntimeOp::Return (return_value) => if let Some(expr) = return_value {
                evaluate_constants(expr, source, scope_id, symbol_table);
            },
            
            RuntimeOp::Call { callable, args } => {

                // TODO: evaluate const functions

                evaluate_constants(callable, source, scope_id, symbol_table);

                for arg in args {
                    evaluate_constants(arg, source, scope_id, symbol_table);
                }
            },

            RuntimeOp::MakeArray { elements } => {
                for element in elements {
                    evaluate_constants(element, source, scope_id, symbol_table);
                }
            },
        },

        SyntaxNodeValue::DoWhile { body, condition } => {

            evaluate_constants_block(body, source, symbol_table);
            evaluate_constants(condition, source, scope_id, symbol_table);

            if let Some(condition_value) = condition.known_literal_value(scope_id, symbol_table) {
                let bool_value = match_unreachable!(LiteralValue::Bool(v) = condition_value.as_ref(), v);
                if *bool_value {
                    // The condition is always true, so the body will always be executed
                    // Downgrade the do-while loop to a unconditional loop

                    error::warn(&condition.token, source, "Do-while loop condition is always true. This loop will be converted to a unconditional loop.");

                    node.value = SyntaxNodeValue::Loop { body: mem::take(body)};
                } else {
                    // The condition is always false, so the body will only be executed once
                    // Downgrate the do-while loop to a simple scope block

                    error::warn(&condition.token, source, "Do-while loop condition is always false. This loop will be converted to a simple block.");

                    node.value = SyntaxNodeValue::Scope(mem::take(body));
                }
            }
        },

        SyntaxNodeValue::While { condition, body } => {

            evaluate_constants(condition, source, scope_id, symbol_table);

            // Cannot remove the while loop if the condition has side effects
            if condition.has_side_effects {
                evaluate_constants_block(body, source, symbol_table);
                return SHOULD_NOT_BE_REMOVED;
            }

            if let Some(condition_value) = condition.known_literal_value(scope_id, symbol_table) {
                let bool_value = match_unreachable!(LiteralValue::Bool(v) = condition_value.as_ref(), v);
                if *bool_value {
                    // The condition is always true, so the body will always be executed
                    // Downgrade the while loop to a unconditional loop

                    error::warn(&condition.token, source, "While loop condition is always true. This loop will be converted to an unconditional loop.");

                    evaluate_constants_block(body, source, symbol_table);

                    node.value = SyntaxNodeValue::Loop { body: mem::take(body) };

                    return SHOULD_NOT_BE_REMOVED;
                } 

                // Condition is always false, so the body will never be executed
                // Remove the while loop entirely
                return SHOULD_BE_REMOVED;
            }
        },

        SyntaxNodeValue::As { target_type, expr } => {

            evaluate_constants(expr, source, scope_id, symbol_table);

            if expr.data_type == *target_type {
                // No need to perform the cast
                // Return directly the expression

                node.data_type = expr.data_type.clone();
                node.has_side_effects = expr.has_side_effects;
                node.value = expr.extract_value();

                return SHOULD_NOT_BE_REMOVED;
            }
            // The cast is to be performed

            // Cannot cast at compile-time if the expression value is unknown (not a literal)
            if !expr.has_literal_value() {
                return SHOULD_NOT_BE_REMOVED;
            }

            node.data_type = expr.data_type.clone();

            let value = expr.assume_literal();

            let new_value = LiteralValue::from_cast(&value, &expr.data_type, target_type);

            node.value = SyntaxNodeValue::Literal (new_value);
        },

        SyntaxNodeValue::Function { name: _, signature: _, body } => {
            evaluate_constants_block(body, source, symbol_table);
        },

        SyntaxNodeValue::IfChain { if_blocks, else_block } => {

            for if_block in if_blocks {
                evaluate_constants(&mut if_block.condition, source, scope_id, symbol_table);
                evaluate_constants_block(&mut if_block.body, source, symbol_table);
            }

            if let Some(else_block) = else_block {
                evaluate_constants_block(else_block, source, symbol_table);
            }
        },     

        SyntaxNodeValue::Scope(inner_block) => {

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

        SyntaxNodeValue::Loop { body } => {
            evaluate_constants_block(body, source, symbol_table);

            if body.statements.is_empty() {
                // Empty loops can be removed
                return SHOULD_BE_REMOVED;
            }
        },

        SyntaxNodeValue::Symbol { .. } |
        SyntaxNodeValue::Literal(_) => {
            // Literal values and symbols are already reduced to the minimum.
            // If the value of a symbol is known, it's up to the operator to get it.
        },

        SyntaxNodeValue::DataType(_) |
        SyntaxNodeValue::FunctionParams(_) |
        SyntaxNodeValue::Const { .. } |
        SyntaxNodeValue::Static { .. } |
        SyntaxNodeValue::TypeDef { .. } |
        SyntaxNodeValue::Placeholder 
            => unreachable!("{:?} should have been removed from the tree.", node)
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
fn resolve_expression_types(expression: &mut SyntaxNode, scope_id: ScopeID, outer_function_return: Option<Rc<DataType>>, function_parent_scope: ScopeID, symbol_table: &mut SymbolTable, source: &SourceCode) {

    /// Assert that, if the node is a symbol, it is initialized.
    /// Not all operators require their operands to be initialized (l_value of assignment, ref)
    macro_rules! require_initialized {
        ($x:expr) => {
            if let SyntaxNodeValue::Symbol { name, scope_discriminant } = $x.value {
                let symbol = symbol_table.get_symbol(scope_id, name, scope_discriminant).unwrap().borrow();
                if !symbol.initialized {
                    error::use_of_uninitialized_value(&$x.token, &$x.data_type, source, format!("Cannot use uninitialized value \"{name}\".\nType of \"{name}\": {}.\n{name} declared at {}:{}:\n{}", symbol.data_type, symbol.line_number(), symbol.token.column, source[symbol.token.line_index]).as_str());
                }
            }
        };
    }

    expression.data_type = match &mut expression.value {

        SyntaxNodeValue::RuntimeOp(operator) => {
            // Resolve and check the types of the operands first
            // Based on the operand values, determine the type of the operator
            
            macro_rules! resolve_arithmetic_binary {
                ( $left:ident, $right:ident, $is_allowed:pat, $allowed_types:expr, $op_name:literal ) => {{
                    
                    resolve_expression_types($left, scope_id, outer_function_return.clone(), function_parent_scope, symbol_table, source);
                    resolve_expression_types($right, scope_id, outer_function_return, function_parent_scope, symbol_table, source);

                    require_initialized!($left);
                    require_initialized!($right);

                    if !matches!($left.data_type.as_ref(), $is_allowed){
                        error::type_error(&$left.token, $allowed_types, &$left.data_type, source, format!("Data type is not allowed for operator {}.", $op_name).as_str())
                    }
                    if !matches!($right.data_type.as_ref(), $is_allowed){
                        error::type_error(&$right.token, $allowed_types, &$right.data_type, source, format!("Data type is not allowed for operator {}.", $op_name).as_str())
                    }

                    // Check if the operands have the same type
                    if $left.data_type != $right.data_type {
                        // Here ot.clone() is acceptable because the program will exit after this error
                        error::type_error(&$right.token, &[&$left.data_type.name()], &$right.data_type, source, format!("Operator {:?} has operands of different types {:?} and {:?}.", $op_name, $left.data_type, $right.data_type).as_str());
                    }

                    $left.data_type.clone()
                }}
            }

            macro_rules! resolve_boolean_binary {
                ( $left:ident, $right:ident, $is_allowed:pat, $allowed_types:expr, $op_name:literal ) => {{
                    
                    resolve_expression_types($left, scope_id, outer_function_return.clone(), function_parent_scope, symbol_table, source);
                    resolve_expression_types($right, scope_id, outer_function_return, function_parent_scope, symbol_table, source);

                    require_initialized!($left);
                    require_initialized!($right);

                    if !matches!($left.data_type.as_ref(), $is_allowed) {
                        error::type_error(&$left.token, $allowed_types, &$left.data_type, source, format!("Data type is not allowed for operator {}.", $op_name).as_str())
                    }
                    if !matches!($right.data_type.as_ref(), $is_allowed){
                        error::type_error(&$right.token, $allowed_types, &$right.data_type, source, format!("Data type is not allowed for operator {}.", $op_name).as_str())
                    }

                    // Check if the operands have the same type
                    if $left.data_type != $right.data_type {
                        // Here ot.clone() is acceptable because the program will exit after this error
                        error::type_error(&$right.token, &[&$left.data_type.name()], &$right.data_type, source, format!("Operator {:?} has operands of different types {:?} and {:?}.", $op_name, $left.data_type, $right.data_type).as_str());
                    }

                    DataType::Bool.into()
                }}
            }

            macro_rules! resolve_generic_unary {
                ( $operand:ident, $is_allowed:pat, $allowed_types:expr, $op_name:literal ) => {{
                    
                    resolve_expression_types($operand, scope_id, outer_function_return.clone(), function_parent_scope, symbol_table, source);

                    require_initialized!($operand);

                    if !matches!($operand.data_type.as_ref(), $is_allowed){
                        error::type_error(&$operand.token, $allowed_types, &$operand.data_type, source, format!("Data type is not allowed for operator {}.", $op_name).as_str())
                    }

                    $operand.data_type.clone()
                }}
            }

            match operator {

                RuntimeOp::Break |
                RuntimeOp::Continue
                 => DataType::Void.into(),

                RuntimeOp::Deref { mutable, expr  } => {

                    resolve_expression_types(expr, scope_id, outer_function_return, function_parent_scope, symbol_table, source);

                    if let DataType::Ref { mutable: operand_mutable, target } = expr.data_type.as_ref() {
                        *mutable = *operand_mutable;
                        target.clone()
                    } else {
                        error::type_error(&expr.token, &[&DataType::Ref { target: DataType::Unspecified.into(), mutable: false }.name()], &expr.data_type, source, "Can only dereference a reference")
                    }
                },

                RuntimeOp::Ref { mutable, expr } => {

                    resolve_expression_types(expr, scope_id, outer_function_return, function_parent_scope, symbol_table, source);

                    if let SyntaxNodeValue::Symbol { name, scope_discriminant } = expr.value {
                        let symbol = symbol_table.get_symbol(scope_id, name, scope_discriminant).unwrap().borrow();
                        // Mutable borrows of immutable symbols are not allowed
                        if !symbol.is_mutable() && *mutable {
                            error::illegal_mutable_borrow(&expr.token, source, format!("Cannot borrow \"{name}\" as mutable because it was declared as immutable.\nType of \"{name}\": {}.\n{name} declared at {}:{}:\n{}", symbol.data_type, symbol.line_number(), symbol.token.column, source[symbol.token.line_index]).as_str())
                        }
                    }

                    DataType::Ref { target: expr.data_type.clone(), mutable: *mutable }.into()
                },

                // Binary operators whose operands must be of the same type
                RuntimeOp::Equal { left, right }
                    => resolve_boolean_binary!(left, right, _, &["any"], "Equal"),
                RuntimeOp::NotEqual { left, right }
                    => resolve_boolean_binary!(left, right, _, &["any"], "NotEqual"),
                RuntimeOp::Greater { left, right }
                    => resolve_boolean_binary!(left, right, numeric_pattern!(), &["numeric"], "Greater"),
                RuntimeOp::Less { left, right }
                    => resolve_boolean_binary!(left, right, numeric_pattern!(), &["numeric"], "Less"),
                RuntimeOp::GreaterEqual { left, right }
                    => resolve_boolean_binary!(left, right, numeric_pattern!(), &["numeric"], "GreaterEqual"),
                RuntimeOp::LessEqual { left, right }
                    => resolve_boolean_binary!(left, right, numeric_pattern!(), &["numeric"], "LessEqual"),
                RuntimeOp::LogicalAnd { left, right }
                    => resolve_boolean_binary!(left, right, DataType::Bool, &["bool"], "LogicalAnd"),
                RuntimeOp::LogicalOr { left, right } 
                    => resolve_boolean_binary!(left, right, DataType::Bool, &["bool"], "LogicalOr"),

                // Unary operators that return a boolean
                RuntimeOp::LogicalNot(operand)
                    => resolve_generic_unary!(operand, DataType::Bool, &["bool"], "LogicalNot"),

                // Unary operators whose return type is the same as the operand type
                RuntimeOp::BitwiseNot(operand)
                    => resolve_generic_unary!(operand, integer_pattern!(), &["integer"], "BitwiseNot"),

                // Binary operators whose return type is the same as the operand type
                RuntimeOp::Add { left, right }
                    => resolve_arithmetic_binary!(left, right, numeric_pattern!(), &["numeric"], "Add"),
                RuntimeOp::Sub { left, right } 
                    => resolve_arithmetic_binary!(left, right, numeric_pattern!(), &["numeric"], "Sub"),
                RuntimeOp::Mul { left, right } 
                    => resolve_arithmetic_binary!(left, right, numeric_pattern!(), &["numeric"], "Mul"),
                RuntimeOp::Div { left, right } 
                    => resolve_arithmetic_binary!(left, right, numeric_pattern!(), &["numeric"], "Div"),
                RuntimeOp::Mod { left, right } 
                    => resolve_arithmetic_binary!(left, right, numeric_pattern!(), &["integer"], "Mod"),
                RuntimeOp::BitShiftLeft { left, right }
                    => resolve_arithmetic_binary!(left, right, integer_pattern!(), &["integer"], "BitShiftLeft"),
                RuntimeOp::BitShiftRight { left, right } 
                    => resolve_arithmetic_binary!(left, right, integer_pattern!(), &["integer"], "BitShiftRight"),
                RuntimeOp::BitwiseOr { left, right } 
                    => resolve_arithmetic_binary!(left, right, integer_pattern!(), &["integer"], "BitwiseOr"),
                RuntimeOp::BitwiseAnd { left, right } 
                    => resolve_arithmetic_binary!(left, right, integer_pattern!(), &["integer"], "BitwiseAnd"),
                RuntimeOp::BitwiseXor { left, right } 
                    => resolve_arithmetic_binary!(left, right, integer_pattern!(), &["integer"], "BitwiseXor"),

                RuntimeOp::Call { callable, args } => {
                    
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

                        _ => error::type_error(&callable.token, &[&DataType::Function { params: Vec::new(), return_type: DataType::Void.into() }.name()], &callable.data_type, source, "Expected a function name or a function pointer.")
                    };

                    // Check if the number of arguments matches the number of parameters
                    // Check this before resolving the types of the arguments to avoid unnecessary work
                    if args.len() != param_types.len() {
                        error::mismatched_call_arguments(&expression.token, param_types.len(), args.len(), source, "Invalid number of arguments for function call.");
                    }                    

                    // Resolve the types of the arguments and check if they match the function parameters
                    for (arg, expected_type) in args.iter_mut().zip(param_types) {
                        resolve_expression_types(arg, scope_id, outer_function_return.clone(), function_parent_scope, symbol_table, source);

                        if arg.data_type != *expected_type {
                            error::type_error(&arg.token, &[&expected_type.name()], &arg.data_type, source, "Argument type does not match function signature.");
                        }
                    }

                    // The return type of the function call is the return type of the function
                    return_type
                },

                RuntimeOp::Return(operand) => {

                    // A return statement is only allowed inside a function
                    let return_type = outer_function_return.as_ref().unwrap_or_else(
                        || error::syntax_error(&expression.token, source, "Return statement is not allowed outside a function.")
                    ).clone();

                    // Resolve the type of the return value, if any
                    if let Some(return_expr) = operand {

                        resolve_expression_types(return_expr, scope_id, outer_function_return, function_parent_scope, symbol_table, source);

                        require_initialized!(return_expr);
                        
                        // Check if the return type matches the outer function return type
                        if return_expr.data_type != return_type {
                            error::type_error(&return_expr.token, &[&return_type.name()], &return_expr.data_type, source, "The returned expression type does not match function signature.");
                        }
                    } else if !matches!(*return_type, DataType::Void) {
                        // If the function doesn't return void, return statements must have a return value
                        error::type_error(&expression.token, &[&return_type.name()], &DataType::Void, source, "Missing return value for function that does not return void.");
                    }

                    // A return statement evaluates to void
                    DataType::Void.into()
                },

                RuntimeOp::Assign { left: l_node, right: r_node } => {
                    
                    // Only allow assignment to a symbol or a dereference
                    if !matches!(l_node.value, SyntaxNodeValue::Symbol { .. } | SyntaxNodeValue::RuntimeOp(RuntimeOp::Deref { .. })) {
                        error::type_error(&l_node.token, &["symbol<any>", "deref<any>"], &l_node.data_type, source, "Invalid left operand for assignment operator.");
                    }

                    // Resolve the types of the operands
                    resolve_expression_types(l_node, scope_id, outer_function_return.clone(), function_parent_scope, symbol_table, source);
                    resolve_expression_types(r_node, scope_id, outer_function_return, function_parent_scope, symbol_table, source);

                    require_initialized!(r_node);
                
                    // Assert that the symbol or dereference can be assigned to (mutable or uninitialized)
                    match &mut l_node.value {
                        SyntaxNodeValue::Symbol { name, scope_discriminant } => {

                            // Unwrap is safe because symbols have already been checked to be valid
                            let mut symbol = symbol_table.get_symbol(scope_id, name, *scope_discriminant).unwrap().borrow_mut();
                            
                            if symbol.initialized {
                                if !symbol.is_mutable() {
                                    // Symbol is immutable and already initialized, so cannot assign to it again
                                    error::immutable_change(&l_node.token, &l_node.data_type, source, "Cannot assign to an immutable symbol.");
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

                        SyntaxNodeValue::RuntimeOp(RuntimeOp::Deref { mutable, .. }) => {
                            // The dereference must be mutable
                            if !*mutable {
                                error::immutable_change(&l_node.token, &l_node.data_type, source, "Cannot assign to an immutable dereference.");
                            }
                        },

                        _ => unreachable!("Invalid syntax node during expression type resolution: {:?}. This is a bug.", l_node)
                    }

                    // Check if the symbol type and the expression type are compatible (the same or implicitly castable)
                    let r_value = r_node.known_literal_value(scope_id, symbol_table);
                    if !r_node.data_type.is_implicitly_castable_to(&l_node.data_type, r_value.as_deref()) {
                        error::type_error(&r_node.token, &[&l_node.data_type.name()], &r_node.data_type, source, "Mismatched right operand type for assignment operator.");
                    }
                    
                    // An assignment is not an expression, so it does not have a type
                    DataType::Void.into()
                },

                RuntimeOp::ArrayIndex { array, index } => {
                    // The data type of an array subscription operation is the type of the array elements

                    resolve_expression_types(array, scope_id, outer_function_return.clone(), function_parent_scope, symbol_table, source);
                    resolve_expression_types(index, scope_id, outer_function_return, function_parent_scope, symbol_table, source);

                    require_initialized!(array);
                    require_initialized!(index);

                    let data_type = match_or!(DataType::Array { element_type, size: _ } = array.data_type.as_ref(), element_type.clone(),
                        error::type_error(&array.token, &[&DataType::Array { element_type: DataType::Unspecified.into(), size: None }.name()], &array.data_type, source, "Can only index arrays.")
                    );

                    // Assert that the array index is an unsigned integer
                    if !matches!(index.data_type.as_ref(), integer_pattern!()) {
                        error::type_error(&index.token, &[&DataType::Usize.name()], &index.data_type, source, "Array index must strictly be an unsigned integer.");
                    }
                    
                    data_type
                },

                RuntimeOp::MakeArray { elements } => {
                    // Recursively resolve the types of the array elements.
                    // The array element type is the type of the first element.
                    // Check if the array elements have the same type.
                    // The array element type is void if the array is empty. A void array can be used as a generic array by assignment operators.
        
                    let array_size = elements.len();
                    
                    let (data_type, is_literal_array, element_type) = if elements.is_empty() {
                        (DataType::Array { element_type: DataType::Void.into(), size: Some(0) }, true, DataType::Void.into())
                    } else {
        
                        let mut element_type: Option<Rc<DataType>> = None;
        
                        let mut is_literal_array = true;
                        for element in elements.iter_mut() {
                            
                            // Resolve the element type
                            resolve_expression_types(element, scope_id, outer_function_return.clone(), function_parent_scope, symbol_table, source);
        
                            require_initialized!(element);
        
                            let expr_type = element.data_type.clone();
        
                            if let Some(expected_element_type) = &element_type {
                                if *expected_element_type != expr_type {
                                    error::type_error(&element.token, &[&expected_element_type.name()], &expr_type, source, "Array elements have different types.");
                                }
                            } else {
                                // The first element of the array determines the array type
                                element_type = Some(expr_type);
                            }
        
                            // Check if all the array elements are literals
                            is_literal_array &= element.has_literal_value();
                        }
        
                        (DataType::Array { element_type: element_type.as_ref().unwrap().clone(), size: Some(array_size) }, is_literal_array, element_type.unwrap())
                    };
        
                    if is_literal_array {
                        // If all the array elements are literals, the whole array is a literal array
                        // Change this token to a literal array
                        let mut literal_items = Vec::with_capacity(elements.len());
                        for element in mem::take(elements) {
                            literal_items.push(element.assume_literal());
                        }
                        expression.value = SyntaxNodeValue::Literal (LiteralValue::Array { element_type, items: literal_items }.into() );
                    }
        
                    data_type.into()
                },
            }
        },

        SyntaxNodeValue::As { target_type, expr } => {

            // Resolve the type of the expression to be cast
            resolve_expression_types(expr, scope_id, outer_function_return, function_parent_scope, symbol_table, source);

            require_initialized!(expr);

            if expr.data_type == *target_type {
                
                error::warn(&expression.token, source, "Redundant type cast. Expression is already of the specified type.")

            } else {
                    // Check if the expression type can be cast to the specified type
                if !expr.data_type.is_castable_to(target_type) {
                    error::type_error(&expr.token, &[&target_type.name()], &expr.data_type, source, format!("Type {:?} cannot be cast to {:?}.", expr.data_type, target_type).as_str());
                }

                // Evaluates to the data type of the type cast
            }

            target_type.clone()
        },


        SyntaxNodeValue::Literal (value) => value.data_type(symbol_table).into(),

        SyntaxNodeValue::Symbol { name, scope_discriminant } => {

            let (symbol, outside_function_boundary) = symbol_table.get_symbol_warn_if_outside_function(scope_id, name, *scope_discriminant, function_parent_scope);
            
            let mut symbol = symbol.unwrap_or_else(
                || error::symbol_undefined(&expression.token, name, source, if let Some(symbol) = symbol_table.get_unreachable_symbol(name) { let symbol = symbol.borrow(); format!("Symbol \"{name}\" is declared in a different scope at {}:{}:\n{}.", symbol.line_number(), symbol.token.column, source[symbol.token.line_index]) } else { format!("Symbol \"{name}\" is not declared in any scope.") }.as_str())
            ).borrow_mut();

            // Disallow caputuring symbols from outsize the function boundary, unless they are constants, statics, or functions
            if outside_function_boundary && !matches!(symbol.value, SymbolValue::Constant(_) | SymbolValue::Function { .. } | SymbolValue::Static { .. }) {
                error::illegal_symbol_capture(&expression.token, source, format!("Cannot capture dynamic environment (symbol \"{}\") inside a function.\n Symbol declared at line {}:{}:\n\n{}", symbol.token.string, symbol.token.line_number(), symbol.token.column, &source[symbol.token.line_index()]).as_str());
            }

            // The symbol has beed used in an expression, so it has been read from.
            // If the symbol is being assigned to instead, the Ops::Assign branch will set this flag to false later.
            // This is not an ideal design choice, but it works.
            symbol.read_from = true;

            symbol.data_type.clone()
        }

        SyntaxNodeValue::Function { name: _, signature, body } => {
            // Resolve the types inside the function body
            
            let return_type = match_unreachable!(DataType::Function { return_type, .. } = signature.as_ref(), return_type);

            resolve_scope_types(body, Some(return_type.clone()), function_parent_scope, symbol_table, source);

            // Check return type
            let return_value = body.return_value_literal(symbol_table);
            if !body.return_type().is_implicitly_castable_to(return_type.as_ref(), return_value.as_deref()) {
                error::type_error(&expression.token, &[&return_type.name()], &body.return_type(), source, "Mismatched return type in function declaration.");
            }

            // Function declaration does not have any type
            DataType::Void.into()
        },

        SyntaxNodeValue::Scope(inner_block) => {
            // Recursively resolve the types of the children statements
            // Determine the type of the scope based on the type of the last statement
            // If the scope is empty, it evaluates to void

            if inner_block.statements.is_empty() {
                DataType::Void.into()
            } else {
                resolve_scope_types(inner_block, outer_function_return, function_parent_scope, symbol_table, source);
                inner_block.return_type()
            }
        },

        SyntaxNodeValue::IfChain { if_blocks, else_block } => {
            // Recursively resolve the types of the if-else chain
            // The return type of the chain is the return type of the conditional blocks

            let mut chain_return_type: Option<Rc<DataType>> = None;

            for if_block in if_blocks {
                resolve_expression_types(&mut if_block.condition, scope_id, outer_function_return.clone(), function_parent_scope, symbol_table, source);
                resolve_scope_types(&mut if_block.body, outer_function_return.clone(), function_parent_scope, symbol_table, source);

                require_initialized!(if_block.condition);

                // Check if the return types match
                if let Some(return_type) = &chain_return_type {
                    if if_block.body.return_type() != *return_type {
                        // If the body is not empty, use its last statement as the culprit of the type mismatch. Otherwise, use the if condition.
                        let culprit_token = if let Some(last_statement) = if_block.body.statements.last() {
                            &last_statement.token
                        } else {
                            &expression.token
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
                        &last_statement.token
                    } else {
                        &expression.token
                    };
                    error::type_error(culprit_token, &[&chain_return_type.unwrap().name()], &else_block.return_type(), source, "Mismatched return type in if-else chain.");
                }
            }

            // Unwrap is safe because the if-else chain is guaranteed to have at least one if block, which sets the chain_return_type
            chain_return_type.unwrap()
        },

        SyntaxNodeValue::DoWhile { body, condition } |
        SyntaxNodeValue::While { condition, body } => {
            
            resolve_expression_types(condition, scope_id, outer_function_return.clone(), function_parent_scope, symbol_table, source);
            resolve_scope_types(body, outer_function_return, function_parent_scope, symbol_table, source);

            require_initialized!(condition);

            // Assert that the condition is a boolean
            if !matches!(*condition.data_type, DataType::Bool) {
                error::type_error(&condition.token, &[&DataType::Bool.name()], &condition.data_type, source, "While loop condition must be a boolean.");
            }

            // A while loop body should not return anything
            if !matches!(body.return_type().as_ref(), DataType::Void) {
                error::type_error(&body.statements.last().unwrap().token, &["void"], &body.return_type(), source, "Loop body should not return anything.");
            }

            DataType::Void.into()
        },

        SyntaxNodeValue::Loop { body } => {

            // Recursively resolve the types of the loop body
            resolve_scope_types(body, outer_function_return, function_parent_scope, symbol_table, source);

            if !matches!(body.return_type().as_ref(), DataType::Void) {
                error::type_error(&body.statements.last().unwrap().token, &["void"], &body.return_type(), source, "Loop body should not return anything.");
            }            

            // A loop evaluates to void
            DataType::Void.into()
        },

        SyntaxNodeValue::FunctionParams(_) |
        SyntaxNodeValue::DataType(_) |
        SyntaxNodeValue::Const { .. } |
        SyntaxNodeValue::Static { .. } |
        SyntaxNodeValue::TypeDef { .. } |
        SyntaxNodeValue::Placeholder 
            => unreachable!("Unexpected syntax node during expression and symbol type resolution: {:?}. This is a bug.", expression)
    };
}


/// Recursively calculate the side effects of the nodes and return whether the operation has global side effects (outside of the function)
fn calculate_side_effects(node: &mut SyntaxNode, scope_id: ScopeID, symbol_table: &SymbolTable) -> bool {
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

    node.has_side_effects = match &mut node.value {

        SyntaxNodeValue::RuntimeOp(op) => match op {
            RuntimeOp::Add { left, right } |
            RuntimeOp::Sub { left, right } |
            RuntimeOp::Mul { left, right } |
            RuntimeOp::Div { left, right } |
            RuntimeOp::Equal { left, right } |
            RuntimeOp::NotEqual { left, right } |
            RuntimeOp::Greater { left, right } |
            RuntimeOp::Less { left, right } |
            RuntimeOp::GreaterEqual { left, right } |
            RuntimeOp::LessEqual { left, right } |
            RuntimeOp::LogicalAnd { left, right } |
            RuntimeOp::LogicalOr { left, right } |
            RuntimeOp::BitShiftLeft { left, right } |
            RuntimeOp::BitShiftRight { left, right } |
            RuntimeOp::BitwiseOr { left, right } |
            RuntimeOp::BitwiseAnd { left, right } |
            RuntimeOp::BitwiseXor { left, right } |
            RuntimeOp::Mod { left, right } |
            RuntimeOp::ArrayIndex { array: left, index: right }
            => {
                
                function_side_effects = calculate_side_effects(left, scope_id, symbol_table);
                function_side_effects |= calculate_side_effects(right, scope_id, symbol_table);
                
                // Propagate the side effects of the children to the parent
                left.has_side_effects | right.has_side_effects
            },

            RuntimeOp::LogicalNot (operand) |
            RuntimeOp::BitwiseNot (operand)
             => {

                function_side_effects = calculate_side_effects(operand, scope_id, symbol_table);

                // Propagate the side effects of the children to the parent
                operand.has_side_effects
            },

            RuntimeOp::Assign { left: l_node, right: r_node } => {

                function_side_effects = calculate_side_effects(l_node, scope_id, symbol_table);
                function_side_effects |= calculate_side_effects(r_node, scope_id, symbol_table);

                // Check if the left operand is a static variable or a dereference of a static variable

                let l_value_node = if let SyntaxNodeValue::RuntimeOp(RuntimeOp::Deref { mutable: _, expr }) = &l_node.value {
                    expr
                } else {
                    l_node
                };

                if let SyntaxNodeValue::Symbol { name, scope_discriminant } = l_value_node.value {
                    let symbol = symbol_table.get_symbol(scope_id, name, scope_discriminant).unwrap().borrow();
                    if matches!(symbol.value, SymbolValue::Static { .. }) {
                        // An assignment to a static variable always has non-local side effects
                        function_side_effects = true;
                    }
                }

                // An assignment operation always has local side effects
                HAS_LOCAL_SIDE_EFFECTS
            },

            RuntimeOp::Ref { mutable, expr } => {

                function_side_effects = calculate_side_effects(expr, scope_id, symbol_table);

                // If the operand is a static symbol and the reference is mutable, the reference operation has global side effects (conservative approach, the ref may not be used to mutate the static symbol)
                if let SyntaxNodeValue::Symbol { name, scope_discriminant } = &expr.value {
                    let symbol = symbol_table.get_symbol(scope_id, name, *scope_discriminant).unwrap().borrow();
                    if matches!(symbol.value, SymbolValue::Static { .. }) && *mutable {
                        function_side_effects = true;
                    }
                }

                expr.has_side_effects
            }, 

            RuntimeOp::Deref { mutable: _, expr } => {

                function_side_effects = calculate_side_effects(expr, scope_id, symbol_table);

                expr.has_side_effects
            },

            RuntimeOp::Call { callable, args } => {

                let mut has_local_side_effects = false;

                function_side_effects = calculate_side_effects(callable, scope_id, symbol_table);
                has_local_side_effects |= callable.has_side_effects;

                for arg in args {
                    function_side_effects |= calculate_side_effects(arg, scope_id, symbol_table);
                    has_local_side_effects |= arg.has_side_effects;
                }

                has_local_side_effects
            },

            RuntimeOp::Return (expr) => {

                if let Some(expr) = expr {
                    function_side_effects = calculate_side_effects(expr, scope_id, symbol_table);
                } else {
                    function_side_effects = false;
                }

                // Unconditional control flow always has local side effects
                HAS_LOCAL_SIDE_EFFECTS
            },

            RuntimeOp::Continue |
            RuntimeOp::Break
             => {
                // No function side effects for unconditional control flow operations
                function_side_effects = false;

                // Unconditional control flow operations always have local side effects
                HAS_LOCAL_SIDE_EFFECTS
            },

            RuntimeOp::MakeArray { elements } => {
                // Has side effects if any of its elements has side effects
    
                function_side_effects = false;
                let mut has_local_side_effects = false;
                for element in elements {
                    function_side_effects |= calculate_side_effects(element, scope_id, symbol_table);
                    has_local_side_effects |= element.has_side_effects;
                }
    
                has_local_side_effects
            },
        },

        SyntaxNodeValue::IfChain { if_blocks, else_block } => {

            function_side_effects = false;
            for if_block in if_blocks {
                function_side_effects |= calculate_side_effects(&mut if_block.condition, scope_id, symbol_table);
                function_side_effects |= calculate_side_effects_block(&mut if_block.body, symbol_table);
            }

            if let Some(else_block) = else_block {
                function_side_effects |= calculate_side_effects_block(else_block, symbol_table);
            }

            NO_LOCAL_SIDE_EFFECTS
        },

        SyntaxNodeValue::While { condition, body } |
        SyntaxNodeValue::DoWhile { body, condition }
         => {
            function_side_effects = calculate_side_effects(condition, scope_id, symbol_table);
            function_side_effects |= calculate_side_effects_block(body, symbol_table);

            NO_LOCAL_SIDE_EFFECTS
        
        },

        SyntaxNodeValue::Loop { body }=> {

            function_side_effects = calculate_side_effects_block(body, symbol_table);

            NO_LOCAL_SIDE_EFFECTS
        },

        SyntaxNodeValue::Scope(inner_block) => {

            function_side_effects = calculate_side_effects_block(inner_block, symbol_table);

            // The local side effects internal to the block don't make it to the parent.
            NO_LOCAL_SIDE_EFFECTS
        },

        SyntaxNodeValue::As { target_type: _, expr } => {

            function_side_effects = calculate_side_effects(expr, scope_id, symbol_table);

            expr.has_side_effects
        },
        
        // A value has no side effects. Side effects may arise when the value is used in an operation.
        SyntaxNodeValue::Symbol { .. } |
        SyntaxNodeValue::Literal(_)
        => {
            function_side_effects = false;
            NO_LOCAL_SIDE_EFFECTS
        },
        
        _ => unreachable!("Unexpected node during side effects calculation: {:?}. This is a bug.", node)
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

    symbol_table.set_function_side_effects(function.name, function.parent_scope, function.has_side_effects);
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
        println!("\n\nAfter symbol resolution:\n{:#?}", functions);
    }
    
    warn_unused_symbols(&block, symbol_table, source);

    calculate_side_effects_functions(&mut functions, symbol_table);

    if optimization_flags.evaluate_constants {
        evaluate_constants_functions(&mut functions, symbol_table, source);
        if verbose {
            println!("\n\nAfter constant expression evaluation:\n{:#?}", functions);
        }
    }
    
    functions
}

