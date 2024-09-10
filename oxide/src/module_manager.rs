use std::collections::HashMap;

use crate::symbol_table::Name;


pub struct ModuleManager {

}


pub struct Module<'a> {
    /// The name of the module. The string is either owned by the source code of another module or it's a command line argument.
    name: &'a str,
    /// Borrows symbol names from the source code.
    symbol_table: (),
    source_code: Box<str>,
    /// Lines are owned by the source code.
    source_lines: Box<[&'a str]>,
    /// Maps the exported name to a symbol in the symbol table.
    exports: HashMap<&'a str, &'a Name<'a>>,
}
