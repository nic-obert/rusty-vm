use std::borrow::Cow;
use std::cell::RefCell;
use std::rc::Rc;
use std::collections::HashMap;

use crate::error::WarnResult;
use crate::data_types::{DataType, LiteralValue};


/// Struct representing a symbol in the source code.
/// 
/// A symbol is a variable, function, any identifier that can be referenced by name.
pub struct Symbol {
    pub data_type: Rc<DataType>,
    pub line_index: usize,
    pub column: usize,
    pub value: SymbolValue,
    pub initialized: bool,
    pub read_from: bool,
}

impl Symbol {

    pub fn initialize_immutable(&mut self, value: LiteralValue) {

        assert!(matches!(self.value, SymbolValue::Immutable(None)));
        assert!(!self.initialized);
        
        self.value = SymbolValue::Immutable(Some(value));
        self.initialized = true;
    }

    pub fn get_value(&self) -> Option<&LiteralValue> {
        match &self.value {
            SymbolValue::Mutable => None,
            SymbolValue::Immutable(v) => v.into(),
            SymbolValue::Constant(v) => Some(v),

            SymbolValue::UninitializedConstant => unreachable!(),
        }
    } 


    pub fn is_mutable(&self) -> bool {
        match self.value {
            SymbolValue::Mutable => true,

            SymbolValue::Immutable(_) |
            SymbolValue::Constant(_) 
             => false,

            SymbolValue::UninitializedConstant => unreachable!(),
        }
    }


    pub const fn line_number(&self) -> usize {
        self.line_index + 1
    }

}


pub enum SymbolValue {
    Mutable,
    Immutable (Option<LiteralValue>),
    Constant (LiteralValue),

    UninitializedConstant,
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
    pub symbols: HashMap<String, Vec<RefCell<Symbol>>>,
}


#[derive(Debug, Copy, Clone)]
pub struct ScopeID(usize);

impl ScopeID {

    pub const fn placeholder() -> Self {
        Self(usize::MAX)
    }
}

impl std::fmt::Display for ScopeID {

    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.0 == Self::placeholder().0 {
            write!(f, "placeholder")
        } else {
            write!(f, "{}", self.0)
        }
    }

}


#[derive(Debug, Copy, Clone, PartialEq)]
pub struct StaticID(usize);


#[derive(Debug, Copy, Clone)]
pub struct ScopeDiscriminant(u16);

#[allow(clippy::derivable_impls)]
impl Default for ScopeDiscriminant {

    fn default() -> Self {
        Self(0)
    }

}


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


    pub fn define_constant(&self, name: &str, discriminant: ScopeDiscriminant, scope_id: ScopeID, value: LiteralValue) -> Result<(), ()> {
        
        let mut symbol = self.get_symbol(scope_id, name, discriminant)
            .ok_or(())?
            .borrow_mut();

        assert!(matches!(symbol.value, SymbolValue::UninitializedConstant));

        symbol.value = SymbolValue::Constant(value);
        symbol.initialized = true;
        Ok(())
    }


    /// Return the requested symbol if it exists in the symbol table.
    pub fn get_unreachable_symbol(&self, symbol_id: &str) -> Option<&RefCell<Symbol>> {
        self.scopes.iter()
            .find_map(|scope| scope.symbols.get(symbol_id))
            .and_then(|s| s.last())
    }

    
    pub fn declare_symbol(&mut self, name: String, symbol: Symbol, scope_id: ScopeID) -> (ScopeDiscriminant, WarnResult<&'static str>) {

        // TODO: eventually, use an immutable borrow of the string in the source code to avoid useless copying

        let symbol_list = self.scopes[scope_id.0].symbols.entry(name).or_default();
        let discriminant = ScopeDiscriminant(symbol_list.len() as u16);
        
        symbol_list.push(
            RefCell::new(symbol)
        );

        let warning = if discriminant.0 > 0 {
            WarnResult::Warning("Symbol already declared in this scope. The new symbol will overshadow the previous declaration.")
        } else {
            WarnResult::Ok
        };

        (discriminant, warning)
    }


    pub fn get_current_discriminant(&self, name: &str, scope_id: ScopeID) -> Option<ScopeDiscriminant> {
        let scope = &self.scopes[scope_id.0];
        if let Some(discriminant) = scope.symbols.get(name).map(|s| ScopeDiscriminant(s.len() as u16 - 1)) {
            Some(discriminant)
        } else if let Some(parent_id) = scope.parent {
            self.scopes[parent_id.0].symbols.get(name).map(|s| ScopeDiscriminant(s.len() as u16 - 1))
        } else {
            None
        }
    }


    /// Get the symbol with the given id from the symbol table.
    pub fn get_symbol(&self, scope_id: ScopeID, symbol_id: &str, discriminant: ScopeDiscriminant) -> Option<&RefCell<Symbol>> {
        let scope = &self.scopes[scope_id.0];

        if let Some(symbol_list) = scope.symbols.get(symbol_id) {
            Some(&symbol_list[discriminant.0 as usize])
        } else if let Some(parent_id) = scope.parent {
            self.get_symbol(parent_id, symbol_id, discriminant)
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

}

