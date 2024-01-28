use std::borrow::Cow;
use std::collections::HashMap;

use crate::error::WarnResult;
use crate::data_types::DataType;


/// Struct representing a symbol in the source code.
/// 
/// A symbol is a variable, function, any identifier that can be referenced by name.
pub struct Symbol {
    pub data_type: DataType,
    pub mutable: bool,
    pub initialized: bool,
}


#[derive(Debug)]
pub enum StaticLiteral<'a> {
    String (Cow<'a, str>),
}


#[derive(Debug)]
pub struct StaticValue<'a> {
    pub data_type: DataType,
    pub value: StaticLiteral<'a>,
}


pub struct Scope {
    pub parent: Option<ScopeID>,
    pub symbols: HashMap<String, Symbol>,
}


#[derive(Debug, Copy, Clone)]
pub struct ScopeID(usize);

impl ScopeID {

    pub const fn placeholder() -> Self {
        Self(usize::MAX)
    }
}


#[derive(Debug, Copy, Clone, PartialEq)]
pub struct StaticID(usize);


/// Struct containing the local symbols of a scope. 
pub struct SymbolTable<'a> {
    scopes: Vec<Scope>,
    statics: Vec<StaticValue<'a>>,
}

impl<'a> SymbolTable<'a> {

    pub fn new() -> Self {
        Self {
            scopes: Vec::new(),
            statics: Vec::new(),
        }
    }


    pub fn add_static_string(&mut self, string: Cow<'a, str>) -> StaticID {
        let id = self.statics.len();
        self.statics.push(StaticValue {
            data_type: DataType::RawString { length: string.len() },
            value: StaticLiteral::String(string),
        });
        StaticID(id)
    }


    pub fn get_static_string(&self, id: StaticID) -> &str {
        match &self.statics[id.0].value {
            StaticLiteral::String(string) => string,
        }
    }


    /// Return whether the symbol is declared in the symbol table in any scope.
    pub fn exists_symbol(&self, symbol_id: &str) -> bool {
        self.scopes.iter().any(|scope| scope.symbols.contains_key(symbol_id))
    }

    
    pub fn declare_symbol(&mut self, name: String, symbol: Symbol, scope_id: ScopeID) -> WarnResult<&'static str> {

        // TODO: eventually, use an immutable borrow of the string in the source code to avoid useless copying

        let shadow = self.scopes[scope_id.0].symbols.insert(name, symbol);

        if shadow.is_some() {
            WarnResult::Warning("Symbol already declared in this scope. The new symbol will shadow the previous declaration.")
        } else {
            WarnResult::Ok
        }
    }


    /// Get the symbol with the given id from the symbol table.
    pub fn get(&self, scope_id: ScopeID, symbol_id: &str) -> Option<&Symbol> {
        let scope = &self.scopes[scope_id.0];

        let symbol = scope.symbols.get(symbol_id);
        if symbol.is_some() {
            symbol
        } else if let Some(parent_id) = scope.parent {
            self.get(parent_id, symbol_id)
        } else {
            None
        }
    }


    unsafe fn _get_mut(&mut self, scope_id: ScopeID, symbol_id: &str) -> Option<*mut Symbol> {
        let scope = &mut self.scopes[scope_id.0];

        let symbol = scope.symbols.get_mut(symbol_id);
        if symbol.is_some() {
            symbol.map(|s| s as *mut Symbol)
        } else if let Some(parent_id) = scope.parent {
            self._get_mut(parent_id, symbol_id)
        } else {
            None
        }
    }


    /// Creates a new scope in the symbol table and returns its id.
    pub fn add_scope(&mut self, parent: Option<ScopeID>) -> ScopeID {
        let id = ScopeID(self.scopes.len());
        self.scopes.push(Scope {
            parent,
            symbols: HashMap::new(),
        });
        id
    }


    pub fn get_mut(&mut self, scope_id: ScopeID, symbol_id: &str) -> Option<&mut Symbol> {
        unsafe { self._get_mut(scope_id, symbol_id).map(|s| &mut *s) }
    }


    // /// Assumes the symbol exists and is not already initialized
    // pub fn set_initialized(&mut self, scope_id: ScopeID, symbol_id: &str) {
    //     let symbol = unsafe { self._get_mut(scope_id, symbol_id).unwrap().as_mut().unwrap() };
    //     symbol.initialized = true;
    // }

}

