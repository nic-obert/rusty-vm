use core::fmt::{Debug, Display};

use std::cell::RefCell;
use std::rc::Rc;
use std::collections::HashMap;

use crate::symbol_table::{ScopeDiscriminant, ScopeID, SymbolTable};
use crate::open_linked_list::OpenLinkedList;
use crate::lang::data_types::{DataType, LiteralValue};

use super::ir_parser::{FunctionLabels, IRIDGenerator};


/// Represents a temporary variable
/// Tns are write-once, read-only variables. They are effectively single-assignment register variables.
/// They have to be unique and immutable because it may be useful to store the result of one operation and re-use it later.
#[derive(Debug, Clone)]
pub struct Tn {
    pub id: TnID,
    pub data_type: Rc<DataType>,
}

impl Display for Tn {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "T{}", self.id.0)
    }
}


#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Label(pub LabelID);

impl Display for Label {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "L{}", self.0.0)
    }

}

impl Label {

    pub const fn to_le_bytes(&self) -> [u8; 8] {
        self.0.0.to_le_bytes()
    }

}


/// Represents an operand of ir operations
pub enum IRValue {

    Tn (Tn),
    Const (Rc<LiteralValue>),

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


pub enum IRJumpTarget {

    Tn (Tn),
    Label (Label),

}

impl Debug for IRJumpTarget {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)

    }
}

impl Display for IRJumpTarget {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IRJumpTarget::Tn(tn) => write!(f, "{}", tn),
            IRJumpTarget::Label(label) => write!(f, "{}", label),
        }
    }
}


#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TnID(pub usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct LabelID(pub usize);


/// Stores information about a scope in the IR generation phase.
pub struct IRScope<'a> {

    /// Maps symbol names to Tns
    pub(super) symbols: HashMap<&'a str, Vec<Tn>>,
    /// Keeps track of the return type and in which Tn to store it
    pub(super) return_tn: Option<Tn>,
    pub(super) parent: Option<IRScopeID>,

}

impl IRScope<'_> {

    pub fn new(parent: Option<IRScopeID>) -> Self {
        Self {
            symbols: HashMap::new(),
            return_tn: None,
            parent,
        }
    }

}


#[derive(Clone, Copy)]
pub struct IRScopeID(pub usize);

/// Stores information about function scopes in the IR
pub struct ScopeTable<'a> {
    /// List of all the scopes. This is NOT a stack.
    pub scopes: Vec<IRScope<'a>>,
}

impl<'a> ScopeTable<'a> {

    pub fn new() -> Self {
        Self {
            scopes: vec![IRScope::new(None)],
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
        self.scopes[ir_scope.0].symbols.get(name)
            .and_then(|symbol_list| symbol_list.get(discriminant.0 as usize).cloned())
            .or_else(|| self.scopes[ir_scope.0].parent
                .and_then(|parent| self.get_tn(name, discriminant, parent)))
    }


    /// Map a symbol name to a Tn in the given scope
    pub fn map_symbol(&mut self, name: &'a str, tn: Tn, ir_scope: IRScopeID) {
        self.scopes[ir_scope.0].symbols.entry(name).or_default().push(tn);
    }

}


/// Represents an intermediate code operation.
#[derive(Debug)]
pub enum IROperator {

    /// Add `left` and `right`, and store the result in `target`
    Add { target: Tn, left: IRValue, right: IRValue },
    /// Subtract `left` and `right`, and store the result in `target`
    Sub { target: Tn, left: IRValue, right: IRValue },
    /// Multiply `left` and `right`, and store the result in `target`
    Mul { target: Tn, left: IRValue, right: IRValue },
    /// Divide `left` and `right`, and store the result in `target`
    Div { target: Tn, left: IRValue, right: IRValue },
    /// Mod `left` and `right`, and store the result in `target`
    Mod { target: Tn, left: IRValue, right: IRValue },

    /// Copy the value of source into target.
    Assign { target: Tn, source: IRValue },
    /// Copy the value pointed to by `ref_` into the target
    Deref { target: Tn, ref_: IRValue },
    /// Copy the value of the source into the address pointed to by the target
    DerefAssign { target: Tn, source: IRValue },
    /// Copy the address of the `ref_` into the target
    Ref { target: Tn, ref_: Tn },

    Greater { target: Tn, left: IRValue, right: IRValue },
    Less { target: Tn, left: IRValue, right: IRValue },
    GreaterEqual { target: Tn, left: IRValue, right: IRValue },
    LessEqual { target: Tn, left: IRValue, right: IRValue },
    Equal { target: Tn, left: IRValue, right: IRValue },
    NotEqual { target: Tn, left: IRValue, right: IRValue },

    BitShiftLeft { target: Tn, left: IRValue, right: IRValue },
    BitShiftRight { target: Tn, left: IRValue, right: IRValue },
    BitNot { target: Tn, operand: IRValue },
    BitAnd { target: Tn, left: IRValue, right: IRValue },
    BitOr { target: Tn, left: IRValue, right: IRValue },
    BitXor { target: Tn, left: IRValue, right: IRValue },

    // TODO: add swap byte endianness instruction

    /// Copy the raw bits from `source` into `target`.
    /// Assume that `target` is either the same size or larget than `source`.
    Copy { target: Tn, source: IRValue },
    /// Copy the raw bits from `source` into the address pointed to by `target`.
    /// Assume that `target` is either the same size or larget than `source`.
    DerefCopy { target: Tn, source: IRValue },

    Jump { target: Label },
    JumpIf { condition: Tn, target: Label },
    JumpIfNot { condition: Tn, target: Label },
    Label { label: Label },

    Call { return_target: Option<Tn>, return_label: Label, callable: IRJumpTarget, args: Box<[IRValue]> },
    Return,

    #[allow(dead_code)]
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
            IROperator::DerefAssign { target, source } => write!(f, "[{}] = {}", target, source),
            IROperator::Deref { target, ref_ } => write!(f, "{} = [{}]", target, ref_),
            IROperator::Ref { target, ref_ } => write!(f, "{} = &{}", target, ref_),
            IROperator::Greater { target, left, right } => write!(f, "{} = {} > {}", target, left, right),
            IROperator::Less { target, left, right } => write!(f, "{} = {} < {}", target, left, right),
            IROperator::GreaterEqual { target, left, right } => write!(f, "{} = {} >= {}", target, left, right),
            IROperator::LessEqual { target, left, right } => write!(f, "{} = {} <= {}", target, left, right),
            IROperator::Equal { target, left, right } => write!(f, "{} = {} == {}", target, left, right),
            IROperator::NotEqual { target, left, right } => write!(f, "{} = {} != {}", target, left, right),
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
            IROperator::Nop => write!(f, "nop"),
            IROperator::Copy { target, source } => write!(f, "copy {} -> {}", source, target),
            IROperator::DerefCopy { target, source } => write!(f, "copy {} -> [{}]", source, target),
        }
    }
}


/// Represents a single operation in the IR code.
/// Each operation may or may not have side effects.
#[derive(Debug)]
pub struct IRNode {

    pub op: IROperator,
    pub has_side_effects: bool,

}

impl Display for IRNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}", self.op, if self.has_side_effects { " // side effects" } else { "" })
    }

}


pub type IRCode = OpenLinkedList<IRNode>;


pub struct FunctionIR<'a> {

    /// Name of the function
    pub name: &'a str,

    /// The IR code of the function
    pub code: Rc<RefCell<IRCode>>,

    pub scope_table: ScopeTable<'a>,

    /// The top-level scope of the function in the symbol table.
    /// This will be needed by the bytecode generator to access local symbols and to calculate the function's stack frame size.
    pub st_top_scope: ScopeID,

    /// Important labels of the function that are needed to perform a call
    pub function_labels: FunctionLabels,

    /// The function signature
    pub signature: Rc<DataType>,

    pub return_tn: Option<Tn>,

    pub ir_top_scope: IRScopeID
}

impl FunctionIR<'_> {

    pub fn new_with_top_scope<'a>(name: &'a str, return_type: Rc<DataType>, first_scope: ScopeID, signature: Rc<DataType>, irid_gen: &mut IRIDGenerator) -> FunctionIR<'a> {

        let mut scope_table = ScopeTable::new();

        // Create the top-level function scope
        let ir_scope = scope_table.add_scope(None);

        // Create a Tn for the return value, if it isn't Void. Non-void return statements will assign to this Tn
        let return_tn = if !matches!(return_type.as_ref(), DataType::Void) {
            let return_tn = Tn { id: irid_gen.next_tn(), data_type: return_type };
            scope_table.scopes[ir_scope.0].return_tn = Some(return_tn.clone());
            Some(return_tn)
        } else {
            None
        };

        FunctionIR {
            name,
            code: Default::default(),
            scope_table,
            st_top_scope: first_scope,
            function_labels: FunctionLabels {
                start: irid_gen.next_label(),
                exit: irid_gen.next_label(),
            },
            signature,
            return_tn,
            ir_top_scope: ir_scope
        }
    }


    pub fn push_code(&mut self, node: IRNode) {
        self.code.borrow_mut().push_back(node);
    }


    pub fn parent_scope(&self, symbol_table: &SymbolTable) -> ScopeID {
        // Assume the parent scope exists since the global scope should always exist
        symbol_table.get_scope(self.st_top_scope).parent.unwrap()
    }

}

impl Display for FunctionIR<'_> {

    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "fn {} {{", self.name)?;
        for op in self.code.borrow().iter() {
            writeln!(f, "    {}", op)?;
        }
        writeln!(f, "}}")
    }

}
