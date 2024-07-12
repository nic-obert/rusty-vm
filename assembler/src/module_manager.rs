use std::cell::UnsafeCell;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::io;
use std::fmt;
use std::env;
use std::pin::Pin;
use std::ptr::NonNull;
use std::mem;

use indoc::formatdoc;
use rusty_vm_lib::assembly::LIBRARY_ENV_VARIABLE;

use crate::error;
use crate::symbol_table::ExportedSymbols;
use crate::tokenizer::SourceCode;


/// A canonicalized absolute path
#[derive(Hash, Eq, PartialEq, Debug, Clone, Copy)]
pub struct UnitPath<'a> {

    path: &'a Path

}

impl UnitPath<'_> {

    pub fn as_path<'a>(&'a self) -> &'a Path {
        self.path
    }

}

impl fmt::Display for UnitPath<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.path.display())
    }
}


/// Struct must not implement Clone or Copy
pub struct AsmUnit<'a> {

    /// The unit's exported symbols
    /// TODO: this should be an Option
    pub exports: ExportedSymbols<'a>,

    /// The actual owned source code string
    _raw_source: Pin<Box<str>>,

    lines: Box<[NonNull<str>]>


}

impl AsmUnit<'_> {

    pub fn new(raw_source: String) -> Self {
        
        // The source code lives until the program terminates because it can be referenced anytime for error messages, so it's ok to use a static lifetime.
        // Also, the source code owns the strings of the program.
        let raw_source = Box::into_pin(raw_source.into_boxed_str());

        let mut lines = Vec::new();
        for line in raw_source.lines() {
            lines.push(
                NonNull::from(line)
            );
        }

        Self {
            _raw_source: raw_source,
            lines: lines.into_boxed_slice(),
            exports: Default::default()
        }
    }


    pub fn lines(&self) -> SourceCode {
        unsafe {
            mem::transmute::<&[NonNull<str>], &[&str]>(
                self.lines.as_ref()
            )
        }
    }

}


pub struct ModuleManager<'a> {

    // Here a Box is used to allow mutating the `units` HashMap without invalidating references to the AsmUnit
    // The Box itself will change, but not the address it points to
    units: UnsafeCell<HashMap<UnitPath<'a>, Box<AsmUnit<'a>>>>,

    /// Vector that owns various paths of ASM modules.
    /// Having references to the paths stored here is safe because they reference the actual string,
    /// and not the PathBuf itself. Remember that PathBuf is just a wrapper around a Vec of bytes
    paths: UnsafeCell<Vec<PathBuf>>,

    /// Directories to be used when resolving the path of an included ASM module.
    include_paths: Box<[PathBuf]>,

}

impl<'a> ModuleManager<'a> {

    pub fn new(mut include_paths: Vec<PathBuf>) -> Self {

        if let Some(env_include_paths) = env::var_os(LIBRARY_ENV_VARIABLE) {
            
            let env_include_paths = env_include_paths.as_os_str().to_str()
                .unwrap_or_else(|| error::io_error(io::Error::new(io::ErrorKind::InvalidFilename, format!("Invalid unicode in environment variable {}: \"{}\"", LIBRARY_ENV_VARIABLE, env_include_paths.display()).as_str()), "Could not parse include library path environment variable."));

            for include_path in env_include_paths.split(':') {
                // Cloning here is acceptable since this is a one-time operation done at startup.
                include_paths.push(include_path.to_owned().into());
            }
        }

        Self {
            units: Default::default(),
            paths: Default::default(),
            include_paths: include_paths.into_boxed_slice(),
        }
    }


    /// Try to resolve `included_path` from the parent folder `caller_directory`.
    /// If the path is resolved successfully, it's added to the manager's owned paths.
    pub fn resolve_include_path(&self, caller_directory: &Path, included_path: &'a Path) -> Result<UnitPath<'a>, io::Error> {

        let paths = unsafe { &mut *self.paths.get() };

        // Maybe the path can be canonicalized without further information
        if let Ok(resolved_path) = included_path.canonicalize() {
            paths.push(resolved_path);
            return Ok(UnitPath { path: paths.last().expect("Just pushed") });
        }

        // If the path is relative, check if it's relative to the caller
        if let Ok(resolved_path) = caller_directory.join(included_path).canonicalize() {
            paths.push(resolved_path);
            return Ok(UnitPath { path: paths.last().expect("Just pushed") });
        }

        // Check if the path is relative to any specified include paths
        for include_path in self.include_paths.iter() {
            if let Ok(resolved_path) = include_path.join(included_path).canonicalize() {
                paths.push(resolved_path);
                return Ok(UnitPath { path: paths.last().expect("Just pushed") });
            }
        }

        // No valid path was found, the include path could not be resolved
        Err(io::Error::new(
            io::ErrorKind::NotFound, 
            formatdoc!("
                Could not resolve the path \"{}\" from directory \"{}\".
                
                Include paths are:
                {}
                ", 
                included_path.display(),
                caller_directory.display(),
                self.include_paths.iter().fold(String::new(), |acc, path| acc + path.to_string_lossy().as_ref() + "\n")
            ).as_str()
        ))
    }


    pub fn add_unit(&self, path: UnitPath<'a>, unit: AsmUnit<'a>) -> &AsmUnit<'a> {

        // This is safe because no references to the map or its elements is ever returned mutably
        let units = unsafe { &mut *self.units.get() };

        let unit_box = Box::new(unit);
        let unit_ref = unit_box.as_ref() as *const AsmUnit;

        units.insert(path, unit_box);

        // Returns a ref to the newly added unit. Since the unit is stored in the heap and is never moved, its memory address won't change
        // and the reference will be valid for the lifetime of the module manager
        unsafe {
            &*unit_ref as &AsmUnit
        }
    }


    /// Get an immutable reference to the assembly unit
    pub fn get_unit(&self, path: UnitPath) -> &AsmUnit<'a> {
        let units = unsafe { &*self.units.get() };

        // Cast away lifetime. The compiler does not recognize that `path` does 
        // indeed live long enough and that the returned value does not borrow from `path`
        let path = unsafe {
            mem::transmute::<UnitPath, UnitPath>(path)
        };

        units.get(&path).expect("Entry should exist")
    }


    pub fn set_unit_exports(&self, path: UnitPath<'a>, exports: ExportedSymbols<'a>) -> &ExportedSymbols<'a> {
        let units = unsafe { &mut *self.units.get() };
        let unit = units.get_mut(&path).expect("Entry should exist");
        unit.exports = exports;
        &unit.exports
    }


    pub fn get_unit_exports(&self, path: UnitPath<'a>) -> &ExportedSymbols<'a> {
        let units = unsafe { &*self.units.get() };
        let unit = units.get(&path).expect("Entry should exist");
        &unit.exports
    }


    pub fn is_loaded(&self, path: UnitPath<'a>) -> bool {
        let units = unsafe { &*self.units.get() };
        units.contains_key(&path)
    }

}

