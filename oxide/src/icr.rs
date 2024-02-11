use std::rc::Rc;
use std::collections::HashMap;

use rusty_vm_lib::ir::SourceCode;

use crate::match_unreachable;
use crate::symbol_table::{ScopeDiscriminant, SymbolTable};
use crate::function_parser::Function;
use crate::data_types::{DataType, LiteralValue};
use crate::token::{TokenKind, Value};
use crate::operations::Ops;
use crate::token_tree::{ChildrenType, ScopeBlock, TokenNode};


/// Generates a sequence of unique ids for the IR code
struct IRIDGenerator {
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


/// Represents a temporary variable
#[derive(Clone)]
pub struct Tn {
    pub id: TnID,
    pub data_type: Rc<DataType>,
}


#[derive(Clone)]
pub struct Label(pub LabelID);


/// Represents an operand of ir operations
pub enum IRValue {

    Tn (Tn),
    Label (Label),
    Const (LiteralValue),

}


#[derive(Clone, Copy)]
pub struct TnID(usize);

#[derive(Clone, Copy)]
pub struct LabelID(usize);


/// Stores information about a scope
pub struct IRScope {

    /// Maps symbol names to Tns
    symbols: HashMap<String, Vec<Tn>>, // TODO: eventually, use a &str or Cow<str> to avoid copying
    /// Keeps track of the return type and in which Tn to store it
    return_tn: Option<Tn>,
    parent: Option<IRScopeID>,

}

impl IRScope {

    pub fn new(parent: Option<IRScopeID>) -> Self {
        Self {
            symbols: HashMap::new(),
            return_tn: None,
            parent,
        }
    }

}


#[derive(Clone, Copy)]
pub struct IRScopeID(usize);

/// Stores information about scopes in the IR
pub struct ScopeTable {
    pub scopes: Vec<IRScope>,
}

impl ScopeTable {

    pub fn new() -> Self {
        Self {
            scopes: vec![IRScope::new(None)]
        }
    }

    /// Recursively get the function's return Tn, if it exists in a reachable scope
    pub fn return_tn(&self, ir_scope: IRScopeID) -> Option<Tn> {
        self.scopes[ir_scope.0].return_tn.clone()
            .or_else(|| self.scopes[ir_scope.0].parent
                .and_then(|parent| self.return_tn(parent)))
            
    }

    pub fn add_scope(&mut self, parent: Option<IRScopeID>) -> IRScopeID {
        self.scopes.push(IRScope::new(parent));
        IRScopeID(self.scopes.len() - 1)
    }

    /// Recursively get the Tn mapped to the given name, if it exists in a reachable scope
    pub fn get_tn(&mut self, name: &str, discriminant: ScopeDiscriminant, ir_scope: IRScopeID) -> Option<Tn> {
        self.scopes[ir_scope.0].symbols.get(name).map(|symbol_list| symbol_list[discriminant.0 as usize].clone())
            .or_else(|| self.scopes[ir_scope.0].parent
                .and_then(|parent| self.get_tn(name, discriminant, parent)))
    }

    pub fn map_symbol(&mut self, name: &str, tn: Tn, ir_scope: IRScopeID) {
        self.scopes[ir_scope.0].symbols.entry(name.to_string()).or_insert_with(Vec::new).push(tn);
    }

}


/// Represents an intermediate code operation
pub enum IRNode {

    Add { target: Tn, left: IRValue, right: IRValue },
    Sub { target: Tn, left: IRValue, right: IRValue },
    Mul { target: Tn, left: IRValue, right: IRValue },
    Div { target: Tn, left: IRValue, right: IRValue },
    Mod { target: Tn, left: IRValue, right: IRValue },
    
    Assign { target: Tn, source: IRValue },
    Deref { target: Tn, ref_: IRValue },
    Ref { target: Tn, ref_: IRValue },
    
    Greater { target: Tn, left: IRValue, right: IRValue },
    Less { target: Tn, left: IRValue, right: IRValue },
    GreaterEqual { target: Tn, left: IRValue, right: IRValue },
    LessEqual { target: Tn, left: IRValue, right: IRValue },
    Equal { target: Tn, left: IRValue, right: IRValue },
    NotEqual { target: Tn, left: IRValue, right: IRValue },
    
    LogicalAnd { target: Tn, left: IRValue, right: IRValue },
    LogicalOr { target: Tn, left: IRValue, right: IRValue },
    LogicalNot { target: Tn, operand: IRValue },
    
    BitShiftLeft { target: Tn, left: IRValue, right: IRValue },
    BitShiftRight { target: Tn, left: IRValue, right: IRValue },
    BitNot { target: Tn, operand: IRValue },
    BitAnd { target: Tn, left: IRValue, right: IRValue },
    BitOr { target: Tn, left: IRValue, right: IRValue },
    BitXor { target: Tn, left: IRValue, right: IRValue },
    
    Jump { target: Label },
    JumpIf { condition: Tn, target: Label },
    Label { label: Label },

    Call, // TODO
    Return,

    PushScope { bytes: usize },
    PopScope { bytes: usize },

    Nop,

}


pub struct FunctionIR {
    code: Vec<IRNode>,
    pub scope_table: ScopeTable,
}


impl FunctionIR {

    pub fn new() -> Self {
        Self {
            code: Vec::new(),
            scope_table: ScopeTable::new(),
        }
    }

    pub fn push(&mut self, node: IRNode) {
        self.code.push(node);
    }

}


/// Recursively generate IR code for the given node and return where its value is stored, if it's an expression 
fn generate_node(node: TokenNode, target: Option<Tn>, irid_gen: &mut IRIDGenerator, ir_code: &mut FunctionIR, ir_scope: IRScopeID, symbol_table: &mut SymbolTable, source: &SourceCode) -> Option<Tn> {
    
    match node.item.value {

        TokenKind::Op(op) => match op {
            Ops::Add => {
                let (l_node, r_node) = match_unreachable!(Some(ChildrenType::Binary(l_node, r_node)) = node.children, (l_node, r_node));

                let target = target.unwrap_or_else(|| Tn { id: irid_gen.next_tn(), data_type: node.data_type });

                let l_value = generate_node(*l_node, None, irid_gen, ir_code, ir_scope, symbol_table, source).expect("Expected an expression");
                let r_value = generate_node(*r_node, None, irid_gen, ir_code, ir_scope, symbol_table, source).expect("Expected an expression");

                ir_code.push(IRNode::Add {
                    target: target.clone(),
                    left: IRValue::Tn(l_value),
                    right: IRValue::Tn(r_value),
                });

                Some(target)
            },
            Ops::Sub => {
                let (l_node, r_node) = match_unreachable!(Some(ChildrenType::Binary(l_node, r_node)) = node.children, (l_node, r_node));

                let target = target.unwrap_or_else(|| Tn { id: irid_gen.next_tn(), data_type: node.data_type });

                let l_value = generate_node(*l_node, None, irid_gen, ir_code, ir_scope, symbol_table, source).expect("Expected an expression");
                let r_value = generate_node(*r_node, None, irid_gen, ir_code, ir_scope, symbol_table, source).expect("Expected an expression");

                ir_code.push(IRNode::Sub {
                    target: target.clone(),
                    left: IRValue::Tn(l_value),
                    right: IRValue::Tn(r_value),
                });

                Some(target)
            },
            Ops::Mul => {
                let (l_node, r_node) = match_unreachable!(Some(ChildrenType::Binary(l_node, r_node)) = node.children, (l_node, r_node));

                let target = target.unwrap_or_else(|| Tn { id: irid_gen.next_tn(), data_type: node.data_type });

                let l_value = generate_node(*l_node, None, irid_gen, ir_code, ir_scope, symbol_table, source).expect("Expected an expression");
                let r_value = generate_node(*r_node, None, irid_gen, ir_code, ir_scope, symbol_table, source).expect("Expected an expression");

                ir_code.push(IRNode::Mul {
                    target: target.clone(),
                    left: IRValue::Tn(l_value),
                    right: IRValue::Tn(r_value),
                });

                Some(target)
            },
            Ops::Div => {
                let (l_node, r_node) = match_unreachable!(Some(ChildrenType::Binary(l_node, r_node)) = node.children, (l_node, r_node));

                let target = target.unwrap_or_else(|| Tn { id: irid_gen.next_tn(), data_type: node.data_type });

                let l_value = generate_node(*l_node, None, irid_gen, ir_code, ir_scope, symbol_table, source).expect("Expected an expression");
                let r_value = generate_node(*r_node, None, irid_gen, ir_code, ir_scope, symbol_table, source).expect("Expected an expression");

                ir_code.push(IRNode::Div {
                    target: target.clone(),
                    left: IRValue::Tn(l_value),
                    right: IRValue::Tn(r_value),
                });

                Some(target)
            },
            Ops::Mod => {
                let (l_node, r_node) = match_unreachable!(Some(ChildrenType::Binary(l_node, r_node)) = node.children, (l_node, r_node));

                let target = target.unwrap_or_else(|| Tn { id: irid_gen.next_tn(), data_type: node.data_type });

                let l_value = generate_node(*l_node, None, irid_gen, ir_code, ir_scope, symbol_table, source).expect("Expected an expression");
                let r_value = generate_node(*r_node, None, irid_gen, ir_code, ir_scope, symbol_table, source).expect("Expected an expression");

                ir_code.push(IRNode::Mod {
                    target: target.clone(),
                    left: IRValue::Tn(l_value),
                    right: IRValue::Tn(r_value),
                });

                Some(target)
            },
            Ops::Assign => {
                let (l_node, r_node) = match_unreachable!(Some(ChildrenType::Binary(l_node, r_node)) = node.children, (l_node, r_node));
                
                let target = generate_node(*l_node, None, irid_gen, ir_code, ir_scope, symbol_table, source).expect("Expected an assignable");
                let src = generate_node(*r_node, Some(target), irid_gen, ir_code, ir_scope, symbol_table, source);
                assert!(src.is_none(), "generate_node should return None since a target is passed");

                // Adding an Assign node is superfluous since genetate_node for the source node has already assigned the value to the target

                None
            },
            Ops::Deref { mutable: _ } => {
                let ref_node = match_unreachable!(Some(ChildrenType::Unary(ref_node)) = node.children, ref_node);

                let target = target.unwrap_or_else(|| Tn { id: irid_gen.next_tn(), data_type: node.data_type });

                let ref_ = generate_node(*ref_node, None, irid_gen, ir_code, ir_scope, symbol_table, source).expect("Expected a reference");

                ir_code.push(IRNode::Deref {
                    target: target.clone(),
                    ref_: IRValue::Tn(ref_),
                });

                Some(target)
            },
            Ops::Ref { mutable: _ } => {
                let ref_node = match_unreachable!(Some(ChildrenType::Unary(ref_node)) = node.children, ref_node);

                let target = target.unwrap_or_else(|| Tn { id: irid_gen.next_tn(), data_type: node.data_type });

                let ref_ = generate_node(*ref_node, None, irid_gen, ir_code, ir_scope, symbol_table, source).expect("Expected a reference");

                ir_code.push(IRNode::Ref {
                    target: target.clone(),
                    ref_: IRValue::Tn(ref_),
                });

                Some(target)
            },
            Ops::FunctionCallOpen => todo!(),
            Ops::Return => {
                match node.children {
                    Some(ChildrenType::Unary(value_node)) => {
                        let return_tn = ir_code.scope_table.return_tn(ir_scope).expect("The function returns a value, but no return Tn was supplied");
                        generate_node(*value_node, Some(return_tn), irid_gen, ir_code, ir_scope, symbol_table, source).expect("Expected a value");
                    },
                    None => { 
                        // No return value is provided and the function should return void
                        assert!(ir_code.scope_table.return_tn(ir_scope).is_none());
                    },
                    _ => unreachable!("Return node has more than one child")
                }

                ir_code.push(IRNode::Return);

                None
            },
            Ops::Equal => {
                let (l_node, r_node) = match_unreachable!(Some(ChildrenType::Binary(l_node, r_node)) = node.children, (l_node, r_node));

                let target = target.unwrap_or_else(|| Tn { id: irid_gen.next_tn(), data_type: node.data_type });

                let l_value = generate_node(*l_node, None, irid_gen, ir_code, ir_scope, symbol_table, source).expect("Expected an expression");
                let r_value = generate_node(*r_node, None, irid_gen, ir_code, ir_scope, symbol_table, source).expect("Expected an expression");

                ir_code.push(IRNode::Equal {
                    target: target.clone(),
                    left: IRValue::Tn(l_value),
                    right: IRValue::Tn(r_value),
                });

                Some(target)
            },
            Ops::NotEqual => {
                let (l_node, r_node) = match_unreachable!(Some(ChildrenType::Binary(l_node, r_node)) = node.children, (l_node, r_node));

                let target = target.unwrap_or_else(|| Tn { id: irid_gen.next_tn(), data_type: node.data_type });

                let l_value = generate_node(*l_node, None, irid_gen, ir_code, ir_scope, symbol_table, source).expect("Expected an expression");
                let r_value = generate_node(*r_node, None, irid_gen, ir_code, ir_scope, symbol_table, source).expect("Expected an expression");

                ir_code.push(IRNode::NotEqual {
                    target: target.clone(),
                    left: IRValue::Tn(l_value),
                    right: IRValue::Tn(r_value),
                });

                Some(target)
            },
            Ops::Greater => {
                let (l_node, r_node) = match_unreachable!(Some(ChildrenType::Binary(l_node, r_node)) = node.children, (l_node, r_node));

                let target = target.unwrap_or_else(|| Tn { id: irid_gen.next_tn(), data_type: node.data_type });

                let l_value = generate_node(*l_node, None, irid_gen, ir_code, ir_scope, symbol_table, source).expect("Expected an expression");
                let r_value = generate_node(*r_node, None, irid_gen, ir_code, ir_scope, symbol_table, source).expect("Expected an expression");

                ir_code.push(IRNode::Greater {
                    target: target.clone(),
                    left: IRValue::Tn(l_value),
                    right: IRValue::Tn(r_value),
                });

                Some(target)
            },
            Ops::Less => {
                let (l_node, r_node) = match_unreachable!(Some(ChildrenType::Binary(l_node, r_node)) = node.children, (l_node, r_node));

                let target = target.unwrap_or_else(|| Tn { id: irid_gen.next_tn(), data_type: node.data_type });

                let l_value = generate_node(*l_node, None, irid_gen, ir_code, ir_scope, symbol_table, source).expect("Expected an expression");
                let r_value = generate_node(*r_node, None, irid_gen, ir_code, ir_scope, symbol_table, source).expect("Expected an expression");

                ir_code.push(IRNode::Less {
                    target: target.clone(),
                    left: IRValue::Tn(l_value),
                    right: IRValue::Tn(r_value),
                });

                Some(target)
            },
            Ops::GreaterEqual => {
                let (l_node, r_node) = match_unreachable!(Some(ChildrenType::Binary(l_node, r_node)) = node.children, (l_node, r_node));

                let target = target.unwrap_or_else(|| Tn { id: irid_gen.next_tn(), data_type: node.data_type });

                let l_value = generate_node(*l_node, None, irid_gen, ir_code, ir_scope, symbol_table, source).expect("Expected an expression");
                let r_value = generate_node(*r_node, None, irid_gen, ir_code, ir_scope, symbol_table, source).expect("Expected an expression");

                ir_code.push(IRNode::GreaterEqual {
                    target: target.clone(),
                    left: IRValue::Tn(l_value),
                    right: IRValue::Tn(r_value),
                });

                Some(target)
            },
            Ops::LessEqual => {
                let (l_node, r_node) = match_unreachable!(Some(ChildrenType::Binary(l_node, r_node)) = node.children, (l_node, r_node));

                let target = target.unwrap_or_else(|| Tn { id: irid_gen.next_tn(), data_type: node.data_type });

                let l_value = generate_node(*l_node, None, irid_gen, ir_code, ir_scope, symbol_table, source).expect("Expected an expression");
                let r_value = generate_node(*r_node, None, irid_gen, ir_code, ir_scope, symbol_table, source).expect("Expected an expression");

                ir_code.push(IRNode::LessEqual {
                    target: target.clone(),
                    left: IRValue::Tn(l_value),
                    right: IRValue::Tn(r_value),
                });

                Some(target)
            },
            Ops::LogicalNot => {
                let operand_node = match_unreachable!(Some(ChildrenType::Unary(operand_node)) = node.children, operand_node);

                let target = target.unwrap_or_else(|| Tn { id: irid_gen.next_tn(), data_type: node.data_type });

                let operand = generate_node(*operand_node, None, irid_gen, ir_code, ir_scope, symbol_table, source).expect("Expected an expression");

                ir_code.push(IRNode::LogicalNot {
                    target: target.clone(),
                    operand: IRValue::Tn(operand),
                });

                Some(target)
            },
            Ops::BitwiseNot => {
                let operand_node = match_unreachable!(Some(ChildrenType::Unary(operand_node)) = node.children, operand_node);

                let target = target.unwrap_or_else(|| Tn { id: irid_gen.next_tn(), data_type: node.data_type });

                let operand = generate_node(*operand_node, None, irid_gen, ir_code, ir_scope, symbol_table, source).expect("Expected an expression");

                ir_code.push(IRNode::BitNot {
                    target: target.clone(),
                    operand: IRValue::Tn(operand),
                });

                Some(target)
            },
            Ops::LogicalAnd => {
                let (l_node, r_node) = match_unreachable!(Some(ChildrenType::Binary(l_node, r_node)) = node.children, (l_node, r_node));

                let target = target.unwrap_or_else(|| Tn { id: irid_gen.next_tn(), data_type: node.data_type });

                let l_value = generate_node(*l_node, None, irid_gen, ir_code, ir_scope, symbol_table, source).expect("Expected an expression");
                let r_value = generate_node(*r_node, None, irid_gen, ir_code, ir_scope, symbol_table, source).expect("Expected an expression");

                ir_code.push(IRNode::LogicalAnd {
                    target: target.clone(),
                    left: IRValue::Tn(l_value),
                    right: IRValue::Tn(r_value),
                });

                Some(target)
            },
            Ops::LogicalOr => {
                let (l_node, r_node) = match_unreachable!(Some(ChildrenType::Binary(l_node, r_node)) = node.children, (l_node, r_node));

                let target = target.unwrap_or_else(|| Tn { id: irid_gen.next_tn(), data_type: node.data_type });

                let l_value = generate_node(*l_node, None, irid_gen, ir_code, ir_scope, symbol_table, source).expect("Expected an expression");
                let r_value = generate_node(*r_node, None, irid_gen, ir_code, ir_scope, symbol_table, source).expect("Expected an expression");

                ir_code.push(IRNode::LogicalOr {
                    target: target.clone(),
                    left: IRValue::Tn(l_value),
                    right: IRValue::Tn(r_value),
                });

                Some(target)
            },
            Ops::BitShiftLeft => {
                let (l_node, r_node) = match_unreachable!(Some(ChildrenType::Binary(l_node, r_node)) = node.children, (l_node, r_node));

                let target = target.unwrap_or_else(|| Tn { id: irid_gen.next_tn(), data_type: node.data_type });

                let l_value = generate_node(*l_node, None, irid_gen, ir_code, ir_scope, symbol_table, source).expect("Expected an expression");
                let r_value = generate_node(*r_node, None, irid_gen, ir_code, ir_scope, symbol_table, source).expect("Expected an expression");

                ir_code.push(IRNode::BitShiftLeft {
                    target: target.clone(),
                    left: IRValue::Tn(l_value),
                    right: IRValue::Tn(r_value),
                });

                Some(target)
            },
            Ops::BitShiftRight => {
                let (l_node, r_node) = match_unreachable!(Some(ChildrenType::Binary(l_node, r_node)) = node.children, (l_node, r_node));

                let target = target.unwrap_or_else(|| Tn { id: irid_gen.next_tn(), data_type: node.data_type });

                let l_value = generate_node(*l_node, None, irid_gen, ir_code, ir_scope, symbol_table, source).expect("Expected an expression");
                let r_value = generate_node(*r_node, None, irid_gen, ir_code, ir_scope, symbol_table, source).expect("Expected an expression");

                ir_code.push(IRNode::BitShiftRight {
                    target: target.clone(),
                    left: IRValue::Tn(l_value),
                    right: IRValue::Tn(r_value),
                });

                Some(target)
            },
            Ops::BitwiseOr => {
                let (l_node, r_node) = match_unreachable!(Some(ChildrenType::Binary(l_node, r_node)) = node.children, (l_node, r_node));

                let target = target.unwrap_or_else(|| Tn { id: irid_gen.next_tn(), data_type: node.data_type });

                let l_value = generate_node(*l_node, None, irid_gen, ir_code, ir_scope, symbol_table, source).expect("Expected an expression");
                let r_value = generate_node(*r_node, None, irid_gen, ir_code, ir_scope, symbol_table, source).expect("Expected an expression");

                ir_code.push(IRNode::BitOr {
                    target: target.clone(),
                    left: IRValue::Tn(l_value),
                    right: IRValue::Tn(r_value),
                });

                Some(target)
            },
            Ops::BitwiseAnd => {
                let (l_node, r_node) = match_unreachable!(Some(ChildrenType::Binary(l_node, r_node)) = node.children, (l_node, r_node));

                let target = target.unwrap_or_else(|| Tn { id: irid_gen.next_tn(), data_type: node.data_type });

                let l_value = generate_node(*l_node, None, irid_gen, ir_code, ir_scope, symbol_table, source).expect("Expected an expression");
                let r_value = generate_node(*r_node, None, irid_gen, ir_code, ir_scope, symbol_table, source).expect("Expected an expression");

                ir_code.push(IRNode::BitAnd {
                    target: target.clone(),
                    left: IRValue::Tn(l_value),
                    right: IRValue::Tn(r_value),
                });

                Some(target)
            },
            Ops::BitwiseXor => {
                let (l_node, r_node) = match_unreachable!(Some(ChildrenType::Binary(l_node, r_node)) = node.children, (l_node, r_node));

                let target = target.unwrap_or_else(|| Tn { id: irid_gen.next_tn(), data_type: node.data_type });

                let l_value = generate_node(*l_node, None, irid_gen, ir_code, ir_scope, symbol_table, source).expect("Expected an expression");
                let r_value = generate_node(*r_node, None, irid_gen, ir_code, ir_scope, symbol_table, source).expect("Expected an expression");

                ir_code.push(IRNode::BitXor {
                    target: target.clone(),
                    left: IRValue::Tn(l_value),
                    right: IRValue::Tn(r_value),
                });

                Some(target)
            },
            Ops::ArrayIndexOpen => todo!(),
            Ops::Break => todo!(),
            Ops::Continue => todo!(),
        },
        TokenKind::Value(value) => match value {
            Value::Literal { value } => {
                let target = target.unwrap_or_else(|| Tn { id: irid_gen.next_tn(), data_type: node.data_type });

                ir_code.push(IRNode::Assign {
                    target: target.clone(),
                    source: IRValue::Const(value),
                });

                Some(target)
            },
            Value::Symbol { name, scope_discriminant } => {
                
                let target = if let Some(tn) = ir_code.scope_table.get_tn(name, scope_discriminant, ir_scope) {
                    tn
                } else {
                    let tn = Tn { id: irid_gen.next_tn(), data_type: node.data_type };
                    ir_code.scope_table.map_symbol(name, tn.clone(), ir_scope);
                    tn
                };
                
                // Symbols don't add any operation to the ir code, they are just mapped to a Tn

                Some(target)
            },
        },
        TokenKind::As => todo!(),
        TokenKind::If => todo!(),
        TokenKind::While => todo!(),
        TokenKind::Loop => {
            let block = match_unreachable!(Some(ChildrenType::ParsedBlock(block)) = node.children, block);

            let size = symbol_table.scope_size(block.scope_id);

            ir_code.push(
                IRNode::PushScope { bytes: size }
            );

            // TODO: generate also an end label to jump to after the loop in case of breaks (and pass it around somehow)
            let start = irid_gen.next_label();

            // Put the label inside the scope bounds to avoid push-popping the scope for every iteration
            ir_code.push(
                IRNode::Label { label: start.clone() }
            );

            let inner_ir_scope = ir_code.scope_table.add_scope(Some(ir_scope));

            generate_block(block, None, irid_gen, ir_code, inner_ir_scope, symbol_table, source);

            ir_code.push(
                IRNode::Jump { target: start }
            );

            ir_code.push(
                IRNode::PopScope { bytes: size }
            );

            None
        },
        TokenKind::ArrayOpen => todo!(),
        TokenKind::ScopeOpen => {

            let block = match_unreachable!(Some(ChildrenType::ParsedBlock(block)) = node.children, block);

            // First add the PushScope instruction, before the block code
            let size = symbol_table.scope_size(block.scope_id);
            ir_code.push(
                IRNode::PushScope { bytes: size }
            );

            let inner_ir_scope = ir_code.scope_table.add_scope(Some(ir_scope));

            generate_block(block, target, irid_gen, ir_code, inner_ir_scope, symbol_table, source);

            // Lastly, add the PopScope instruction, after the block code
            ir_code.push(
                IRNode::PopScope { bytes: size }
            );

            None
        },

        _ => unreachable!("{:?} is not exprected.", node.item.value)
    }
}


fn generate_block(mut block: ScopeBlock, target: Option<Tn>, irid_gen: &mut IRIDGenerator, ir_code: &mut FunctionIR, ir_scope: IRScopeID, symbol_table: &mut SymbolTable, source: &SourceCode) -> Option<Tn> {

    for statement in block.statements.drain(0..block.statements.len() - 1) {

        generate_node(statement, None, irid_gen, ir_code, ir_scope, symbol_table, source);

    }

    let last_statement = block.statements.pop().unwrap();
    generate_node(last_statement, target, irid_gen, ir_code, ir_scope, symbol_table, source);
    

    todo!()
}


fn generate_function(function: Function, symbol_table: &mut SymbolTable, source: &SourceCode) -> FunctionIR {

    let mut irid_gen = IRIDGenerator::new();
    let mut ir_code = FunctionIR::new();

    // Create the top-level function scope
    let ir_scope = ir_code.scope_table.add_scope(None);

    // Create a Tn for the return value, if it isn't Void. Non-void return statements will assign to this Tn
    let return_type = function.return_type();
    let return_tn = if *return_type != DataType::Void {
        let return_tn = Tn { id: irid_gen.next_tn(), data_type: return_type };
        ir_code.scope_table.scopes[ir_scope.0].return_tn = Some(return_tn.clone());
        Some(return_tn)
    } else {
        None
    };

    generate_block(function.code, return_tn, &mut irid_gen,&mut ir_code, ir_scope, symbol_table, source);

    ir_code
}


/// Generate ir code from the given functions
pub fn generate(functions: Vec<Function>, symbol_table: &mut SymbolTable, source: &SourceCode) {

    let mut ir_code = FunctionIR::new();

    for function in functions {

        let irc = generate_function(function, symbol_table, source);

        //ir_code.extend(irc);

    }

    todo!()
}

