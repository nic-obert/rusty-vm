use std::rc::Rc;
use std::collections::HashMap;
use std::cell::UnsafeCell;

use rusty_vm_lib::vm::Address;
use rusty_vm_lib::assembly::SourceToken;

use crate::error;
use crate::lang::{FunctionMacroDef, InlineMacroDef, LabelDef};
use crate::module_manager::ModuleManager;


#[derive(Debug, Clone)]
struct LabelExport<'a> {
    name: &'a str,
    def: LabelDef<'a>
}

impl<'a> From<LabelExport<'a>> for (&'a str, LabelDef<'a>) {
    fn from(val: LabelExport<'a>) -> Self {
        (val.name, val.def)
    }
}


#[derive(Debug, Clone)]
struct InlineMacroExport<'a> {
    name: &'a str,
    def: InlineMacroDef<'a>
}

impl<'a> From<InlineMacroExport<'a>> for (&'a str, InlineMacroDef<'a>) {
    fn from(val: InlineMacroExport<'a>) -> Self {
        (val.name, val.def)
    }
}

#[derive(Debug, Clone)]
struct FunctionMacroExport<'a> {
    name: &'a str,
    def: FunctionMacroDef<'a>
}

impl<'a> From<FunctionMacroExport<'a>> for (&'a str, FunctionMacroDef<'a>) {
    fn from(val: FunctionMacroExport<'a>) -> Self {
        (val.name, val.def)
    }
}


#[derive(Default, Debug)]
pub struct ExportedSymbols<'a> {

    labels: Box<[LabelExport<'a>]>,
    inline_macros: Box<[InlineMacroExport<'a>]>,
    function_macros: Box<[FunctionMacroExport<'a>]>,

}


pub struct SymbolTable<'a> {

    labels: UnsafeCell<HashMap<&'a str, LabelDef<'a>>>,
    inline_macros: UnsafeCell<HashMap<&'a str, InlineMacroDef<'a>>>,
    function_macros: UnsafeCell<HashMap<&'a str, FunctionMacroDef<'a>>>,

    export_labels: UnsafeCell<Vec<&'a str>>,
    export_inline_macros: UnsafeCell<Vec<&'a str>>,
    export_function_macros: UnsafeCell<Vec<&'a str>>,

}

impl<'a> SymbolTable<'a> {

    pub fn new() -> Self {
        Self {
            labels: Default::default(),
            inline_macros: Default::default(),
            function_macros: Default::default(),
            export_function_macros: Default::default(),
            export_labels: Default::default(),
            export_inline_macros: Default::default(),
        }
    }


    pub fn import_symbols(&self, imports: &ExportedSymbols<'a>, re_export: bool, module_manager: &ModuleManager<'a>) {

        // TODO: find a way to avoid cloning each definition for every export. Maybe we could use an Rc<Definition> for this since definitions should be accessible from wherever they are imported.

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
        for import in &imports.labels {

            let new_source = Rc::clone(&import.def.source);

            if let Some(old_def) = labels.insert(import.name, import.def.clone()) {
                error::symbol_redeclaration(&old_def.source, &new_source, module_manager, "Imported label conflicts with existing symbol")
            }
        }

        inline_macros.reserve(imports.inline_macros.len());
        for import in &imports.inline_macros {

            let new_source = Rc::clone(&import.def.source);

            if let Some(old_def) = inline_macros.insert(import.name, import.def.clone()) {
                error::symbol_redeclaration(&old_def.source, &new_source, module_manager, "Imported inline macro conflicts with existing symbol")
            }
        }

        function_macros.reserve(imports.function_macros.len());
        for import in &imports.function_macros {

            let new_source = Rc::clone(&import.def.source);

            if let Some(old_def) = function_macros.insert(import.name, import.def.clone()) {
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


    pub fn declare_inline_macro(&self, name: &'a str, def: InlineMacroDef<'a>, export: bool) -> Result<(), InlineMacroDef> {
        let macros = unsafe { &mut *self.inline_macros.get() };

        let old_def = macros.insert(name, def);

        if export {
            unsafe { &mut *self.export_inline_macros.get() }.push(name);
        }

        if let Some(old_def) = old_def {
            Err(old_def)
        } else {
            Ok(())
        }
    }


    pub fn declare_function_macro(&self, name: &'a str, def: FunctionMacroDef<'a>, export: bool) -> Result<(), FunctionMacroDef> {
        let macros = unsafe { &mut *self.function_macros.get() };

        let old_def = macros.insert(name, def);

        if export {
            unsafe { &mut *self.export_function_macros.get() }.push(name);
        }

        if let Some(old_def) = old_def {
            Err(old_def)
        } else {
            Ok(())
        }
    }


    pub fn declare_label(&self, name: &'a str, def: Rc<SourceToken<'a>>, export: bool) -> Result<(), LabelDef> {

        let labels = unsafe { &mut *self.labels.get() };

        let old_def = labels.insert(name, LabelDef {
            source: def,
            value: None
        });

        if export {
            unsafe { &mut *self.export_labels.get() }.push(name);
        }

        if let Some(old_def) = old_def {
            Err(old_def)
        } else {
            Ok(())
        }
    }


    /// Assumes the label is present in the table
    pub fn define_label(&self, name: &'a str, value: Address) {

        let labels = unsafe { &mut *self.labels.get() };

        labels.get_mut(name)
            .unwrap_or_else(|| panic!("Label `{name}` should already be declared"))
            .value = Some(value);
    }


    pub fn get_resolved_label(&self, name: &str) -> Option<Address> {

        let labels = unsafe { &*self.labels.get() };

        labels.get(name)?.value
    }


    pub fn get_inline_macro(&self, id: &str) -> Option<&InlineMacroDef<'a>> {
        let macros = unsafe { &*self.inline_macros.get() };

        macros.get(id)
    }


    pub fn inline_macros(&self) -> impl Iterator<Item = &'a str> {
        let macros = unsafe { &*self.inline_macros.get() };

        macros.keys().copied()
    }


    pub fn get_function_macro(&self, id: &str) -> Option<&FunctionMacroDef<'a>> {
        let macros = unsafe { &*self.function_macros.get() };

        macros.get(id)
    }


    pub fn function_macros(&self) -> impl Iterator<Item = &'a str> {
        let macros = unsafe { &*self.function_macros.get() };

        macros.keys().copied()
    }

}
