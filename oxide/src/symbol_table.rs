use std::collections::HashMap;

use crate::data_types::DataType;


/// Struct representing a symbol in the source code.
/// 
/// A symbol is a variable, function, any identifier that can be referenced by name.
pub struct Symbol {
    pub name: String,
    pub data_type: DataType,
    pub mutable: bool,
    pub initialized: bool,
}

impl Symbol {

    pub fn new(id: String, data_type: DataType, mutable: bool) -> Symbol {
        Symbol {
            name: id,
            data_type,
            mutable,
            initialized: false,
        }
    }

}


pub struct Scope {
    pub parent: Option<ScopeID>,
    pub symbols: HashMap<String, Symbol>,
}


#[derive(Debug, Copy, Clone)]
pub struct ScopeID(usize);


/// Struct containing the local symbols of a scope. 
pub struct SymbolTable {
    scopes: Vec<Scope>,
}

impl SymbolTable {

    pub fn new() -> Self {
        Self {
            scopes: Vec::new(),
        }
    }

    /// Return whether the symbol is declared in the symbol table in any scope.
    pub fn exists_symbol(&self, symbol_id: &str) -> bool {
        self.scopes.iter().any(|scope| scope.symbols.contains_key(symbol_id))
    }

    pub fn declare_symbol(&mut self, symbol: Symbol, scope_id: ScopeID) {
        // TODO: Check if the symbol is already declared
        // TODO: create a unique id for the symbol
        self.scopes[scope_id.0].symbols.insert(symbol.name.clone(), symbol);
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

    /// Creates a new scope in the symbol table and returns its id.
    pub fn add_scope(&mut self, parent: Option<ScopeID>) -> ScopeID {
        let id = ScopeID(self.scopes.len());
        self.scopes.push(Scope {
            parent,
            symbols: HashMap::new(),
        });
        id
    }

    /// Set the initialized flag for the symbol.
    pub fn set_initialized(&mut self, scope_id: ScopeID, symbol_id: &str) {
        let symbol = self.scopes[scope_id.0].symbols.get_mut(symbol_id).unwrap();
        symbol.initialized = true;
    }

}

