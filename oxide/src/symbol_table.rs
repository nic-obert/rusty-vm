use std::rc::Rc;
use std::collections::HashMap;
use std::cell::RefCell;

use crate::tokenizer::SourceToken;
use crate::lang::{LiteralValue, DataType};


pub struct SymbolTable<'a> {
    scopes: Vec<Scope<'a>>,
}

impl<'a> SymbolTable<'a> {

    pub fn new() -> Self {
        Self {
            scopes: vec![Scope::new(None)], // Initialize the global scope
        }
    }

}


struct Scope<'a> {
    /// Symbols declared in this scope.
    symbols: HashMap<&'a str, RefCell<Symbol<'a>>>,
    /// The outer scope this scope was declared in, if any.
    parent: Option<ScopeID>,
    /// Maps programmer-defined type names to their type definition.
    types: HashMap<&'a str, TypeDef<'a>>,
    /// Inner scopes declared in the outer level of this scope.
    children: Vec<ScopeID>,
    /// Namespaces declared in this scope.
    namespaces: Vec<RefCell<NameSpace<'a>>>,
}

impl<'a> Scope<'a> {

    pub fn new(parent_id: Option<ScopeID>) -> Self {
        Self {
            parent: parent_id,
            symbols: Default::default(),
            types: Default::default(),
            children: Default::default(),
            namespaces: Default::default(),
        }
    }

}


pub enum Name<'a> {
    Symbol (&'a RefCell<Symbol<'a>>),
    NameSpace (&'a RefCell<NameSpace<'a>>),
}


pub struct NameSpace<'a> {
    inner_names: Vec<Name<'a>>,
}


#[derive(Debug)]
pub struct SymbolID<'a> {
    name: &'a str,
    scope_id: ScopeID
}


#[derive(Debug)]
pub struct StaticID (usize);


struct TypeDef<'a> {
    pub definition: Rc<DataType>,
    pub source: Rc<SourceToken<'a>>,
}

#[derive(Debug)]
struct ScopeID(usize);


pub struct Symbol<'a> {
    pub source: Rc<SourceToken<'a>>,
    pub data_type: Rc<DataType>,
    pub symbol_value: SymbolValue<'a>,
    /// Whether the symbol is referenced anywhere for reading. Unread symbols may generate warnings and could be optimized out
    pub is_read: bool,
    pub is_public: bool, // ??? maybe it's not needed
    /// Whether the symbol has been optimized out
    pub removed: bool,
    /// It's considered an error to have uninitialized symbols
    pub initialized: bool,
    /// Function parameters may be handled differently by the bytecode generator because of stack frame stuff. See symbol_table.rs
    pub is_function_parameter: bool,
}

pub enum SymbolValue<'a> {
    Mutable,
    Immutable { value: Option<LiteralValue<'a>> },
    Constant { value: LiteralValue<'a> },
    Function {  },
    Static { mutable: bool, init_value: StaticID },

    UninitializedConstant,
    UninitializedStatic { mutable: bool },
}
