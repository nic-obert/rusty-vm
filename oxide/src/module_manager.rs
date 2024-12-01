use std::ptr::NonNull;
use std::pin::Pin;
use std::path::{Path, PathBuf};
use std::mem;
use std::fs;
use std::io;
use std::env;
use std::collections::HashMap;
use std::cell::UnsafeCell;

use indoc::formatdoc;

use rusty_vm_lib::oxide::{LIBRARY_ENV_VARIABLE, OXIDE_FILE_EXTENSION};

use crate::symbol_table::{Name, SymbolTable};
use crate::lang::errors;


pub struct ModuleManager<'a> {

    /// Box<Module> is used to allow mutating the hashmap without invalidating references to the Module struct.
    modules: UnsafeCell<HashMap<&'a Path, Pin<Box<Module<'a>>>>>,

    /// Directories to be used as reference points when resolving the path of an included module.
    include_paths: Box<[Box<Path>]>,

    /// Owns the various paths of the modules, allowing &'a Path to be passed around safely.
    paths: UnsafeCell<Vec<Box<Path>>>,

}

impl<'a> ModuleManager<'a> {

    pub fn new(mut include_paths: Vec<Box<Path>>) -> Self {

        if let Some(env_include_paths) = env::var_os(LIBRARY_ENV_VARIABLE) {

            let env_include_paths = env_include_paths.
                as_os_str().
                to_str()
                .unwrap_or_else(|| errors::io_error(
                    io::Error::new(io::ErrorKind::InvalidData, format!("Invalid unicode in environment variable {}: \"{}\"", LIBRARY_ENV_VARIABLE, env_include_paths.display()).as_str()),
                    "Could not parse include library environment variable."
                ));

            for include_path in env_include_paths.split(':') {
                // Cloning here is acceptable since this is a one-time operation done at startup.
                include_paths.push(include_path.to_owned().into());
            }
        }

        Self {
            modules: UnsafeCell::new(HashMap::with_capacity(1)), // We should expect at least one module
            include_paths: include_paths.into_boxed_slice(),
            paths: UnsafeCell::new(Vec::with_capacity(1)),
        }
    }


    fn resolve_module_path(&self, module_name: &Path, parent_dir: Option<&Path>) -> Result<PathBuf, io::Error> {

        let module_with_ext = PathBuf::from(module_name).with_extension(OXIDE_FILE_EXTENSION);

        // Maybe the path can be canonicalized without further information
        if let Ok(resolved_path) = module_with_ext.canonicalize() {
            return Ok(resolved_path);
        }

        // If the path is relative, check if it's relative to the caller
        if let Some(parent_dir) = parent_dir {
            if let Ok(resolved_path) = parent_dir.join(&module_with_ext).canonicalize() {
                return Ok(resolved_path);
            }
        }

        // Check if the path is relative to any specified include paths
        let mut ok_path = None;
        for include_path in self.include_paths.iter() {
            if let Ok(resolved_path) = include_path.join(&module_with_ext).canonicalize() {
                ok_path = Some(resolved_path);
                break;
            }
        }
        if let Some(resolved_path) = ok_path {
            Ok(resolved_path)
        } else {
            Err(io::Error::new(
                io::ErrorKind::NotFound,
                formatdoc!("
                    Could not resolve module name \"{}\" from directory \"{}\".

                    Include paths are:
                    {}
                ",
                    module_name,
                    parent_dir.display(),
                    self.include_paths.iter().fold(String::new(), |acc, path| acc + path.to_string_lossy().as_ref() + "\n")
                )
            ))
        }
    }


    pub fn load_module(&self, parent_dir: Option<&Path>, module_name: &Path) -> Result<Pin<&'a Module<'a>>, io::Error> {

        let module_path = self.resolve_module_path(module_name, parent_dir)?;

        let modules = unsafe { &mut *self.modules.get() };
        // Don't reload the module if it's already loaded
        if modules.contains_key(module_path.as_path()) { // TODO: use if let Some() to avoid querying the hashmap twice
            // TODO: This is very ugly, but the borrow checker couldn't understand that the immutable borrow of `modules` ends afte the if block.
            return Ok(modules.get(module_path.as_path()).unwrap().as_ref());
        }

        let paths = unsafe { &mut *self.paths.get() };
        paths.push(module_path.into_boxed_path());
        let module_path = paths.last().unwrap().as_path();

        let source_code = match fs::read_to_string(module_path) {
            Ok(source_code) => source_code,
            Err(err) => return Err(err),
        };

        let module = Box::pin(Module::new(source_code, module_path));

        // This is safe because we are inserting a &Path, which is a wide pointer to a heap-allocated string.
        // Moving the owners in the paths Vec does not invalidate this &Path
        modules.insert(module_path, module);

        // TODO: This is ugly and inefficient, but it satisfies the borrow checker. Find a better way
        Ok(modules.get(module_path.as_path()).unwrap().as_ref())
    }


    pub fn get_module(&self, module_path: &'a Path) -> Pin<&Module<'a>> {
        let modules = unsafe { &mut *self.modules.get() };

        modules.get(module_path).unwrap().as_ref()
    }

}


pub struct Module<'a> {

    /// The canonicalized path of the module
    pub path: &'a Path,
    /// Borrows symbol names from the source code.
    symbol_table: SymbolTable<'a>,
    source_code: Pin<Box<str>>,
    /// Lines are owned by the source code.
    source_lines: Box<[NonNull<str>]>,
    /// Maps the exported name to a symbol in the symbol table.
    exports: HashMap<&'a str, &'a Name<'a>>,
}

impl<'a> Module<'a> {

    pub fn new(source_code: String, path: &'a Path) -> Self {

        let source_code = Box::into_pin(source_code.into_boxed_str());

        let mut lines = Vec::new();
        for line in source_code.lines() {
            lines.push(
                NonNull::from(line)
            );
        }

        Self {
            path,
            symbol_table: SymbolTable::new(),
            source_code,
            source_lines: lines.into_boxed_slice(),
            exports: Default::default()
        }
    }


    pub fn lines(&self) -> &[&str] {
        unsafe {
            mem::transmute::<&[NonNull<str>], &[&str]>(
                self.source_lines.as_ref()
            )
        }
    }

}
