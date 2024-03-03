use std::borrow::Borrow;
use std::collections::HashMap;

use rusty_vm_lib::ir::SourceCode;

use crate::cli_parser::OptimizationFlags;
use crate::lang::error;
use crate::match_unreachable;
use crate::symbol_table::{FunctionCode, FunctionConstantness, FunctionUUID, ScopeID, SymbolTable, SymbolValue};
use crate::function_parser::Function;
use crate::lang::data_types::{DataType, LiteralValue, Number};
use crate::ast::{RuntimeOp, ScopeBlock, SyntaxNode, SyntaxNodeValue};

use super::{FunctionIR, IRNode, IROperator, IRScopeID, IRValue, Label, LabelID, Tn, TnID};


/// Generates a sequence of unique ids for the IR code
pub struct IRIDGenerator {
    next_tn: TnID,
    next_label: LabelID,
}

impl IRIDGenerator {

    /// Create a new sequential Tn generator
    pub fn new() -> Self {
        Self { 
            next_tn: TnID(0),
            next_label: LabelID(0),
        }
    }

    /// Get the next Tn
    pub fn next_tn(&mut self) -> TnID {
        let old = self.next_tn;
        self.next_tn = TnID(old.0 + 1);
        old
    }

    /// Get the next Label
    pub fn next_label(&mut self) -> Label {
        let old = self.next_label;
        self.next_label = LabelID(old.0 + 1);
        Label(old)
    }

}


struct LoopLabels {
    /// The start of the loop body, does not include the condition check.
    /// If the condition is met, the program should jump here.
    pub start: Label,
    /// This is where the loop's condition is checked.
    /// This is optional because not all loops have a condition (e.g. `loop`).
    pub check: Option<Label>,
    /// After the loop, every instruction after this label does not belong to the loop.
    /// Break statements should jump here.
    pub after: Label,
}


/// Recursively generate IR code for the given node and return where its value is stored, if it's an expression 
#[allow(clippy::too_many_arguments)]
fn generate_node<'a>(node: SyntaxNode<'a>, target: Option<Tn>, outer_loop: Option<&LoopLabels>, irid_gen: &mut IRIDGenerator, ir_function: &mut FunctionIR<'a>, ir_scope: IRScopeID, st_scope: ScopeID, symbol_table: &mut SymbolTable, source: &SourceCode) -> Option<Tn> {
    
    match node.value {

        SyntaxNodeValue::RuntimeOp(op) => match op {

            RuntimeOp::Add { left, right } => {

                let target = target.unwrap_or_else(|| Tn { id: irid_gen.next_tn(), data_type: node.data_type });

                let l_value = generate_node(*left, None, outer_loop, irid_gen, ir_function, ir_scope, st_scope, symbol_table, source).expect("Expected an expression");
                let r_value = generate_node(*right, None, outer_loop, irid_gen, ir_function, ir_scope, st_scope, symbol_table, source).expect("Expected an expression");

                ir_function.push_code(IRNode {
                    op: IROperator::Add {
                        target: target.clone(),
                        left: IRValue::Tn(l_value),
                        right: IRValue::Tn(r_value),
                    },
                    has_side_effects: node.has_side_effects
                });

                Some(target)
            },

            RuntimeOp::Sub { left, right } => {

                let target = target.unwrap_or_else(|| Tn { id: irid_gen.next_tn(), data_type: node.data_type });

                let l_value = generate_node(*left, None, outer_loop, irid_gen, ir_function, ir_scope, st_scope, symbol_table, source).expect("Expected an expression");
                let r_value = generate_node(*right, None, outer_loop, irid_gen, ir_function, ir_scope, st_scope, symbol_table, source).expect("Expected an expression");

                ir_function.push_code(IRNode {
                    op: IROperator::Sub {
                        target: target.clone(),
                        left: IRValue::Tn(l_value),
                        right: IRValue::Tn(r_value),
                    },
                    has_side_effects: node.has_side_effects
                });

                Some(target)
            },

            RuntimeOp::Mul { left, right } => {

                let target = target.unwrap_or_else(|| Tn { id: irid_gen.next_tn(), data_type: node.data_type });

                let l_value = generate_node(*left, None, outer_loop, irid_gen, ir_function, ir_scope, st_scope, symbol_table, source).expect("Expected an expression");
                let r_value = generate_node(*right, None, outer_loop, irid_gen, ir_function, ir_scope, st_scope, symbol_table, source).expect("Expected an expression");

                ir_function.push_code(IRNode {
                    op: IROperator::Mul {
                        target: target.clone(),
                        left: IRValue::Tn(l_value),
                        right: IRValue::Tn(r_value),
                    },
                    has_side_effects: node.has_side_effects
                });

                Some(target)
            },

            RuntimeOp::Div { left, right } => {

                let target = target.unwrap_or_else(|| Tn { id: irid_gen.next_tn(), data_type: node.data_type });

                let l_value = generate_node(*left, None, outer_loop, irid_gen, ir_function, ir_scope, st_scope, symbol_table, source).expect("Expected an expression");
                let r_value = generate_node(*right, None, outer_loop, irid_gen, ir_function, ir_scope, st_scope, symbol_table, source).expect("Expected an expression");

                ir_function.push_code(IRNode {
                    op: IROperator::Div {
                        target: target.clone(),
                        left: IRValue::Tn(l_value),
                        right: IRValue::Tn(r_value),
                    },
                    has_side_effects: node.has_side_effects
                });

                Some(target)
            },

            RuntimeOp::Mod { left, right } => {

                let target = target.unwrap_or_else(|| Tn { id: irid_gen.next_tn(), data_type: node.data_type });

                let l_value = generate_node(*left, None, outer_loop, irid_gen, ir_function, ir_scope, st_scope, symbol_table, source).expect("Expected an expression");
                let r_value = generate_node(*right, None, outer_loop, irid_gen, ir_function, ir_scope, st_scope, symbol_table, source).expect("Expected an expression");

                ir_function.push_code(IRNode {
                    op: IROperator::Mod {
                        target: target.clone(),
                        left: IRValue::Tn(l_value),
                        right: IRValue::Tn(r_value),
                    },
                    has_side_effects: node.has_side_effects
                });

                Some(target)
            },

            RuntimeOp::Assign { left, right } => {
                
                let deref_assign = matches!(left.value, SyntaxNodeValue::RuntimeOp(RuntimeOp::Deref { .. }));

                let target = generate_node(*left, None, outer_loop, irid_gen, ir_function, ir_scope, st_scope, symbol_table, source).expect("Expected an assignable");


                if deref_assign {
                    // Assigning to a dereference, this is a different operation than regular assignment

                    let source = generate_node(*right, None, outer_loop, irid_gen, ir_function, ir_scope, st_scope, symbol_table, source).expect("Expected an expression");

                    ir_function.push_code(IRNode {
                            op: IROperator::DerefAssign {
                            target,
                            source: IRValue::Tn(source),
                        },
                        has_side_effects: node.has_side_effects
                    });

                } else {
                    // Regular assignment, no further processing is needed

                    generate_node(*right, Some(target), outer_loop, irid_gen, ir_function, ir_scope, st_scope, symbol_table, source);
    
                    // Adding an Assign node is superfluous since genetate_node for the source node has already assigned the value to the target
    
                }

                None
            },

            RuntimeOp::Deref { mutable: _, expr } => {

                let target = target.unwrap_or_else(|| Tn { id: irid_gen.next_tn(), data_type: node.data_type });

                let ref_ = generate_node(*expr, None, outer_loop, irid_gen, ir_function, ir_scope, st_scope, symbol_table, source).expect("Expected a reference");

                ir_function.push_code(IRNode {
                    op: IROperator::Deref {
                        target: target.clone(),
                        ref_: IRValue::Tn(ref_),
                    },
                    has_side_effects: node.has_side_effects
                });

                Some(target)
            },

            RuntimeOp::Ref { mutable: _, expr } => {

                let target = target.unwrap_or_else(|| Tn { id: irid_gen.next_tn(), data_type: node.data_type });

                let ref_ = generate_node(*expr, None, outer_loop, irid_gen, ir_function, ir_scope, st_scope, symbol_table, source).expect("Expected a reference");

                ir_function.push_code(IRNode {
                    op: IROperator::Ref {
                        target: target.clone(),
                        ref_,
                    },
                    has_side_effects: node.has_side_effects
                });

                Some(target)
            },

            RuntimeOp::Call { callable, args } => {
                /*
                    Tcallable = <callable>
                    [Targ-n = <arg-n>...]
                    [Tresult =] call Tcallable [Targ-n...] return: Lreturn
                    Lreturn:
                */

                // TODO: const functions should be evaluated at this point
                
                let return_target = if !matches!(*node.data_type, DataType::Void) {
                    target.or_else(|| Some(Tn { id: irid_gen.next_tn(), data_type: node.data_type }))
                } else {
                    None
                };

                if let SyntaxNodeValue::Symbol { name, scope_discriminant  } = &callable.value {

                    let function_symbol = symbol_table.get_symbol(st_scope, name, *scope_discriminant).unwrap().borrow();
                    // Assume the symbol is a function. The type checker should have already checked this.
                    let function_info = match_unreachable!(SymbolValue::Function(function_info) = &function_symbol.value, function_info);

                    if matches!(function_info.constantness, FunctionConstantness::ProvenConst) {
                        // Evaluate the const function if the arguments are also constant

                        let args_are_constant = args.iter()
                            .all(|arg| arg.known_literal_value(st_scope, symbol_table).is_some());
                        
                        if args_are_constant {
                            let literal_args = args.into_iter()
                                .map(|arg| arg.known_literal_value(st_scope, symbol_table).unwrap());
                            
                            todo!("evaluate const function")
                        }
                    }

                }
                
                let callable = generate_node(*callable, None, outer_loop, irid_gen, ir_function, ir_scope, st_scope, symbol_table, source).expect("Expected a callable expression");
                let args: Vec<IRValue> = args.into_iter().map(
                    |arg| generate_node(arg, None, outer_loop, irid_gen, ir_function, ir_scope, st_scope, symbol_table, source).expect("Argument is expected to be an expression")
                ).map(IRValue::Tn)
                    .collect();

                let return_label = irid_gen.next_label();
                
                ir_function.push_code(IRNode {
                    op: IROperator::Call {
                        return_target: return_target.clone(),
                        return_label,
                        callable,
                        args,
                    },
                    has_side_effects: node.has_side_effects // This will be true, but future changes could break this, though unlikely
                });

                ir_function.push_code(IRNode {
                    op: IROperator::Label { 
                        label: return_label
                    },
                    has_side_effects: false // Labels are not operations and thus have no side effects
                });

                return_target
            },

            RuntimeOp::Return(expr) => {
                
                if let Some(expr) = expr {

                    let return_tn = ir_function.scope_table.return_tn(ir_scope).expect("The function returns a value, but no return Tn was supplied");
                    generate_node(*expr, Some(return_tn), outer_loop, irid_gen, ir_function, ir_scope, st_scope, symbol_table, source).expect("Expected a value");

                } else {
                    // No return value is provided and the function should return void
                    assert!(ir_function.scope_table.return_tn(ir_scope).is_none());
                }
                
                // Jump to the function's exit label, which will take care of popping the stack and returning to the caller
                ir_function.push_code(IRNode {
                    op: IROperator::Jump { 
                        target: ir_function.function_labels.exit
                    },
                    has_side_effects: node.has_side_effects
                });

                None
            },

            RuntimeOp::Equal { left, right } => {

                let target = target.unwrap_or_else(|| Tn { id: irid_gen.next_tn(), data_type: node.data_type });

                let l_value = generate_node(*left, None, outer_loop, irid_gen, ir_function, ir_scope, st_scope, symbol_table, source).expect("Expected an expression");
                let r_value = generate_node(*right, None, outer_loop, irid_gen, ir_function, ir_scope, st_scope, symbol_table, source).expect("Expected an expression");

                ir_function.push_code(IRNode { 
                    op: IROperator::Equal {
                        target: target.clone(),
                        left: IRValue::Tn(l_value),
                        right: IRValue::Tn(r_value),
                    },
                    has_side_effects: node.has_side_effects
                });

                Some(target)
            },

            RuntimeOp::NotEqual { left, right } => {

                let target = target.unwrap_or_else(|| Tn { id: irid_gen.next_tn(), data_type: node.data_type });

                let l_value = generate_node(*left, None, outer_loop, irid_gen, ir_function, ir_scope, st_scope, symbol_table, source).expect("Expected an expression");
                let r_value = generate_node(*right, None, outer_loop, irid_gen, ir_function, ir_scope, st_scope, symbol_table, source).expect("Expected an expression");

                ir_function.push_code(IRNode {
                    op: IROperator::NotEqual {
                        target: target.clone(),
                        left: IRValue::Tn(l_value),
                        right: IRValue::Tn(r_value),
                    },
                    has_side_effects: node.has_side_effects
                });

                Some(target)
            },

            RuntimeOp::Greater { left, right } => {

                let target = target.unwrap_or_else(|| Tn { id: irid_gen.next_tn(), data_type: node.data_type });

                let l_value = generate_node(*left, None, outer_loop, irid_gen, ir_function, ir_scope, st_scope, symbol_table, source).expect("Expected an expression");
                let r_value = generate_node(*right, None, outer_loop, irid_gen, ir_function, ir_scope, st_scope, symbol_table, source).expect("Expected an expression");

                ir_function.push_code(IRNode {
                    op: IROperator::Greater {
                        target: target.clone(),
                        left: IRValue::Tn(l_value),
                        right: IRValue::Tn(r_value),
                    },
                    has_side_effects: node.has_side_effects
                });

                Some(target)
            },

            RuntimeOp::Less { left, right } => {

                let target = target.unwrap_or_else(|| Tn { id: irid_gen.next_tn(), data_type: node.data_type });

                let l_value = generate_node(*left, None, outer_loop, irid_gen, ir_function, ir_scope, st_scope, symbol_table, source).expect("Expected an expression");
                let r_value = generate_node(*right, None, outer_loop, irid_gen, ir_function, ir_scope, st_scope, symbol_table, source).expect("Expected an expression");

                ir_function.push_code(IRNode {
                    op: IROperator::Less {
                        target: target.clone(),
                        left: IRValue::Tn(l_value),
                        right: IRValue::Tn(r_value),
                    },
                    has_side_effects: node.has_side_effects
                });

                Some(target)
            },

            RuntimeOp::GreaterEqual { left, right } => {

                let target = target.unwrap_or_else(|| Tn { id: irid_gen.next_tn(), data_type: node.data_type });

                let l_value = generate_node(*left, None, outer_loop, irid_gen, ir_function, ir_scope, st_scope, symbol_table, source).expect("Expected an expression");
                let r_value = generate_node(*right, None, outer_loop, irid_gen, ir_function, ir_scope, st_scope, symbol_table, source).expect("Expected an expression");

                ir_function.push_code(IRNode {
                    op: IROperator::GreaterEqual {
                        target: target.clone(),
                        left: IRValue::Tn(l_value),
                        right: IRValue::Tn(r_value),
                    },
                    has_side_effects: node.has_side_effects
                });

                Some(target)
            },

            RuntimeOp::LessEqual { left, right } => {

                let target = target.unwrap_or_else(|| Tn { id: irid_gen.next_tn(), data_type: node.data_type });

                let l_value = generate_node(*left, None, outer_loop, irid_gen, ir_function, ir_scope, st_scope, symbol_table, source).expect("Expected an expression");
                let r_value = generate_node(*right, None, outer_loop, irid_gen, ir_function, ir_scope, st_scope, symbol_table, source).expect("Expected an expression");

                ir_function.push_code(IRNode {
                    op: IROperator::LessEqual {
                        target: target.clone(),
                        left: IRValue::Tn(l_value),
                        right: IRValue::Tn(r_value),
                    },
                    has_side_effects: node.has_side_effects
                });

                Some(target)
            },

            RuntimeOp::LogicalNot (operand)=> {

                let target = target.unwrap_or_else(|| Tn { id: irid_gen.next_tn(), data_type: node.data_type });

                let operand = generate_node(*operand, None, outer_loop, irid_gen, ir_function, ir_scope, st_scope, symbol_table, source).expect("Expected an expression");

                ir_function.push_code(IRNode {
                    op: IROperator::BitNot {
                        target: target.clone(),
                        operand: IRValue::Tn(operand),
                    },
                    has_side_effects: node.has_side_effects
                });

                Some(target)
            },

            RuntimeOp::BitwiseNot (operand) => {

                let target = target.unwrap_or_else(|| Tn { id: irid_gen.next_tn(), data_type: node.data_type });

                let operand = generate_node(*operand, None, outer_loop, irid_gen, ir_function, ir_scope, st_scope, symbol_table, source).expect("Expected an expression");

                ir_function.push_code(IRNode {
                    op: IROperator::BitNot {
                        target: target.clone(),
                        operand: IRValue::Tn(operand),
                    },
                    has_side_effects: node.has_side_effects
                });

                Some(target)
            },

            RuntimeOp::LogicalAnd { left, right } => {

                let target = target.unwrap_or_else(|| Tn { id: irid_gen.next_tn(), data_type: node.data_type });

                let l_value = generate_node(*left, None, outer_loop, irid_gen, ir_function, ir_scope, st_scope, symbol_table, source).expect("Expected an expression");
                let r_value = generate_node(*right, None, outer_loop, irid_gen, ir_function, ir_scope, st_scope, symbol_table, source).expect("Expected an expression");

                ir_function.push_code(IRNode {
                    op: IROperator::BitAnd {
                        target: target.clone(),
                        left: IRValue::Tn(l_value),
                        right: IRValue::Tn(r_value),
                    },
                    has_side_effects: node.has_side_effects
                });

                Some(target)
            },

            RuntimeOp::LogicalOr { left, right } => {

                let target = target.unwrap_or_else(|| Tn { id: irid_gen.next_tn(), data_type: node.data_type });

                let l_value = generate_node(*left, None, outer_loop, irid_gen, ir_function, ir_scope, st_scope, symbol_table, source).expect("Expected an expression");
                let r_value = generate_node(*right, None, outer_loop, irid_gen, ir_function, ir_scope, st_scope, symbol_table, source).expect("Expected an expression");

                ir_function.push_code(IRNode {
                    op: IROperator::BitOr {
                        target: target.clone(),
                        left: IRValue::Tn(l_value),
                        right: IRValue::Tn(r_value),
                    },
                    has_side_effects: node.has_side_effects
                });

                Some(target)
            },

            RuntimeOp::BitShiftLeft { left, right } => {

                let target = target.unwrap_or_else(|| Tn { id: irid_gen.next_tn(), data_type: node.data_type });

                let l_value = generate_node(*left, None, outer_loop, irid_gen, ir_function, ir_scope, st_scope, symbol_table, source).expect("Expected an expression");
                let r_value = generate_node(*right, None, outer_loop, irid_gen, ir_function, ir_scope, st_scope, symbol_table, source).expect("Expected an expression");

                ir_function.push_code(IRNode {
                    op: IROperator::BitShiftLeft {
                        target: target.clone(),
                        left: IRValue::Tn(l_value),
                        right: IRValue::Tn(r_value),
                    }, 
                    has_side_effects: node.has_side_effects
                });

                Some(target)
            },

            RuntimeOp::BitShiftRight { left, right } => {

                let target = target.unwrap_or_else(|| Tn { id: irid_gen.next_tn(), data_type: node.data_type });

                let l_value = generate_node(*left, None, outer_loop, irid_gen, ir_function, ir_scope, st_scope, symbol_table, source).expect("Expected an expression");
                let r_value = generate_node(*right, None, outer_loop, irid_gen, ir_function, ir_scope, st_scope, symbol_table, source).expect("Expected an expression");

                ir_function.push_code(IRNode {
                    op: IROperator::BitShiftRight {
                        target: target.clone(),
                        left: IRValue::Tn(l_value),
                        right: IRValue::Tn(r_value),
                    },
                    has_side_effects: node.has_side_effects
                });

                Some(target)
            },

            RuntimeOp::BitwiseOr { left, right } => {

                let target = target.unwrap_or_else(|| Tn { id: irid_gen.next_tn(), data_type: node.data_type });

                let l_value = generate_node(*left, None, outer_loop, irid_gen, ir_function, ir_scope, st_scope, symbol_table, source).expect("Expected an expression");
                let r_value = generate_node(*right, None, outer_loop, irid_gen, ir_function, ir_scope, st_scope, symbol_table, source).expect("Expected an expression");

                ir_function.push_code(IRNode {
                    op: IROperator::BitOr {
                        target: target.clone(),
                        left: IRValue::Tn(l_value),
                        right: IRValue::Tn(r_value),
                    },
                    has_side_effects: node.has_side_effects
                });

                Some(target)
            },

            RuntimeOp::BitwiseAnd { left, right } => {

                let target = target.unwrap_or_else(|| Tn { id: irid_gen.next_tn(), data_type: node.data_type });

                let l_value = generate_node(*left, None, outer_loop, irid_gen, ir_function, ir_scope, st_scope, symbol_table, source).expect("Expected an expression");
                let r_value = generate_node(*right, None, outer_loop, irid_gen, ir_function, ir_scope, st_scope, symbol_table, source).expect("Expected an expression");

                ir_function.push_code(IRNode {
                    op: IROperator::BitAnd {
                        target: target.clone(),
                        left: IRValue::Tn(l_value),
                        right: IRValue::Tn(r_value),
                    },
                    has_side_effects: node.has_side_effects
                });

                Some(target)
            },

            RuntimeOp::BitwiseXor { left, right } => {

                let target = target.unwrap_or_else(|| Tn { id: irid_gen.next_tn(), data_type: node.data_type });

                let l_value = generate_node(*left, None, outer_loop, irid_gen, ir_function, ir_scope, st_scope, symbol_table, source).expect("Expected an expression");
                let r_value = generate_node(*right, None, outer_loop, irid_gen, ir_function, ir_scope, st_scope, symbol_table, source).expect("Expected an expression");

                ir_function.push_code(IRNode {
                    op: IROperator::BitXor {
                        target: target.clone(),
                        left: IRValue::Tn(l_value),
                        right: IRValue::Tn(r_value),
                    },
                    has_side_effects: node.has_side_effects
                });

                Some(target)
            },

            RuntimeOp::ArrayIndex { array, index } => {

                let element_type = match_unreachable!(DataType::Array { element_type, size: _ } = array.data_type.as_ref(), element_type.clone());
                let element_size = element_type.static_size()
                    .unwrap_or_else(|()| error::unknown_sizes(&[(node.token, element_type.clone())], source, &format!("Array element type has unknown size: {:?}. Array elements must have static sizes.", element_type)));

                let array_addr_tn = generate_node(*array, None, outer_loop, irid_gen, ir_function, ir_scope, st_scope, symbol_table, source).expect("Expected an array");
                let index_tn = generate_node(*index, None, outer_loop, irid_gen, ir_function, ir_scope, st_scope, symbol_table, source).expect("Expected an index");

                let offset_tn = Tn { id: irid_gen.next_tn(), data_type: DataType::Usize.into() };
                ir_function.push_code(IRNode {
                    op: IROperator::Mul {
                        target: offset_tn.clone(),
                        left: IRValue::Const(LiteralValue::Numeric(Number::Uint(element_size as u64)).into()),
                        right: IRValue::Tn(index_tn),
                    },
                    has_side_effects: node.has_side_effects
                });

                let element_addr_tn = Tn { id: irid_gen.next_tn(), data_type: DataType::Ref { target: element_type.clone(), mutable: true }.into() };
                ir_function.push_code(IRNode {
                    op: IROperator::Add {
                        target: element_addr_tn.clone(),
                        left: IRValue::Tn(array_addr_tn),
                        right: IRValue::Tn(offset_tn),
                    },
                    has_side_effects: node.has_side_effects
                });

                let target = target.unwrap_or_else(|| Tn { id: irid_gen.next_tn(), data_type: element_type });

                ir_function.push_code(IRNode {
                    op: IROperator::Deref { 
                        target: target.clone(),
                        ref_: IRValue::Tn(element_addr_tn),
                    },
                    has_side_effects: node.has_side_effects
                });

                Some(target)
            },

            RuntimeOp::Break => {

                let loop_labels = outer_loop.expect("Break statement outside of a loop");

                ir_function.push_code( IRNode {
                    op: IROperator::Jump { 
                        target: loop_labels.after 
                    },
                    has_side_effects: node.has_side_effects
                });

                None
            },

            RuntimeOp::Continue => {

                let loop_labels = outer_loop.expect("Continue statement outside of a loop");
                
                ir_function.push_code(IRNode {
                    op: IROperator::Jump {
                        target: loop_labels.start
                    },
                    has_side_effects: node.has_side_effects
                });
                
                None
            },

            RuntimeOp::MakeArray { elements } => {
                // Set each array item to the corresponding value
    
                let element_type = match_unreachable!(DataType::Array { element_type, size: _ } = node.data_type.as_ref(), element_type.clone());
                let element_size = element_type.static_size()
                    .unwrap_or_else(|()| error::unknown_sizes(&[(node.token, element_type.clone())], source, &format!("Array element type has unknown size: {:?}. Array elements must have static sizes.", element_type)));
    
                let target = target.unwrap_or_else(|| Tn { id: irid_gen.next_tn(), data_type: node.data_type });
    
                // Reference to the first element of the array, for now. This will be incremented for each element
                let arr_element_ptr = Tn { id: irid_gen.next_tn(), data_type: DataType::Ref { target: element_type.clone(), mutable: true }.into() };
    
                // Get the address of the array (and store it into the `element_ptr` Tn)
                ir_function.push_code(IRNode {
                    op: IROperator::Ref { 
                        target: arr_element_ptr.clone(), 
                        ref_: target.clone() 
                    },
                    has_side_effects: node.has_side_effects
                });
                
                // Initialize each element
                for element_node in elements {
                    
                    // Generate the code for the element and store the result in the array
                    let element_tn = generate_node(element_node, None, outer_loop, irid_gen, ir_function, ir_scope, st_scope, symbol_table, source).expect("Expected an expression");
    
                    ir_function.push_code(IRNode {
                        op: IROperator::DerefCopy {
                            target: arr_element_ptr.clone(),
                            source: IRValue::Tn(element_tn),
                        },
                        has_side_effects: node.has_side_effects
                    });
    
                    // Increment the pointer to the next element
                    ir_function.push_code(IRNode {
                        op: IROperator::Add { 
                            target: arr_element_ptr.clone(), 
                            left: IRValue::Tn(arr_element_ptr.clone()), 
                            right: IRValue::Const(LiteralValue::Numeric(Number::Uint(element_size as u64)).into())
                        },
                        has_side_effects: node.has_side_effects
                    });
                }
    
                Some(target)
            }, 
        },

        SyntaxNodeValue::Literal (value) => {
            let target = target.unwrap_or_else(|| Tn { id: irid_gen.next_tn(), data_type: node.data_type });

            ir_function.push_code(IRNode {
                op: IROperator::Assign {
                    target: target.clone(),
                    source: IRValue::Const(value),
                },
                has_side_effects: node.has_side_effects
            });

            Some(target)
        },

        SyntaxNodeValue::Symbol { name, scope_discriminant } => {

            // Try to get the symbol's Tn
            let symbol_tn = ir_function.scope_table.get_tn(name, scope_discriminant, ir_scope);
            
            if let Some(target) = target {
                // The symbol should be loaded into `target`
                // Assume the symbol has already been mapped to a Tn

                let symbol_tn =  symbol_tn.expect("Symbol not found in scope table, but it's being read");
                    
                ir_function.push_code(IRNode {
                    op: IROperator::Assign { 
                        target: target.clone(),
                        source: IRValue::Tn(symbol_tn)
                    },
                    has_side_effects: node.has_side_effects
                });

                Some(target)

            } else {

                let target = if let Some(tn) = symbol_tn {
                    tn
                } else {
                    let tn = Tn { id: irid_gen.next_tn(), data_type: node.data_type };
                    ir_function.scope_table.map_symbol(name, tn.clone(), ir_scope);
                    tn
                };

                Some(target)
            }
        },

        SyntaxNodeValue::As { target_type, expr } => {
            // Just reinterpret the bits (drop excess bits or add padding if necessary)
            // Assume the conversion is possible, since the parser should have already checked that

            let mut target = target.unwrap_or_else(|| Tn { id: irid_gen.next_tn(), data_type: node.data_type });

            let src_size = expr.data_type.static_size();
            let target_size = target_type.static_size();

            match src_size.cmp(&target_size) {
                std::cmp::Ordering::Less => {
                    // The source has less bits, so create a copy with padding
                    // Reading directly from the source would read garbage and writing would overwrite surrounding memory.
                    // Copying is cheap since type casting is only allowed on primitives, which are usually small.

                    let expr_tn = generate_node(*expr, None, outer_loop, irid_gen, ir_function, ir_scope, st_scope, symbol_table, source).expect("Expected an expression");

                    target.data_type = target_type;

                    ir_function.push_code(IRNode {
                        op: IROperator::Copy {
                            target: target.clone(),
                            source: IRValue::Tn(expr_tn),
                        },
                        has_side_effects: node.has_side_effects
                    });

                    Some(target)
                },
                std::cmp::Ordering::Equal |
                std::cmp::Ordering::Greater
                 => {
                    // No need to do anything, just reinterpret the bits as the new type.
                    // If the source and target have the same size, the bits are already in the correct format.
                    // If the source has more bits than the target, the excess bits are simply ignored.
                    
                    generate_node(*expr, Some(target.clone()), outer_loop, irid_gen, ir_function, ir_scope, st_scope, symbol_table, source).expect("Expected an expression");
                    
                    // Just change the type of the Tn
                    target.data_type = target_type;
                    Some(target)
                },
            }
        },

        SyntaxNodeValue::IfChain { if_blocks, else_block } => {
            /*
                Tcondition = <condition>
                jumpifnot Tcondition Lnext
                <if_block>
                jump Lafter
                Lnext:
                    Tcondition = <condition>
                    jumpifnot Tcondition L
                Lelse:
                    <else_block>
                Lafter:
            */

            let mut next_if_block = irid_gen.next_label();
            let after_chain = irid_gen.next_label();

            for if_block in if_blocks {
                
                let condition_tn = generate_node(if_block.condition, None, outer_loop, irid_gen, ir_function, ir_scope, st_scope, symbol_table, source).expect("Expected a condition");

                ir_function.push_code(IRNode {
                    op: IROperator::JumpIfNot {
                        condition: condition_tn,
                        target: next_if_block
                    },
                    has_side_effects: node.has_side_effects
                });

                let inner_ir_scope = ir_function.scope_table.add_scope(Some(ir_scope));
                generate_block(if_block.body, target.clone(), outer_loop, irid_gen, ir_function, inner_ir_scope, symbol_table, source);

                ir_function.push_code(IRNode {
                    op: IROperator::Jump {
                        target: after_chain
                    },
                    has_side_effects: node.has_side_effects
            });

                // If there's no else block, this label coincides with the after_chain label. This is ok, though, since labels are no-ops.
                ir_function.push_code(IRNode {
                    op: IROperator::Label {
                        label: next_if_block
                    },
                    has_side_effects: false
            });
                
                // A redundant label is generated at the last iteration of the loop, but that's ok since this operation is cheap and labels don't have to be serial.
                next_if_block = irid_gen.next_label();
            }

            if let Some(else_block) = else_block {
                let inner_ir_scope = ir_function.scope_table.add_scope(Some(ir_scope));
                generate_block(else_block, target, outer_loop, irid_gen, ir_function, inner_ir_scope, symbol_table, source);
            }

            // Return None because the if-chain's return value is stored in the target Tn by the if-blocks
            None
        },

        SyntaxNodeValue::While { condition, body } => {
            /*
                jump Lcheck
                Lstart:
                    <body>
                Lcheck:
                    Tcondition = <condition>
                    jumpifnot Tcondition Lafter
                    jump Lstart
                Lafter:
            */
            
            let loop_labels = LoopLabels {
                start: irid_gen.next_label(),
                check: Some(irid_gen.next_label()),
                after: irid_gen.next_label(),
            };

            ir_function.push_code(IRNode {
                op: IROperator::Jump { 
                    target: loop_labels.check.unwrap() 
                },
                has_side_effects: node.has_side_effects
            });

            ir_function.push_code(IRNode {
                op: IROperator::Label { 
                    label: loop_labels.start 
                },
                has_side_effects: node.has_side_effects
            });

            let inner_ir_scope = ir_function.scope_table.add_scope(Some(ir_scope));
            generate_block(body, None, Some(&loop_labels), irid_gen, ir_function, inner_ir_scope, symbol_table, source);

            ir_function.push_code(IRNode {
                op: IROperator::Label {
                    label: loop_labels.check.unwrap()
                },
                has_side_effects: false
            });
            
            let condition_tn = generate_node(*condition, None, Some(&loop_labels), irid_gen, ir_function, ir_scope, st_scope, symbol_table, source).expect("Expected a condition");
       
            ir_function.push_code(IRNode {
                op: IROperator::JumpIf {
                    condition: condition_tn,
                    target: loop_labels.start
                },
                has_side_effects: node.has_side_effects
            });     
       
            ir_function.push_code(IRNode {
                op: IROperator::Label {
                    label: loop_labels.after
                },
                has_side_effects: node.has_side_effects
            });

            None
        },

        SyntaxNodeValue::Loop { body } => {
            /*
                Lstart:
                    <body>
                jump Lstart
                Lafter:
            */

            let loop_labels = LoopLabels {
                start: irid_gen.next_label(),
                check: None,
                after: irid_gen.next_label(),
            };

            // Put the label inside the scope bounds to avoid push-popping the scope for every iteration
            ir_function.push_code(IRNode {
                op: IROperator::Label {
                    label: loop_labels.start 
                },
                has_side_effects: false
            });

            let inner_ir_scope = ir_function.scope_table.add_scope(Some(ir_scope));
            generate_block(body, None, Some(&loop_labels), irid_gen, ir_function, inner_ir_scope, symbol_table, source);

            ir_function.push_code(IRNode {
                op: IROperator::Jump {
                    target: loop_labels.start 
                },
                has_side_effects: node.has_side_effects
            });

            ir_function.push_code(IRNode {
                op: IROperator::Label {
                    label: loop_labels.after 
                },
                has_side_effects: false
            });

            None
        },

        SyntaxNodeValue::DoWhile { body, condition } => {
            /*
                Lstart:
                    <body>
                Lcheck:
                    Tcondition = <condition>
                    jumpif Tcondition Lstart
                Lafter:
            */

            let loop_labels = LoopLabels {
                start: irid_gen.next_label(),
                check: Some(irid_gen.next_label()),
                after: irid_gen.next_label(),
            };

            ir_function.push_code(IRNode {
                op: IROperator::Label {
                    label: loop_labels.start 
                },
                has_side_effects: false
            });

            let inner_ir_scope = ir_function.scope_table.add_scope(Some(ir_scope));
            generate_block(body, None, Some(&loop_labels), irid_gen, ir_function, inner_ir_scope, symbol_table, source);

            ir_function.push_code(IRNode {
                op: IROperator::Label {
                    label: loop_labels.check.unwrap() 
                },
                has_side_effects: false
            });

            let condition_tn = generate_node(*condition, None, Some(&loop_labels), irid_gen, ir_function, ir_scope, st_scope, symbol_table, source).expect("Expected a condition");

            ir_function.push_code(IRNode {
                op: IROperator::JumpIf {
                    condition: condition_tn,
                    target: loop_labels.start 
                }, 
                has_side_effects: node.has_side_effects
            });

            ir_function.push_code(IRNode {
                op: IROperator::Label {
                    label: loop_labels.after 
                },
                has_side_effects: false
            });

            None
        },
        
        SyntaxNodeValue::Scope(block) => {

            let inner_ir_scope = ir_function.scope_table.add_scope(Some(ir_scope));

            generate_block(block, target, outer_loop, irid_gen, ir_function, inner_ir_scope, symbol_table, source);

            None
        },

        SyntaxNodeValue::FunctionParams(_) |
        SyntaxNodeValue::DataType(_) |
        SyntaxNodeValue::Function { .. } |
        SyntaxNodeValue::Const { .. } |
        SyntaxNodeValue::Static { .. } |
        SyntaxNodeValue::TypeDef { .. } |
        SyntaxNodeValue::Placeholder 
            => unreachable!("{:?} is not expected. This is a bug.", node)
    }
}


/// Recursively generate the IR code for the given ScopeBlock.
/// This function does not take care of pushing and popping the block's scope, so manual stack managenent is required.
/// Manual scope management is required to produce more efficient code based on the context.
#[allow(clippy::too_many_arguments)]
fn generate_block<'a>(mut block: ScopeBlock<'a>, target: Option<Tn>, outer_loop: Option<&LoopLabels>, irid_gen: &mut IRIDGenerator, ir_function: &mut FunctionIR<'a>, ir_scope: IRScopeID, symbol_table: &mut SymbolTable, source: &SourceCode) {

    // Don't generate IR code for empty blocks
    // An empty block may exist due to internal optimizations (e.g. useless code removal)
    if block.statements.is_empty() {
        return;
    }

    for statement in block.statements.drain(0..block.statements.len() - 1) {

        generate_node(statement, None, outer_loop, irid_gen, ir_function, ir_scope, block.scope_id, symbol_table, source);

    }

    let last_statement = block.statements.pop().unwrap();
    generate_node(last_statement, target, outer_loop, irid_gen, ir_function, ir_scope, block.scope_id, symbol_table, source);

}


pub struct FunctionLabels {
    /// The first instruction of the function (pushing the function's scope onto the stack)
    /// This label should be the target of function calls.
    pub start: Label,
    /// The exiting instructions of the function (popping the scope and returning to the caller).
    /// This label should be the target of return statements.
    pub exit: Label,

}


/// Recursively generate the IR code for the given function.
fn generate_function<'a>(function: Function<'a>, irid_gen: &mut IRIDGenerator, symbol_table: &mut SymbolTable, source: &SourceCode) -> FunctionIR<'a> {
    /*
        Lstart:
            pushscope <function_size>
            <function_code>
        Lexit:
            popscope <function_size>
            return
    */

    let mut ir_function = FunctionIR::new(function.name, function.code.scope_id, irid_gen);

    // Create the top-level function scope
    let ir_scope = ir_function.scope_table.add_scope(None);

    // Create a Tn for the return value, if it isn't Void. Non-void return statements will assign to this Tn
    let return_type = function.return_type();
    let return_tn = if *return_type != DataType::Void {
        let return_tn = Tn { id: irid_gen.next_tn(), data_type: return_type };
        ir_function.scope_table.scopes[ir_scope.0].return_tn = Some(return_tn.clone());
        Some(return_tn)
    } else {
        None
    };

    // Put the label before pushing the function's scope
    ir_function.push_code(IRNode {
        op: IROperator::Label { label: ir_function.function_labels.start },
        has_side_effects: false
    });

    let function_size = match symbol_table.total_scope_size(function.code.scope_id) {
        Ok(size) => size,
        Err(unknowns) => error::unknown_sizes(&unknowns, source, "All local sizes should be known at compile-time.")
    };

    ir_function.push_code(IRNode {
        op: IROperator::PushScope { bytes: function_size },
        has_side_effects: false
    });

    symbol_table.map_function_label(FunctionUUID { name: function.name.to_string(), scope: function.parent_scope }, ir_function.function_labels.start);

    generate_block(function.code, return_tn, None, irid_gen, &mut ir_function, ir_scope, symbol_table, source);

    ir_function.push_code(IRNode {
        op: IROperator::Label { label: ir_function.function_labels.exit },
        has_side_effects: false
    });

    ir_function.push_code(IRNode {
        op: IROperator::PopScope { bytes: function_size },
        has_side_effects: false
    });

    ir_function.push_code(IRNode {
        op: IROperator::Return,
        has_side_effects: false
    });

    ir_function
}


/// Reverse iteration over the IR code to remove operations whose result is never used
/// Starting from the back, when a Tn is assigned to but never read, the assignment is removed.
fn remove_unread_operations(ir_function: &mut FunctionIR) {
    /*
        Statements marked as having side effects won't be removed, even if their result is never read.
        Their result will probably have effects outside of the function's scope (or I/O).
        Inside an macro-operation with side effects, there may be operations that do not have side effects, and they can be remvoved.
    */

    let mut function_code = ir_function.code.borrow_mut();

    let mut node_ptr = unsafe { function_code.tail() };

    // Allocate at least as much hashmap slots as the maximum number of Tns that will ever be inserted.
    // This isn't a bad estimate since almost every operation assigns to a Tn.
    // Also, the memory will be freed upon returning from this function.
    let mut read_tns: HashMap<TnID, ()> = HashMap::with_capacity(function_code.length());

    while let Some(node) = unsafe { node_ptr.as_ref() } {

        // Do not remove operations that have side effects
        if node.data.has_side_effects {
            node_ptr = unsafe { node.prev() };
            continue;
        }

        match &node.data.op {
            IROperator::Add { target, left, right } |
            IROperator::Sub { target, left, right } |
            IROperator::Mul { target, left, right } |
            IROperator::Div { target, left, right } |
            IROperator::Mod { target, left, right } |
            IROperator::GreaterEqual { target, left, right } |
            IROperator::LessEqual { target, left, right } |
            IROperator::Equal { target, left, right } |
            IROperator::NotEqual { target, left, right } |
            IROperator::BitAnd { target, left, right } |
            IROperator::BitOr { target, left, right } |
            IROperator::BitXor { target, left, right } |
            IROperator::Greater { target, left, right } |
            IROperator::Less { target, left, right } |
            IROperator::BitShiftLeft { target, left, right } |
            IROperator::BitShiftRight { target, left, right } 
            => {
                // If the target is never read, the operation result is useless
                if !read_tns.contains_key(&target.id) {
                    // The target is never read, so remove the operation
                    // Save the previous node in a temporary variable because removing the node from the list invalidates it.
                    let prev = unsafe { node.prev() };
                    unsafe { function_code.remove(node_ptr) };
                    node_ptr = prev;
                    continue;
                }

                // The target is read, so add the Tn operands to the list of read Tns
                if let IRValue::Tn(tn) = left {
                    read_tns.insert(tn.id, ());
                }

                if let IRValue::Tn(tn) = right {
                    read_tns.insert(tn.id, ());
                }
            },
            
            IROperator::Assign { target, source: operand } |
            IROperator::Deref { target, ref_: operand } |
            IROperator::DerefAssign { target, source: operand } |
            IROperator::BitNot { target, operand } |
            IROperator::Copy { target, source: operand } |
            IROperator::DerefCopy { target, source: operand }
             => {
                // If the target is never read, the operation result is useless
                if !read_tns.contains_key(&target.id) {
                    // The target is never read, so remove the operation
                    // Save the previous node in a temporary variable because removing the node from the list invalidates it.
                    let prev = unsafe { node.prev() };
                    unsafe { function_code.remove(node_ptr) };
                    node_ptr = prev;
                    continue;
                }

                // The target is read, so add the Tn operands to the list of read Tns
                if let IRValue::Tn(tn) = operand {
                    read_tns.insert(tn.id, ());
                }
            },

            IROperator::Ref { target, ref_ } => {
                // If the target is never read, the operation result is useless
                if !read_tns.contains_key(&target.id) {
                    // The target is never read, so remove the operation
                    // Save the previous node in a temporary variable because removing the node from the list invalidates it.
                    let prev = unsafe { node.prev() };
                    unsafe { function_code.remove(node_ptr) };
                    node_ptr = prev;
                    continue;
                }

                read_tns.insert(ref_.id, ());
            },

            IROperator::JumpIf { condition, target: _ } |
            IROperator::JumpIfNot { condition, target: _ } => {
                read_tns.insert(condition.id, ());
            },

            IROperator::Call { return_target: _, return_label: _, callable: _, args } => {
                // The function will be called anyway. 
                // TODO: if there are no side effects to the functions, the call can be removed

                for arg in args {
                    if let IRValue::Tn(tn) = arg {
                        read_tns.insert(tn.id, ());
                    }
                }
            },

            IROperator::Jump { target: _ } |
            IROperator::Label { label: _ } |
            IROperator::Return |
            IROperator::PushScope { bytes: _ } |
            IROperator::PopScope { bytes: _ } |
            IROperator::Nop
             => {}
        }

        node_ptr = unsafe { node.prev() };
    }

}


/// Generate ir code from the given functions
pub fn generate<'a>(functions: Vec<Function<'a>>, symbol_table: &mut SymbolTable, optimization_flags: &OptimizationFlags, verbose: bool, source: &SourceCode) -> Vec<FunctionIR<'a>> {

    let mut ir_functions = Vec::new();
    let mut irid_gen = IRIDGenerator::new();

    if verbose {
        println!("\n\nGenerating IR code for the following functions:");
    }

    for function in functions {

        let mut ir_function = generate_function(function, &mut irid_gen, symbol_table, source);

        let mut function_symbol = symbol_table.get_function(ir_function.name, ir_function.parent_scope(symbol_table)).unwrap().borrow_mut();
        let function_info = match_unreachable!(SymbolValue::Function(function_info) = &mut function_symbol.value, function_info);
        function_info.code = Some(FunctionCode {
            label: ir_function.function_labels.start,
            code: ir_function.code.clone(),
        });

        if optimization_flags.remove_useless_code {
            remove_unread_operations(&mut ir_function);
        }

        ir_functions.push(
            ir_function
        );

        if verbose {
            println!("\n{}\n", ir_functions.last().unwrap());
        }
    }

    ir_functions
}

