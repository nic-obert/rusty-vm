use std::fmt::{Debug, Display};
use std::rc::Rc;
use std::collections::HashMap;

use crate::match_unreachable;
use crate::symbol_table::{FunctionUUID, ScopeDiscriminant, ScopeID, SymbolTable};
use crate::function_parser::Function;
use crate::data_types::{DataType, LiteralValue, Number};
use crate::token::{TokenKind, Value};
use crate::operations::Ops;
use crate::token_tree::{ChildrenType, ScopeBlock, TokenNode};


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


/// Represents a temporary variable
#[derive(Clone)]
pub struct Tn {
    pub id: TnID,
    pub data_type: Rc<DataType>,
}

impl Display for Tn {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "T{}", self.id.0)
    }
}


#[derive(Clone, Copy)]
pub struct Label(pub LabelID);

impl Display for Label {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "L{}", self.0.0)
    }

}


/// Represents an operand of ir operations
pub enum IRValue {

    Tn (Tn),
    Const (LiteralValue),

}

impl Debug for IRValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    
    }
}

impl Display for IRValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IRValue::Tn(tn) => write!(f, "{}", tn),
            IRValue::Const(value) => write!(f, "{}", value),
        }
    }
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

/// Stores information about function scopes in the IR
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
        self.scopes[ir_scope.0].symbols.entry(name.to_string()).or_default().push(tn);
    }

}


/// Represents an intermediate code operation
pub enum IROperator {

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

    /// Copies the raw bits from `source` into `target`.
    /// Assumes that `target` is either the same size or larget than `source`.
    Copy { target: Tn, source: IRValue },
    
    Jump { target: Label },
    JumpIf { condition: Tn, target: Label },
    JumpIfNot { condition: Tn, target: Label },
    Label { label: Label },

    Call { return_target: Option<Tn>, return_label: Label, callable: Tn, args: Vec<IRValue> },
    Return,

    PushScope { bytes: usize },
    PopScope { bytes: usize },

    Nop,

}

impl Display for IROperator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IROperator::Add { target, left, right } => write!(f, "{} = {} + {}", target, left, right),
            IROperator::Sub { target, left, right } => write!(f, "{} = {} - {}", target, left, right),
            IROperator::Mul { target, left, right } => write!(f, "{} = {} * {}", target, left, right),
            IROperator::Div { target, left, right } => write!(f, "{} = {} / {}", target, left, right),
            IROperator::Mod { target, left, right } => write!(f, "{} = {} % {}", target, left, right),
            IROperator::Assign { target, source } => write!(f, "{} = {}", target, source),
            IROperator::Deref { target, ref_ } => write!(f, "{} = *{}", target, ref_),
            IROperator::Ref { target, ref_ } => write!(f, "{} = &{}", target, ref_),
            IROperator::Greater { target, left, right } => write!(f, "{} = {} > {}", target, left, right),
            IROperator::Less { target, left, right } => write!(f, "{} = {} < {}", target, left, right),
            IROperator::GreaterEqual { target, left, right } => write!(f, "{} = {} >= {}", target, left, right),
            IROperator::LessEqual { target, left, right } => write!(f, "{} = {} <= {}", target, left, right),
            IROperator::Equal { target, left, right } => write!(f, "{} = {} == {}", target, left, right),
            IROperator::NotEqual { target, left, right } => write!(f, "{} = {} != {}", target, left, right),
            IROperator::LogicalAnd { target, left, right } => write!(f, "{} = {} && {}", target, left, right),
            IROperator::LogicalOr { target, left, right } => write!(f, "{} = {} || {}", target, left, right),
            IROperator::LogicalNot { target, operand } => write!(f, "{} = !{}", target, operand),
            IROperator::BitShiftLeft { target, left, right } => write!(f, "{} = {} << {}", target, left, right),
            IROperator::BitShiftRight { target, left, right } => write!(f, "{} = {} >> {}", target, left, right),
            IROperator::BitNot { target, operand } => write!(f, "{} = ~{}", target, operand),
            IROperator::BitAnd { target, left, right } => write!(f, "{} = {} & {}", target, left, right),
            IROperator::BitOr { target, left, right } => write!(f, "{} = {} | {}", target, left, right),
            IROperator::BitXor { target, left, right } => write!(f, "{} = {} ^ {}", target, left, right),
            IROperator::Jump { target } => write!(f, "jump {}", target),
            IROperator::JumpIf { condition, target } => write!(f, "jumpif {} {}", condition, target),
            IROperator::JumpIfNot { condition, target } => write!(f, "jumpifnot {} {}", condition, target),
            IROperator::Label { label } => write!(f, "{}:", label),
            IROperator::Call { return_target, return_label, callable, args } => write!(f, "{}call {callable} {:?} (return: {return_label})", if let Some(target) = return_target { format!("{target} = ") } else { "".to_string() }, args),
            IROperator::Return => write!(f, "return"),
            IROperator::PushScope { bytes } => write!(f, "pushscope {}", bytes),
            IROperator::PopScope { bytes } => write!(f, "popscope {}", bytes),
            IROperator::Nop => write!(f, "nop"),
            IROperator::Copy { target, source } => write!(f, "copy {} -> {}", source, target),
        }
    }
}


pub struct FunctionIR<'a> {
    pub name: &'a str,
    pub code: Vec<IROperator>,
    pub scope_table: ScopeTable,
    /// The first scope of the function in the symbol table.
    // This is used to calculate how many bytes to pop upon returning from the function.
    pub st_first_scope: ScopeID,
    pub function_labels: FunctionLabels,
}

impl FunctionIR<'_> {

    pub fn new<'a>(name: &'a str, first_scope: ScopeID, irid_gen: &mut IRIDGenerator) -> FunctionIR<'a> {
        FunctionIR {
            name,
            code: Vec::new(),
            scope_table: ScopeTable::new(),
            st_first_scope: first_scope,
            function_labels: FunctionLabels {
                start: irid_gen.next_label(),
                exit: irid_gen.next_label(),
            },
        }
    }

    pub fn push(&mut self, node: IROperator) {
        self.code.push(node);
    }

}

impl Display for FunctionIR<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "fn {} {{", self.name)?;
        for op in &self.code {
            writeln!(f, "    {}", op)?;
        }
        writeln!(f, "}}")
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
fn generate_node(node: TokenNode, target: Option<Tn>, outer_loop: Option<&LoopLabels>, irid_gen: &mut IRIDGenerator, ir_function: &mut FunctionIR, ir_scope: IRScopeID, st_scope: ScopeID, symbol_table: &mut SymbolTable) -> Option<Tn> {
    
    match node.item.value {

        TokenKind::Op(op) => match op {
            Ops::Add => {
                let (l_node, r_node) = match_unreachable!(Some(ChildrenType::Binary(l_node, r_node)) = node.children, (l_node, r_node));

                let target = target.unwrap_or_else(|| Tn { id: irid_gen.next_tn(), data_type: node.data_type });

                let l_value = generate_node(*l_node, None, outer_loop, irid_gen, ir_function, ir_scope, st_scope, symbol_table).expect("Expected an expression");
                let r_value = generate_node(*r_node, None, outer_loop, irid_gen, ir_function, ir_scope, st_scope, symbol_table).expect("Expected an expression");

                ir_function.push(IROperator::Add {
                    target: target.clone(),
                    left: IRValue::Tn(l_value),
                    right: IRValue::Tn(r_value),
                });

                Some(target)
            },
            Ops::Sub => {
                let (l_node, r_node) = match_unreachable!(Some(ChildrenType::Binary(l_node, r_node)) = node.children, (l_node, r_node));

                let target = target.unwrap_or_else(|| Tn { id: irid_gen.next_tn(), data_type: node.data_type });

                let l_value = generate_node(*l_node, None, outer_loop, irid_gen, ir_function, ir_scope, st_scope, symbol_table).expect("Expected an expression");
                let r_value = generate_node(*r_node, None, outer_loop, irid_gen, ir_function, ir_scope, st_scope, symbol_table).expect("Expected an expression");

                ir_function.push(IROperator::Sub {
                    target: target.clone(),
                    left: IRValue::Tn(l_value),
                    right: IRValue::Tn(r_value),
                });

                Some(target)
            },
            Ops::Mul => {
                let (l_node, r_node) = match_unreachable!(Some(ChildrenType::Binary(l_node, r_node)) = node.children, (l_node, r_node));

                let target = target.unwrap_or_else(|| Tn { id: irid_gen.next_tn(), data_type: node.data_type });

                let l_value = generate_node(*l_node, None, outer_loop, irid_gen, ir_function, ir_scope, st_scope, symbol_table).expect("Expected an expression");
                let r_value = generate_node(*r_node, None, outer_loop, irid_gen, ir_function, ir_scope, st_scope, symbol_table).expect("Expected an expression");

                ir_function.push(IROperator::Mul {
                    target: target.clone(),
                    left: IRValue::Tn(l_value),
                    right: IRValue::Tn(r_value),
                });

                Some(target)
            },
            Ops::Div => {
                let (l_node, r_node) = match_unreachable!(Some(ChildrenType::Binary(l_node, r_node)) = node.children, (l_node, r_node));

                let target = target.unwrap_or_else(|| Tn { id: irid_gen.next_tn(), data_type: node.data_type });

                let l_value = generate_node(*l_node, None, outer_loop, irid_gen, ir_function, ir_scope, st_scope, symbol_table).expect("Expected an expression");
                let r_value = generate_node(*r_node, None, outer_loop, irid_gen, ir_function, ir_scope, st_scope, symbol_table).expect("Expected an expression");

                ir_function.push(IROperator::Div {
                    target: target.clone(),
                    left: IRValue::Tn(l_value),
                    right: IRValue::Tn(r_value),
                });

                Some(target)
            },
            Ops::Mod => {
                let (l_node, r_node) = match_unreachable!(Some(ChildrenType::Binary(l_node, r_node)) = node.children, (l_node, r_node));

                let target = target.unwrap_or_else(|| Tn { id: irid_gen.next_tn(), data_type: node.data_type });

                let l_value = generate_node(*l_node, None, outer_loop, irid_gen, ir_function, ir_scope, st_scope, symbol_table).expect("Expected an expression");
                let r_value = generate_node(*r_node, None, outer_loop, irid_gen, ir_function, ir_scope, st_scope, symbol_table).expect("Expected an expression");

                ir_function.push(IROperator::Mod {
                    target: target.clone(),
                    left: IRValue::Tn(l_value),
                    right: IRValue::Tn(r_value),
                });

                Some(target)
            },
            Ops::Assign => {
                let (l_node, r_node) = match_unreachable!(Some(ChildrenType::Binary(l_node, r_node)) = node.children, (l_node, r_node));
                
                let target = generate_node(*l_node, None, outer_loop, irid_gen, ir_function, ir_scope, st_scope, symbol_table).expect("Expected an assignable");
                generate_node(*r_node, Some(target), outer_loop, irid_gen, ir_function, ir_scope, st_scope, symbol_table);

                // Adding an Assign node is superfluous since genetate_node for the source node has already assigned the value to the target

                None
            },
            Ops::Deref { mutable: _ } => {
                let ref_node = match_unreachable!(Some(ChildrenType::Unary(ref_node)) = node.children, ref_node);

                let target = target.unwrap_or_else(|| Tn { id: irid_gen.next_tn(), data_type: node.data_type });

                let ref_ = generate_node(*ref_node, None, outer_loop, irid_gen, ir_function, ir_scope, st_scope, symbol_table).expect("Expected a reference");

                ir_function.push(IROperator::Deref {
                    target: target.clone(),
                    ref_: IRValue::Tn(ref_),
                });

                Some(target)
            },
            Ops::Ref { mutable: _ } => {
                let ref_node = match_unreachable!(Some(ChildrenType::Unary(ref_node)) = node.children, ref_node);

                let target = target.unwrap_or_else(|| Tn { id: irid_gen.next_tn(), data_type: node.data_type });

                let ref_ = generate_node(*ref_node, None, outer_loop, irid_gen, ir_function, ir_scope, st_scope, symbol_table).expect("Expected a reference");

                ir_function.push(IROperator::Ref {
                    target: target.clone(),
                    ref_: IRValue::Tn(ref_),
                });

                Some(target)
            },
            Ops::FunctionCallOpen => {
                /*
                    Tcallable = <callable>
                    [Targ-n = <arg-n>...]
                    [Tresult =] call Tcallable [Targ-n...] return: Lreturn
                    Lreturn:
                */
                let (callable, args) = match_unreachable!(Some(ChildrenType::Call { callable, args }) = node.children, (callable, args));

                let return_target = if !matches!(*node.data_type, DataType::Void) {
                    target.or_else(|| Some(Tn { id: irid_gen.next_tn(), data_type: node.data_type }))
                } else {
                    None
                };

                let callable = generate_node(*callable, None, outer_loop, irid_gen, ir_function, ir_scope, st_scope, symbol_table).expect("Expected a callable expression");
                let args: Vec<IRValue> = args.into_iter().map(
                        |arg| generate_node(arg, None, outer_loop, irid_gen, ir_function, ir_scope, st_scope, symbol_table).expect("Argument is expected to be an expression")
                    ).map(IRValue::Tn)
                    .collect();

                let return_label = irid_gen.next_label();
                
                ir_function.push(IROperator::Call {
                    return_target: return_target.clone(),
                    return_label,
                    callable,
                    args,
                });

                ir_function.push(IROperator::Label { 
                    label: return_label
                });

                return_target
            },
            Ops::Return => {
                match node.children {
                    Some(ChildrenType::Unary(value_node)) => {
                        let return_tn = ir_function.scope_table.return_tn(ir_scope).expect("The function returns a value, but no return Tn was supplied");
                        generate_node(*value_node, Some(return_tn), outer_loop, irid_gen, ir_function, ir_scope, st_scope, symbol_table).expect("Expected a value");
                    },
                    None => { 
                        // No return value is provided and the function should return void
                        assert!(ir_function.scope_table.return_tn(ir_scope).is_none());
                    },
                    _ => unreachable!("Return node has more than one child")
                }

                // Jump to the function's exit label, which will take care of popping the stack and returning to the caller
                ir_function.push(IROperator::Jump { target: ir_function.function_labels.exit });

                None
            },
            Ops::Equal => {
                let (l_node, r_node) = match_unreachable!(Some(ChildrenType::Binary(l_node, r_node)) = node.children, (l_node, r_node));

                let target = target.unwrap_or_else(|| Tn { id: irid_gen.next_tn(), data_type: node.data_type });

                let l_value = generate_node(*l_node, None, outer_loop, irid_gen, ir_function, ir_scope, st_scope, symbol_table).expect("Expected an expression");
                let r_value = generate_node(*r_node, None, outer_loop, irid_gen, ir_function, ir_scope, st_scope, symbol_table).expect("Expected an expression");

                ir_function.push(IROperator::Equal {
                    target: target.clone(),
                    left: IRValue::Tn(l_value),
                    right: IRValue::Tn(r_value),
                });

                Some(target)
            },
            Ops::NotEqual => {
                let (l_node, r_node) = match_unreachable!(Some(ChildrenType::Binary(l_node, r_node)) = node.children, (l_node, r_node));

                let target = target.unwrap_or_else(|| Tn { id: irid_gen.next_tn(), data_type: node.data_type });

                let l_value = generate_node(*l_node, None, outer_loop, irid_gen, ir_function, ir_scope, st_scope, symbol_table).expect("Expected an expression");
                let r_value = generate_node(*r_node, None, outer_loop, irid_gen, ir_function, ir_scope, st_scope, symbol_table).expect("Expected an expression");

                ir_function.push(IROperator::NotEqual {
                    target: target.clone(),
                    left: IRValue::Tn(l_value),
                    right: IRValue::Tn(r_value),
                });

                Some(target)
            },
            Ops::Greater => {
                let (l_node, r_node) = match_unreachable!(Some(ChildrenType::Binary(l_node, r_node)) = node.children, (l_node, r_node));

                let target = target.unwrap_or_else(|| Tn { id: irid_gen.next_tn(), data_type: node.data_type });

                let l_value = generate_node(*l_node, None, outer_loop, irid_gen, ir_function, ir_scope, st_scope, symbol_table).expect("Expected an expression");
                let r_value = generate_node(*r_node, None, outer_loop, irid_gen, ir_function, ir_scope, st_scope, symbol_table).expect("Expected an expression");

                ir_function.push(IROperator::Greater {
                    target: target.clone(),
                    left: IRValue::Tn(l_value),
                    right: IRValue::Tn(r_value),
                });

                Some(target)
            },
            Ops::Less => {
                let (l_node, r_node) = match_unreachable!(Some(ChildrenType::Binary(l_node, r_node)) = node.children, (l_node, r_node));

                let target = target.unwrap_or_else(|| Tn { id: irid_gen.next_tn(), data_type: node.data_type });

                let l_value = generate_node(*l_node, None, outer_loop, irid_gen, ir_function, ir_scope, st_scope, symbol_table).expect("Expected an expression");
                let r_value = generate_node(*r_node, None, outer_loop, irid_gen, ir_function, ir_scope, st_scope, symbol_table).expect("Expected an expression");

                ir_function.push(IROperator::Less {
                    target: target.clone(),
                    left: IRValue::Tn(l_value),
                    right: IRValue::Tn(r_value),
                });

                Some(target)
            },
            Ops::GreaterEqual => {
                let (l_node, r_node) = match_unreachable!(Some(ChildrenType::Binary(l_node, r_node)) = node.children, (l_node, r_node));

                let target = target.unwrap_or_else(|| Tn { id: irid_gen.next_tn(), data_type: node.data_type });

                let l_value = generate_node(*l_node, None, outer_loop, irid_gen, ir_function, ir_scope, st_scope, symbol_table).expect("Expected an expression");
                let r_value = generate_node(*r_node, None, outer_loop, irid_gen, ir_function, ir_scope, st_scope, symbol_table).expect("Expected an expression");

                ir_function.push(IROperator::GreaterEqual {
                    target: target.clone(),
                    left: IRValue::Tn(l_value),
                    right: IRValue::Tn(r_value),
                });

                Some(target)
            },
            Ops::LessEqual => {
                let (l_node, r_node) = match_unreachable!(Some(ChildrenType::Binary(l_node, r_node)) = node.children, (l_node, r_node));

                let target = target.unwrap_or_else(|| Tn { id: irid_gen.next_tn(), data_type: node.data_type });

                let l_value = generate_node(*l_node, None, outer_loop, irid_gen, ir_function, ir_scope, st_scope, symbol_table).expect("Expected an expression");
                let r_value = generate_node(*r_node, None, outer_loop, irid_gen, ir_function, ir_scope, st_scope, symbol_table).expect("Expected an expression");

                ir_function.push(IROperator::LessEqual {
                    target: target.clone(),
                    left: IRValue::Tn(l_value),
                    right: IRValue::Tn(r_value),
                });

                Some(target)
            },
            Ops::LogicalNot => {
                let operand_node = match_unreachable!(Some(ChildrenType::Unary(operand_node)) = node.children, operand_node);

                let target = target.unwrap_or_else(|| Tn { id: irid_gen.next_tn(), data_type: node.data_type });

                let operand = generate_node(*operand_node, None, outer_loop, irid_gen, ir_function, ir_scope, st_scope, symbol_table).expect("Expected an expression");

                ir_function.push(IROperator::LogicalNot {
                    target: target.clone(),
                    operand: IRValue::Tn(operand),
                });

                Some(target)
            },
            Ops::BitwiseNot => {
                let operand_node = match_unreachable!(Some(ChildrenType::Unary(operand_node)) = node.children, operand_node);

                let target = target.unwrap_or_else(|| Tn { id: irid_gen.next_tn(), data_type: node.data_type });

                let operand = generate_node(*operand_node, None, outer_loop, irid_gen, ir_function, ir_scope, st_scope, symbol_table).expect("Expected an expression");

                ir_function.push(IROperator::BitNot {
                    target: target.clone(),
                    operand: IRValue::Tn(operand),
                });

                Some(target)
            },
            Ops::LogicalAnd => {
                let (l_node, r_node) = match_unreachable!(Some(ChildrenType::Binary(l_node, r_node)) = node.children, (l_node, r_node));

                let target = target.unwrap_or_else(|| Tn { id: irid_gen.next_tn(), data_type: node.data_type });

                let l_value = generate_node(*l_node, None, outer_loop, irid_gen, ir_function, ir_scope, st_scope, symbol_table).expect("Expected an expression");
                let r_value = generate_node(*r_node, None, outer_loop, irid_gen, ir_function, ir_scope, st_scope, symbol_table).expect("Expected an expression");

                ir_function.push(IROperator::LogicalAnd {
                    target: target.clone(),
                    left: IRValue::Tn(l_value),
                    right: IRValue::Tn(r_value),
                });

                Some(target)
            },
            Ops::LogicalOr => {
                let (l_node, r_node) = match_unreachable!(Some(ChildrenType::Binary(l_node, r_node)) = node.children, (l_node, r_node));

                let target = target.unwrap_or_else(|| Tn { id: irid_gen.next_tn(), data_type: node.data_type });

                let l_value = generate_node(*l_node, None, outer_loop, irid_gen, ir_function, ir_scope, st_scope, symbol_table).expect("Expected an expression");
                let r_value = generate_node(*r_node, None, outer_loop, irid_gen, ir_function, ir_scope, st_scope, symbol_table).expect("Expected an expression");

                ir_function.push(IROperator::LogicalOr {
                    target: target.clone(),
                    left: IRValue::Tn(l_value),
                    right: IRValue::Tn(r_value),
                });

                Some(target)
            },
            Ops::BitShiftLeft => {
                let (l_node, r_node) = match_unreachable!(Some(ChildrenType::Binary(l_node, r_node)) = node.children, (l_node, r_node));

                let target = target.unwrap_or_else(|| Tn { id: irid_gen.next_tn(), data_type: node.data_type });

                let l_value = generate_node(*l_node, None, outer_loop, irid_gen, ir_function, ir_scope, st_scope, symbol_table).expect("Expected an expression");
                let r_value = generate_node(*r_node, None, outer_loop, irid_gen, ir_function, ir_scope, st_scope, symbol_table).expect("Expected an expression");

                ir_function.push(IROperator::BitShiftLeft {
                    target: target.clone(),
                    left: IRValue::Tn(l_value),
                    right: IRValue::Tn(r_value),
                });

                Some(target)
            },
            Ops::BitShiftRight => {
                let (l_node, r_node) = match_unreachable!(Some(ChildrenType::Binary(l_node, r_node)) = node.children, (l_node, r_node));

                let target = target.unwrap_or_else(|| Tn { id: irid_gen.next_tn(), data_type: node.data_type });

                let l_value = generate_node(*l_node, None, outer_loop, irid_gen, ir_function, ir_scope, st_scope, symbol_table).expect("Expected an expression");
                let r_value = generate_node(*r_node, None, outer_loop, irid_gen, ir_function, ir_scope, st_scope, symbol_table).expect("Expected an expression");

                ir_function.push(IROperator::BitShiftRight {
                    target: target.clone(),
                    left: IRValue::Tn(l_value),
                    right: IRValue::Tn(r_value),
                });

                Some(target)
            },
            Ops::BitwiseOr => {
                let (l_node, r_node) = match_unreachable!(Some(ChildrenType::Binary(l_node, r_node)) = node.children, (l_node, r_node));

                let target = target.unwrap_or_else(|| Tn { id: irid_gen.next_tn(), data_type: node.data_type });

                let l_value = generate_node(*l_node, None, outer_loop, irid_gen, ir_function, ir_scope, st_scope, symbol_table).expect("Expected an expression");
                let r_value = generate_node(*r_node, None, outer_loop, irid_gen, ir_function, ir_scope, st_scope, symbol_table).expect("Expected an expression");

                ir_function.push(IROperator::BitOr {
                    target: target.clone(),
                    left: IRValue::Tn(l_value),
                    right: IRValue::Tn(r_value),
                });

                Some(target)
            },
            Ops::BitwiseAnd => {
                let (l_node, r_node) = match_unreachable!(Some(ChildrenType::Binary(l_node, r_node)) = node.children, (l_node, r_node));

                let target = target.unwrap_or_else(|| Tn { id: irid_gen.next_tn(), data_type: node.data_type });

                let l_value = generate_node(*l_node, None, outer_loop, irid_gen, ir_function, ir_scope, st_scope, symbol_table).expect("Expected an expression");
                let r_value = generate_node(*r_node, None, outer_loop, irid_gen, ir_function, ir_scope, st_scope, symbol_table).expect("Expected an expression");

                ir_function.push(IROperator::BitAnd {
                    target: target.clone(),
                    left: IRValue::Tn(l_value),
                    right: IRValue::Tn(r_value),
                });

                Some(target)
            },
            Ops::BitwiseXor => {
                let (l_node, r_node) = match_unreachable!(Some(ChildrenType::Binary(l_node, r_node)) = node.children, (l_node, r_node));

                let target = target.unwrap_or_else(|| Tn { id: irid_gen.next_tn(), data_type: node.data_type });

                let l_value = generate_node(*l_node, None, outer_loop, irid_gen, ir_function, ir_scope, st_scope, symbol_table).expect("Expected an expression");
                let r_value = generate_node(*r_node, None, outer_loop, irid_gen, ir_function, ir_scope, st_scope, symbol_table).expect("Expected an expression");

                ir_function.push(IROperator::BitXor {
                    target: target.clone(),
                    left: IRValue::Tn(l_value),
                    right: IRValue::Tn(r_value),
                });

                Some(target)
            },
            Ops::ArrayIndexOpen => {
                let (array_node, index_node) = match_unreachable!(Some(ChildrenType::Binary(array, index)) = node.children, (array, index));

                let element_type = match_unreachable!(DataType::Array(element_type) = array_node.data_type.as_ref(), element_type.clone());
                let element_size = element_type.static_size();

                let array_addr_tn = generate_node(*array_node, None, outer_loop, irid_gen, ir_function, ir_scope, st_scope, symbol_table).expect("Expected an array");
                let index_tn = generate_node(*index_node, None, outer_loop, irid_gen, ir_function, ir_scope, st_scope, symbol_table).expect("Expected an index");

                let offset_tn = Tn { id: irid_gen.next_tn(), data_type: DataType::Usize.into() };
                ir_function.code.push(IROperator::Mul {
                    target: offset_tn.clone(),
                    left: IRValue::Const(LiteralValue::Numeric(Number::Uint(element_size as u64))),
                    right: IRValue::Tn(index_tn),
                });

                let element_addr_tn = Tn { id: irid_gen.next_tn(), data_type: DataType::Ref { target: element_type.clone(), mutable: true }.into() };
                ir_function.code.push(IROperator::Add {
                    target: element_addr_tn.clone(),
                    left: IRValue::Tn(array_addr_tn),
                    right: IRValue::Tn(offset_tn),
                });

                let target = target.unwrap_or_else(|| Tn { id: irid_gen.next_tn(), data_type: element_type });

                ir_function.code.push(IROperator::Deref { 
                    target: target.clone(),
                    ref_: IRValue::Tn(element_addr_tn),
                });

                Some(target)
            },
            Ops::Break => {
                let loop_labels = outer_loop.expect("Break statement outside of a loop");
                ir_function.push(
                    IROperator::Jump { target: loop_labels.after }
                );
                None
            
            },
            Ops::Continue => {
                let loop_labels = outer_loop.expect("Continue statement outside of a loop");
                ir_function.push(
                    IROperator::Jump { target: loop_labels.start }
                );
                None
            },
        },
        TokenKind::Value(value) => match value {
            Value::Literal { value } => {
                let target = target.unwrap_or_else(|| Tn { id: irid_gen.next_tn(), data_type: node.data_type });

                ir_function.push(IROperator::Assign {
                    target: target.clone(),
                    source: IRValue::Const(value),
                });

                Some(target)
            },
            Value::Symbol { name, scope_discriminant } => {
                
                let target = if let Some(tn) = ir_function.scope_table.get_tn(name, scope_discriminant, ir_scope) {
                    tn
                } else {
                    let tn = Tn { id: irid_gen.next_tn(), data_type: node.data_type };
                    ir_function.scope_table.map_symbol(name, tn.clone(), ir_scope);
                    tn
                };
                
                // Symbols don't add any operation to the ir code, they are just mapped to a Tn

                Some(target)
            },
        },
        TokenKind::As => {
            // Just reinterpret the bits (drop excess bits or add padding if necessary)
            // Assume the conversion is possible, since the parser should have already checked that

            let mut target = target.unwrap_or_else(|| Tn { id: irid_gen.next_tn(), data_type: node.data_type });

            let (expr_node, target_type) = match_unreachable!(Some(ChildrenType::TypeCast { target_type, expr }) = node.children, (expr, target_type));
            
            let src_size = expr_node.data_type.static_size();
            let target_size = target_type.static_size();

            match src_size.cmp(&target_size) {
                std::cmp::Ordering::Less => {
                    // The source has less bits, so create a copy with padding
                    // Reading directly from the source would read garbage and writing would overwrite surrounding memory.
                    // Copying is cheap since type casting is only allowed on primitives, which are usually small.

                    let expr_tn = generate_node(*expr_node, None, outer_loop, irid_gen, ir_function, ir_scope, st_scope, symbol_table).expect("Expected an expression");

                    target.data_type = target_type;

                    ir_function.code.push(IROperator::Copy {
                        target: target.clone(),
                        source: IRValue::Tn(expr_tn),
                    });

                    Some(target)
                },
                std::cmp::Ordering::Equal |
                std::cmp::Ordering::Greater
                 => {
                    // No need to do anything, just reinterpret the bits as the new type.
                    // If the source and target have the same size, the bits are already in the correct format.
                    // If the source has more bits than the target, the excess bits are simply ignored.
                    
                    generate_node(*expr_node, Some(target.clone()), outer_loop, irid_gen, ir_function, ir_scope, st_scope, symbol_table).expect("Expected an expression");
                    
                    // Just change the type of the Tn
                    target.data_type = target_type;
                    Some(target)
                },
            }
        },
        TokenKind::If => {
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
            let (if_chain, else_block) = match_unreachable!(Some(ChildrenType::IfChain { if_chain, else_block }) = node.children, (if_chain, else_block));

            let mut next_if_block = irid_gen.next_label();
            let after_chain = irid_gen.next_label();

            for if_block in if_chain {
                
                let condition_tn = generate_node(if_block.condition, None, outer_loop, irid_gen, ir_function, ir_scope, st_scope, symbol_table).expect("Expected a condition");

                ir_function.push(
                    IROperator::JumpIfNot { condition: condition_tn, target: next_if_block }
                );

                let inner_ir_scope = ir_function.scope_table.add_scope(Some(ir_scope));
                generate_block(if_block.body, target.clone(), outer_loop, irid_gen, ir_function, inner_ir_scope, symbol_table);

                ir_function.push(
                    IROperator::Jump { target: after_chain }
                );

                // If there's no else block, this label coincides with the after_chain label. This is ok, though, since labels are no-ops.
                ir_function.push(
                    IROperator::Label { label: next_if_block }
                );
                
                // A redundant label is generated at the last iteration of the loop, but that's ok since this operation is cheap and labels don't have to be serial.
                next_if_block = irid_gen.next_label();
            }

            if let Some(else_block) = else_block {
                let inner_ir_scope = ir_function.scope_table.add_scope(Some(ir_scope));
                generate_block(else_block, target, outer_loop, irid_gen, ir_function, inner_ir_scope, symbol_table);
            }

            // Return None because the if-chain's return value is stored in the target Tn by the if-blocks
            None
        },
        TokenKind::While => {
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
            let (condition, body) = match_unreachable!(Some(ChildrenType::While { condition, body }) = node.children, (condition, body));
            
            let loop_labels = LoopLabels {
                start: irid_gen.next_label(),
                check: Some(irid_gen.next_label()),
                after: irid_gen.next_label(),
            };

            ir_function.push(
                IROperator::Jump { target: loop_labels.check.unwrap() }
            );

            ir_function.push(
                IROperator::Label { label: loop_labels.start }
            );

            let inner_ir_scope = ir_function.scope_table.add_scope(Some(ir_scope));
            generate_block(body, None, Some(&loop_labels), irid_gen, ir_function, inner_ir_scope, symbol_table);

            ir_function.push(
                IROperator::Label { label: loop_labels.check.unwrap() }
            );
            
            let condition_tn = generate_node(*condition, None, Some(&loop_labels), irid_gen, ir_function, ir_scope, st_scope, symbol_table).expect("Expected a condition");
       
            ir_function.code.push(
                IROperator::JumpIf { condition: condition_tn, target: loop_labels.start }
            );     
       
            ir_function.push(
                IROperator::Label { label: loop_labels.after }
            );

            None
        },
        TokenKind::Loop => {
            /*
                Lstart:
                    <body>
                jump Lstart
                Lafter:
            */
            let block = match_unreachable!(Some(ChildrenType::ParsedBlock(block)) = node.children, block);

            let loop_labels = LoopLabels {
                start: irid_gen.next_label(),
                check: None,
                after: irid_gen.next_label(),
            };

            // Put the label inside the scope bounds to avoid push-popping the scope for every iteration
            ir_function.push(
                IROperator::Label { label: loop_labels.start }
            );

            let inner_ir_scope = ir_function.scope_table.add_scope(Some(ir_scope));
            generate_block(block, None, Some(&loop_labels), irid_gen, ir_function, inner_ir_scope, symbol_table);

            ir_function.push(
                IROperator::Jump { target: loop_labels.start }
            );

            ir_function.push(
                IROperator::Label { label: loop_labels.after }
            );

            None
        },
        TokenKind::DoWhile => {
            /*
                Lstart:
                    <body>
                Lcheck:
                    Tcondition = <condition>
                    jumpif Tcondition Lstart
                Lafter:
            */
            let (condition, body) = match_unreachable!(Some(ChildrenType::While { condition, body }) = node.children, (condition, body));

            let loop_labels = LoopLabels {
                start: irid_gen.next_label(),
                check: Some(irid_gen.next_label()),
                after: irid_gen.next_label(),
            };

            ir_function.push(
                IROperator::Label { label: loop_labels.start }
            );

            let inner_ir_scope = ir_function.scope_table.add_scope(Some(ir_scope));
            generate_block(body, None, Some(&loop_labels), irid_gen, ir_function, inner_ir_scope, symbol_table);

            ir_function.push(
                IROperator::Label { label: loop_labels.check.unwrap() }
            );

            let condition_tn = generate_node(*condition, None, Some(&loop_labels), irid_gen, ir_function, ir_scope, st_scope, symbol_table).expect("Expected a condition");

            ir_function.code.push(
                IROperator::JumpIf { condition: condition_tn, target: loop_labels.start }
            );

            ir_function.push(
                IROperator::Label { label: loop_labels.after }
            );

            None
        }
        TokenKind::ArrayOpen => todo!(), // Should return a pointer to the array
        TokenKind::ScopeOpen => {

            let block = match_unreachable!(Some(ChildrenType::ParsedBlock(block)) = node.children, block);

            let inner_ir_scope = ir_function.scope_table.add_scope(Some(ir_scope));

            generate_block(block, target, outer_loop, irid_gen, ir_function, inner_ir_scope, symbol_table);

            None
        },

        _ => unreachable!("{:?} is not exprected.", node.item.value)
    }
}


/// Recursively generate the IR code for the given ScopeBlock.
/// This function does not take care of pushing and popping the block's scope, so manual stack managenent is required.
/// Manual scope management is required to produce more efficient code based on the context.
fn generate_block(mut block: ScopeBlock, target: Option<Tn>, outer_loop: Option<&LoopLabels>, irid_gen: &mut IRIDGenerator, ir_function: &mut FunctionIR, ir_scope: IRScopeID, symbol_table: &mut SymbolTable) {

    for statement in block.statements.drain(0..block.statements.len() - 1) {

        generate_node(statement, None, outer_loop, irid_gen, ir_function, ir_scope, block.scope_id, symbol_table);

    }

    let last_statement = block.statements.pop().unwrap();
    generate_node(last_statement, target, outer_loop, irid_gen, ir_function, ir_scope, block.scope_id, symbol_table);

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
fn generate_function<'a>(function: Function<'a>, irid_gen: &mut IRIDGenerator, symbol_table: &mut SymbolTable) -> FunctionIR<'a> {
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
    ir_function.push(IROperator::Label { label: ir_function.function_labels.start });

    let function_size = symbol_table.total_scope_size(function.code.scope_id);
    ir_function.push(IROperator::PushScope { bytes: function_size });

    symbol_table.map_function_label(FunctionUUID { name: function.name.to_string(), scope: function.parent_scope }, ir_function.function_labels.start);

    generate_block(function.code, return_tn, None, irid_gen, &mut ir_function, ir_scope, symbol_table);

    ir_function.push(IROperator::Label { label: ir_function.function_labels.exit });

    ir_function.push(IROperator::PopScope { bytes: function_size });

    ir_function.push(IROperator::Return);

    ir_function
}


/// Generate ir code from the given functions
pub fn generate<'a>(functions: Vec<Function<'a>>, symbol_table: &mut SymbolTable) -> Vec<FunctionIR<'a>> {

    let mut ir_functions = Vec::new();
    let mut irid_gen = IRIDGenerator::new();

    println!("\n\nGenerating IR code for the following functions:");

    for function in functions {

        ir_functions.push(
            generate_function(function, &mut irid_gen, symbol_table)
        );

        println!("\n{}\n", ir_functions.last().unwrap());

    }

    ir_functions
}

