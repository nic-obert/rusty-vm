use std::path::Path;
use std::fmt;

use crate::tokenizer;
use crate::module_manager::ModuleManager;
use crate::lang::errors::io_error;


pub enum CompilationPhases {

    Tokenization,
    SyntaxAnalysis,

}

impl fmt::Display for CompilationPhases {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", match self {
            CompilationPhases::Tokenization => "tokenization",
            CompilationPhases::SyntaxAnalysis => "syntax analysis",
        })
    }
}


/// Returns the syntax trees of all loaded modules
pub fn prepare_modules(module_manager: &mut ModuleManager, main_module_name: &Path) -> Vec<()> {
    /*
        The main module is loaded into the module manager (the source code is divided into lines and a Module struct is created).
        Starting from the main module, the source code is tokenized and its syntax parsed into syntax trees.
        From analyzing the resulting syntax trees, new modules may be included due to import statements.

    */

    let current_dir = std::env::current_dir()
        .unwrap_or_else(|err| io_error(err, "Could not get current directory"));
    let parent_dir = current_dir
        .parent()
        .unwrap_or_else(|| Path::new(""));

    let root_module = module_manager.load_module_if_new(parent_dir, main_module_name)
        .unwrap_or_else(|err| io_error(err, "Could not load source file"))
        .expect("Root module should not be already loaded");

    let tokens = tokenizer::tokenize(&root_module);

    println!("{:?}", tokens);

    todo!()
}
