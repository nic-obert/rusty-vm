use std::rc::Rc;
use std::collections::HashMap;
use std::cell::{Ref, RefCell, UnsafeCell};
use std::borrow::Cow;

use rusty_vm_lib::vm::Address;

use crate::error;
use crate::lang::{FunctionMacroDef, InlineMacroDef, LabelDef};
use crate::module_manager::ModuleManager;


struct LabelExport<'a> {
    name: &'a str,
    def: LabelDef<'a>
}

impl<'a> Into<(&'a str, LabelDef<'a>)> for LabelExport<'a> {
    fn into(self) -> (&'a str, LabelDef<'a>) {
        (self.name, self.def)
    }
}


struct InlineMacroExport<'a> {
    name: &'a str,
    def: InlineMacroDef<'a>
}

impl<'a> Into<(&'a str, InlineMacroDef<'a>)> for InlineMacroExport<'a> {
    fn into(self) -> (&'a str, InlineMacroDef<'a>) {
        (self.name, self.def)
    }
}

struct FunctionMacroExport<'a> {
    name: &'a str,
    def: FunctionMacroDef<'a>
}

impl<'a> Into<(&'a str, FunctionMacroDef<'a>)> for FunctionMacroExport<'a> {
    fn into(self) -> (&'a str, FunctionMacroDef<'a>) {
        (self.name, self.def)
    }
}

pub struct ExportedSymbols<'a> {

    labels: Box<[LabelExport<'a>]>,
    inline_macros: Box<[InlineMacroExport<'a>]>,
    function_macros: Box<[FunctionMacroExport<'a>]>,

}


#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct SymbolID(pub usize);

#[derive(Debug, Clone, Copy)]
pub struct StaticID(pub usize);


pub enum StaticValue<'a> {
    StringLiteral(Cow<'a, str>),
    // TODO: add arrays
}

impl StaticValue<'_> {

    pub fn as_string<'a>(&'a self) -> &'a str {
        match self {
            Self::StringLiteral(s) => s,
        }
    }

}


pub struct SymbolTable<'a> {

    labels: UnsafeCell<HashMap<&'a str, LabelDef<'a>>>,
    inline_macros: UnsafeCell<HashMap<&'a str, InlineMacroDef<'a>>>,
    function_macros: UnsafeCell<HashMap<&'a str, FunctionMacroDef<'a>>>,

    statics: UnsafeCell<Vec<RefCell<StaticValue<'a>>>>,

    export_labels: UnsafeCell<Vec<&'a str>>,
    export_inline_macros: UnsafeCell<Vec<&'a str>>,
    export_function_macros: UnsafeCell<Vec<&'a str>>,

}

impl<'a> SymbolTable<'a> {

    pub fn new() -> Self {
        Self {
            labels: Default::default(),
            statics: Default::default(),
            inline_macros: Default::default(),
            function_macros: Default::default(),
            export_function_macros: Default::default(),
            export_labels: Default::default(),
            export_inline_macros: Default::default(),
        }
    }


    pub fn import_symbols(&self, imports: ExportedSymbols<'a>, re_export: bool, module_manager: &ModuleManager) {

        let labels = unsafe { &mut *self.labels.get() };
        let inline_macros = unsafe { &mut *self.inline_macros.get() };
        let function_macros = unsafe { &mut *self.function_macros.get() };

        if re_export {

            unsafe { &mut *self.export_labels.get() }
                .extend(imports.labels.iter()
                    .map(|l| l.name)
            );

            unsafe { &mut *self.export_inline_macros.get() }
                .extend(imports.inline_macros.iter()
                    .map(|m| m.name)
            );

            unsafe { &mut *self.export_function_macros.get() }
                .extend(imports.function_macros.iter()
                    .map(|m| m.name)
            );
        }

        labels.reserve(imports.labels.len());
        for import in imports.labels {

            let new_source = Rc::clone(&import.def.source);

            if let Some(old_def) = labels.insert(import.name, import.def) {
                error::symbol_redeclaration(&old_def.source, &new_source, module_manager, "Imported label conflicts with existing symbol")
            }
        }

        inline_macros.reserve(imports.inline_macros.len());
        for import in imports.inline_macros {

            let new_source = Rc::clone(&import.def.source);
            
            if let Some(old_def) = inline_macros.insert(import.name, import.def) {
                error::symbol_redeclaration(&old_def.source, &new_source, module_manager, "Imported inline macro conflicts with existing symbol")
            }
        }

        function_macros.reserve(imports.function_macros.len());
        for import in imports.function_macros {

            let new_source = Rc::clone(&import.def.source);

            if let Some(old_def) = function_macros.insert(import.name, import.def) {
                error::symbol_redeclaration(&old_def.source, &new_source, module_manager, "Imported function macro conflicts with existing symbol")
            }
        }
    }


    pub fn export_symbols(mut self) -> ExportedSymbols<'a> {

        ExportedSymbols {

            labels: unsafe { &*self.export_labels.get() }
                .iter()
                .map(|name| 
                    LabelExport {
                        name, 
                        def: self.labels.get_mut().remove(name).unwrap()
                    }
                )
                .collect::<Vec<LabelExport>>()
                .into_boxed_slice(),

            inline_macros: unsafe { &*self.export_inline_macros.get() }
                .iter()
                .map(|name| 
                    InlineMacroExport {
                        name,
                        def: self.inline_macros.get_mut().remove(name).unwrap()
                    }
                )
                .collect::<Vec<InlineMacroExport>>()
                .into_boxed_slice(),

            function_macros: unsafe { &*self.export_function_macros.get() }
                .iter()
                .map(|name| 
                    FunctionMacroExport {
                        name,
                        def: self.function_macros.get_mut().remove(name).unwrap()
                    }
                )
                .collect::<Vec<FunctionMacroExport>>()
                .into_boxed_slice(),

        }
    }


    pub fn declare_static(&self, value: StaticValue<'a>) -> StaticID {

        let statics = unsafe { &mut *self.statics.get() };

        let id = statics.len();
        statics.push(RefCell::new(value));
        StaticID(id)
    }


    pub fn declare_inline_macro(&self, name: &'a str, def: InlineMacroDef<'a>, export: bool) -> Option<InlineMacroDef> {
        let macros = unsafe { &mut *self.inline_macros.get() };

        let old_def = macros.insert(name, def);

        if export {
            unsafe { &mut *self.export_inline_macros.get() }.push(name);
        }

        old_def
    }


    pub fn declare_function_macro(&self, name: &'a str, def: FunctionMacroDef<'a>, export: bool) -> Option<FunctionMacroDef> {
        let macros = unsafe { &mut *self.function_macros.get() };

        let old_def = macros.insert(name, def);

        if export {
            unsafe { &mut *self.export_function_macros.get() }.push(name);
        }

        old_def
    }

    
    pub fn get_static(&self, id: StaticID) -> &'a StaticValue<'a> {
        let statics = unsafe { &*self.statics.get() };
        // Leak is safe because statics never get mutated
        Ref::leak(statics[id.0].borrow())
    }


    pub fn declare_label(&self, name: &'a str, def: LabelDef<'a>, export: bool) -> Option<LabelDef> {

        let labels = unsafe { &mut *self.labels.get() };

        let old_def = labels.insert(name, def);

        if export {
            unsafe { &mut *self.export_labels.get() }.push(name);
        }

        old_def
    }


    /// Assumes the label is present in the table
    pub fn define_label(&self, name: &'a str, value: Address) {

        let labels = unsafe { &mut *self.labels.get() };

        labels.get_mut(name).unwrap().value = Some(value);
    }


    /// Assumes the label is present in the table
    pub fn get_resolved_label(&self, name: &str) -> Option<Address> {

        let labels = unsafe { &*self.labels.get() };

        labels.get(name).unwrap().value
    }


    pub fn get_inline_macro(&self, id: &str) -> Option<&InlineMacroDef<'a>> {
        let macros = unsafe { &*self.inline_macros.get() };

        macros.get(id)
    }

}

