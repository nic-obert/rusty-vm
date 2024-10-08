use std::borrow::Cow;
use std::cell::RefCell;
use std::fmt::Display;
use std::rc::Rc;
use std::collections::HashMap;

use crate::lang::data_types::{DataType, LiteralValue};
use crate::irc::{IRCode, Label};
use crate::match_unreachable;
use crate::tokenizer::SourceToken;


/// Struct representing a symbol in the source code.
///
/// A symbol is a variable, function, any identifier that can be referenced by name.
pub struct Symbol<'a> {

    /// The data type this symbol was declared as.
    pub data_type: Rc<DataType>,
    /// The source code token this symbol was declared at.
    pub token: Rc<SourceToken<'a>>,
    /// The type and value of the symbol.
    pub value: SymbolValue<'a>,
    /// Whether the symbol has been initialized with a value.
    pub initialized: bool,
    /// Whether the symbol has been utilized at least once since its declaration.
    pub read_from: bool,
    /// Whether the symbol has been removed as a result of optimization.
    /// If a symbol has been removed, it should not be pushed to the stack.
    pub removed: bool,
    /// Whether the symbol is a function paremeter or not.
    /// This is used to differentiate between regular local symbols and function parameters,
    /// which may be handled differently by the bytecode generator.
    /// For example, local symbols may be included in the function's stack frame, but parameters may be placed before that.
    pub is_function_parameter: bool
}

impl<'a> Symbol<'a> {

    pub unsafe fn assume_function(&self) -> &FunctionInfo<'a> {
        match_unreachable!(SymbolValue::Function(info) = &self.value, info)
    }

    pub unsafe fn assume_function_mut(&mut self) -> &mut FunctionInfo<'a> {
        match_unreachable!(SymbolValue::Function(info) = &mut self.value, info)
    }


    pub fn new_uninitialized(data_type: Rc<DataType>, token: Rc<SourceToken<'a>>, value: SymbolValue<'a>, is_function_parameter: bool) -> Symbol<'a> {
        Symbol {
            data_type,
            token,
            value,
            initialized: false,
            read_from: false,
            removed: false,
            is_function_parameter
        }
    }


    pub fn new_function(signature: Rc<DataType>, param_names: Box<[&'a str]>, is_const: bool, token: Rc<SourceToken<'a>>) -> Symbol<'a> {
        Symbol {
            data_type: signature,
            token,
            value: SymbolValue::Function(FunctionInfo {
                constantness: if is_const { FunctionConstantness::MarkedConst } else { FunctionConstantness::NotConst },
                has_side_effects: false,
                param_names,
                code: None,
            }),
            initialized: true,
            read_from: false,
            removed: false,
            is_function_parameter: false
        }
    }


    pub fn initialize_immutable(&mut self, value: Rc<LiteralValue>) {

        assert!(matches!(self.value, SymbolValue::Immutable(None)));

        self.value = SymbolValue::Immutable(Some(value));
        self.initialized = true;
    }


    pub fn get_value(&self) -> Option<Rc<LiteralValue>> {
        match &self.value {
            SymbolValue::Mutable => None,
            SymbolValue::Function { .. } => None,
            SymbolValue::Immutable(v) => v.clone(),
            SymbolValue::Constant(v) => Some(v.clone()),

            SymbolValue::Static { init_value, mutable: false } => Some(init_value.clone()),
            // Since mutable statics can change at any moment, we cannot know their value at compile-time.
            SymbolValue::Static { mutable: true, .. } => None,

            SymbolValue::UninitializedConstant |
            SymbolValue::UninitializedStatic { .. }
             => unreachable!(),
        }
    }


    pub fn is_mutable(&self) -> bool {
        self.value.is_mutable()
    }


    pub fn line_number(&self) -> usize {
        self.token.line_index + 1
    }

}


/// Information about a function's IR code.
#[derive(Debug)]
pub struct FunctionCode {

    /// The starting label of the function.
    pub label: Label,
    /// The IR code of the function.
    /// Here we use shared mutability to allow for modification of the code for optimization reasons.
    pub code: Rc<RefCell<IRCode>>,

}


#[derive(Debug, Clone)]
#[allow(clippy::enum_variant_names)]
pub enum FunctionConstantness {
    NotConst,
    MarkedConst,
    ProvenConst,
}


#[derive(Debug)]
pub struct FunctionInfo<'a> {
    /// Whether the function is marked as const in the source code (or by the compiler if it can be determined that the function is const)
    pub constantness: FunctionConstantness,
    pub has_side_effects: bool,
    pub param_names: Box<[&'a str]>,
    /// The IR code of the function, if it has been generated.
    /// This code may be copy-pasted in place of the function call if the compiler determines it's good to do so.
    pub code: Option<FunctionCode>,
}


#[derive(Debug)]
pub enum SymbolValue<'a> {
    Mutable,
    Immutable (Option<Rc<LiteralValue>>),
    Constant (Rc<LiteralValue>),
    Function (FunctionInfo<'a>),
    Static { init_value: Rc<LiteralValue>, mutable: bool },

    UninitializedConstant,
    UninitializedStatic { mutable: bool },
}

impl SymbolValue<'_> {

    pub const fn is_mutable(&self) -> bool {
        match self {
            SymbolValue::Mutable |
            SymbolValue::Static { init_value: _, mutable: true }
             => true,

            SymbolValue::Static { init_value: _, mutable: false } |
            SymbolValue::Immutable(_) |
            SymbolValue::Constant(_) |
            SymbolValue::Function (_)
            => false,

            SymbolValue::UninitializedConstant |
            SymbolValue::UninitializedStatic { mutable: _ }
             => unreachable!(),
        }
    }

}


#[derive(Debug)]
pub enum StaticLiteral<'a> {
    String (Cow<'a, str>),
}

impl StaticLiteral<'_> {

    pub fn as_bytes(&self) -> &[u8] {
        match self {
            StaticLiteral::String(s) => s.as_bytes(),
        }
    }

}


#[derive(Debug)]
pub struct StaticValue<'a> {
    pub data_type: DataType,
    pub value: StaticLiteral<'a>,
}


pub struct TypeDef<'a> {
    pub definition: Rc<DataType>,
    pub token: Rc<SourceToken<'a>>
}


pub struct Scope<'a> {
    /// The parent scope of the current scope.
    /// This is used to loop for symbols that are not defined in the current scope, but may be defined in outer scopes.
    pub parent: Option<ScopeID>,
    /// The symbols defined in the current scope.
    pub symbols: HashMap<&'a str, Vec<RefCell<Symbol<'a>>>>,
    /// The types defined in the current scope.
    pub types: HashMap<&'a str, TypeDef<'a>>,
    /// The child scopes of the current scope.
    /// This is used to calculate the total size of the scope when pushing it to the stack.
    pub children: Vec<ScopeID>,
}

impl<'a> Scope<'a> {

    pub fn new(parent_id: Option<ScopeID>) -> Scope<'a> {
        Scope {
            parent: parent_id,
            symbols: Default::default(),
            types: Default::default(),
            children: Default::default(),
        }
    }


    pub fn get_symbol(&self, symbol_id: &str, discriminant: ScopeDiscriminant) -> Option<&RefCell<Symbol<'a>>> {
        self.symbols.get(symbol_id).map(move |s| &s[discriminant.0 as usize])
    }


    /// Get the symbol name-value pairs that have not been read from.
    pub fn get_unread_symbols(&self) -> Vec<(&str, &RefCell<Symbol<'a>>)> {
        self.symbols.iter()
            .flat_map(|(name, symbols)| symbols.iter().map(move |s| (*name, s)))
            .filter(|(_, symbol)| !symbol.borrow().read_from)
            .collect()
    }

}


#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
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


#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct StaticID(usize);


#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct ScopeDiscriminant(pub u16);

#[allow(clippy::derivable_impls)]
impl Default for ScopeDiscriminant {

    fn default() -> Self {
        Self(0)
    }

}

impl Display for ScopeDiscriminant {

    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }

}


/// Uniquely identifies a function by its name and its scope.
/// Function names must be unique within the same scope.
#[derive(PartialEq, Eq, Hash)]
pub struct FunctionUUID<'a> {
    pub name: &'a str,
    pub scope: ScopeID,
}


/// Struct containing the local symbols of a scope.
pub struct SymbolTable<'a> {
    scopes: Vec<Scope<'a>>,
    statics: Vec<StaticValue<'a>>,
    function_labels: HashMap<FunctionUUID<'a>, Label>,
}

impl<'a> SymbolTable<'a> {

    pub fn new() -> Self {
        Self {
            scopes: Vec::new(),
            statics: Vec::new(),
            function_labels: HashMap::new(),
        }
    }


    pub fn get_statics(&self) -> impl Iterator<Item = (StaticID, &StaticValue<'a>)> {
        self.statics.iter()
            .enumerate()
            .map(|(i, s)| (StaticID(i), s))
    }


    /// Set the given function as having (or not having) side effects.
    pub fn set_function_side_effects(&self, name: &str, scope_id: ScopeID, has_side_effects: bool) {
        let mut symbol = self.get_symbol(scope_id, name, ScopeDiscriminant(0))
            .unwrap() // Assume the symbol is present
            .borrow_mut();

        match_unreachable!(SymbolValue::Function (FunctionInfo { has_side_effects: x, .. }) = &mut symbol.value, *x = has_side_effects);
    }


    /// Maps a function id to a IR label, which will than be used to call the function.
    pub fn map_function_label(&mut self, function: FunctionUUID<'a>, label: Label) {
        self.function_labels.insert(function, label);
    }


    /// Get the size of a scope in bytes, including its children.
    /// This calculation does not include symbols that have been optimized out and function parameters.
    #[allow(clippy::type_complexity)]
    pub fn total_scope_size_excluding_parameters(&self, scope_id: ScopeID) -> Result<usize, Vec<(Rc<SourceToken>, Rc<DataType>)>> {

        let mut size = 0;
        let mut unknown_sizes = Vec::new();

        let scope = &self.scopes[scope_id.0];

        for symbol in scope.symbols.values().flat_map(|s| s.iter()) {

            let symbol = symbol.borrow();

            // Do not include removed symbols and function parameters in the scope stack size calculation.
            // Removed symbols have been optimized out.
            // Function parameters may be handled separately.
            if symbol.removed || symbol.is_function_parameter {
                continue;
            }

            match symbol.data_type.static_size() {
                Ok(s) => size += s,
                Err(()) => unknown_sizes.push((symbol.token.clone(), symbol.data_type.clone()))
            }
        }

        for child_id in &scope.children {
            match self.total_scope_size_excluding_parameters(*child_id) {
                Ok(s) => size += s,
                Err(e) => unknown_sizes.extend(e)
            }
        }

        if unknown_sizes.is_empty() {
            Ok(size)
        } else {
            Err(unknown_sizes)
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


    pub fn define_static(&self, name: &str, scope_id: ScopeID, value: Rc<LiteralValue>) -> Result<(), ()> {

        let mut symbol = self.get_symbol(scope_id, name, ScopeDiscriminant(0))
            .ok_or(())?
            .borrow_mut();

        let mutable = match_unreachable!(SymbolValue::UninitializedStatic { mutable } = &symbol.value, *mutable);

        symbol.value = SymbolValue::Static { init_value: value, mutable };
        symbol.initialized = true;
        Ok(())
    }


    pub fn define_constant(&self, name: &str, scope_id: ScopeID, value: Rc<LiteralValue>) -> Result<(), ()> {

        let mut symbol = self.get_symbol(scope_id, name, ScopeDiscriminant(0))
            .ok_or(())?
            .borrow_mut();

        assert!(matches!(symbol.value, SymbolValue::UninitializedConstant));

        symbol.value = SymbolValue::Constant(value);
        symbol.initialized = true;
        Ok(())
    }


    /// Return the requested symbol if it exists in the symbol table.
    pub fn get_unreachable_symbol(&self, symbol_id: &str) -> Option<&RefCell<Symbol<'a>>> {
        self.scopes.iter()
            .find_map(|scope|
                scope.get_symbol(symbol_id, ScopeDiscriminant(0)
            )
        )
    }


    pub fn declare_function(&mut self, name: &'a str, is_const: bool, signature: Rc<DataType>, param_names: Box<[&'a str]>, token: Rc<SourceToken<'a>>, scope_id: ScopeID) -> Result<(), Rc<SourceToken>> {

        let symbol_list = self.scopes[scope_id.0].symbols.entry(name).or_default();
        let discriminant = ScopeDiscriminant(symbol_list.len() as u16);

        symbol_list.push(
            RefCell::new(Symbol::new_function(signature, param_names, is_const, token))
        );

        // Cannot re-declare a function in the same scope
        if discriminant.0 > 0 {
            Err(symbol_list[(discriminant.0 - 1) as usize].borrow().token.clone())
        } else {
            Ok(())
        }
    }


    pub fn declare_constant_or_static(&mut self, name: &'a str, symbol: Symbol<'a>, scope_id: ScopeID) -> Result<(), Rc<SourceToken>> {

        let symbol_list = self.scopes[scope_id.0].symbols.entry(name).or_default();
        let discriminant = ScopeDiscriminant(symbol_list.len() as u16);

        symbol_list.push(
            RefCell::new(symbol)
        );

        // Cannot re-declare a constant or a static in the same scope
        if discriminant.0 > 0 {
            Err(symbol_list[(discriminant.0 - 1) as usize].borrow().token.clone())
        } else {
            Ok(())
        }
    }


    /// Declare the new symbol in the symbol table.
    /// Return the the previous declaration of the symbol, if it exists.
    pub fn declare_symbol(&mut self, name: &'a str, symbol: Symbol<'a>, scope_id: ScopeID) -> (ScopeDiscriminant, Option<Rc<SourceToken>>) {

        let symbol_list = self.scopes[scope_id.0].symbols.entry(name).or_default();
        let discriminant = ScopeDiscriminant(symbol_list.len() as u16);

        symbol_list.push(
            RefCell::new(symbol)
        );

        let prev_declaration = if discriminant.0 > 0 {
            Some(symbol_list[(discriminant.0 - 1) as usize].borrow().token.clone())
        } else {
            None
        };

        (discriminant, prev_declaration)
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
    pub fn get_symbol(&self, scope_id: ScopeID, symbol_id: &str, discriminant: ScopeDiscriminant) -> Option<&RefCell<Symbol<'a>>> {
        let scope = &self.scopes[scope_id.0];

        scope.get_symbol(symbol_id, discriminant).or_else(
            || if let Some(parent_id) = scope.parent {
                self.get_symbol(parent_id, symbol_id, discriminant)
            } else {
                None
            }
        )
    }


    /// Recursively search the requested function symbol in the given scope and its reachable parents.
    pub fn get_function(&self, name: &str, scope_id: ScopeID) -> Option<&RefCell<Symbol<'a>>> {
        let scope = &self.scopes[scope_id.0];

        if let Some(symbol) = scope.get_symbol(name, ScopeDiscriminant(0)) {
            if let SymbolValue::Function { .. } = symbol.borrow().value {
                return Some(symbol);
            }
        }

        if let Some(parent_id) = scope.parent {
            self.get_function(name, parent_id)
        } else {
            None
        }
    }


    /// Get the symbol with the given id from the symbol table.
    /// If the symbol is found outside the function boundary, including the boundary scope, return a true flag, else return a false flag.
    /// `function_boundary` is the scope id of the function's parent scope
    pub fn get_symbol_warn_if_outside_function(&self, scope_id: ScopeID, symbol_id: &str, discriminant: ScopeDiscriminant, function_boundary: ScopeID) -> (Option<&RefCell<Symbol<'a>>>, bool) {

        let scope = &self.scopes[scope_id.0];

        if let Some(symbol) = scope.get_symbol(symbol_id, discriminant) {
            (Some(symbol), false)
        } else if let Some(parent_id) = scope.parent {
            if parent_id == function_boundary {
                (self.get_symbol(scope_id, symbol_id, discriminant), true)
            } else {
                self.get_symbol_warn_if_outside_function(parent_id, symbol_id, discriminant, function_boundary)
            }
        } else {
            (None, true)
        }
    }


    /// Recursively checks if the symbol was declared in the top-level scope of the function identified by the given scope boundary.
    /// `function_top_scope` is the top-level scope of the function.
    pub fn is_function_top_level_symbol(&self, symbol_scope_id: ScopeID, function_top_scope: ScopeID, name: &str, discriminant: ScopeDiscriminant) -> bool {

        let scope = &self.scopes[symbol_scope_id.0];

        if scope.get_symbol(name, discriminant).is_some() {
            symbol_scope_id == function_top_scope
        } else if symbol_scope_id == function_top_scope {
            // If the symbol is not found in the top-level scope of the function, it is not a top-level symbol of the function. (Declared outside the function boundary)
            false
        } else if let Some(parent_id) = scope.parent {
            self.is_function_top_level_symbol(parent_id, function_top_scope, name, discriminant)
        } else {
            unreachable!("Function scopes should always have a parent scope. A top-level function has the global scope as parent scope. This is a bug.")
        }
    }


    /// Creates a new scope in the symbol table and returns its id.
    pub fn add_scope(&mut self, parent: Option<ScopeID>) -> ScopeID {

        let new_scope_id = ScopeID(self.scopes.len());
        self.scopes.push(
            Scope::new(parent)
        );

        if let Some(parent_id) = parent {
            let parent = &mut self.scopes[parent_id.0];
            parent.children.push(new_scope_id);
        }

        new_scope_id
    }


    pub fn get_unread_symbols(&self, scope_id: ScopeID) -> Vec<(&str, &RefCell<Symbol<'a>>)>{
        self.scopes[scope_id.0].get_unread_symbols()
    }


    fn get_type_def(&self, name: &str, scope_id: ScopeID) -> Option<&TypeDef> {
        let scope = &self.scopes[scope_id.0];

        if let Some(type_def) = scope.types.get(name) {
            Some(type_def)
        } else if let Some(parent_id) = scope.parent {
            self.get_type_def(name, parent_id)
        } else {
            None
        }
    }


    pub fn get_name_type(&self, name: &str, scope_id: ScopeID) -> Option<NameType> {
        self.get_current_discriminant(name, scope_id).map(NameType::Symbol)
            .or_else(|| self.get_type_def(name, scope_id).map(|type_def| NameType::Type(type_def.definition.clone())))
    }


    /// Try to define a new type in the scope.
    /// If a type with the same name is already defined in the same scope, return an error.
    pub fn define_type(&mut self, name: &'a str, scope_id: ScopeID, definition: Rc<DataType>, token: Rc<SourceToken<'a>>) -> Result<(), TypeDef> {

        let type_def = TypeDef {
            definition,
            token
        };

        let shadow = self.scopes[scope_id.0].types.insert(name, type_def);
        match shadow {
            Some(shadow) => Err(shadow),
            None => Ok(())
        }
    }


    pub fn get_scope(&self, scope_id: ScopeID) -> &Scope<'a> {
        &self.scopes[scope_id.0]
    }

}


pub enum NameType {
    Symbol(ScopeDiscriminant),
    Type(Rc<DataType>)
}
