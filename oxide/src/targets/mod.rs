
use rusty_vm_lib::assembly::ByteCode;

use crate::symbol_table::SymbolTable;
use crate::flow_analyzer::FunctionGraph;
use crate::irc::IRIDGenerator;

use casey::lower;


pub mod rusty_vm;

pub trait CompiledBinary {

    fn as_bytes(&self) -> &[u8];

}

impl CompiledBinary for ByteCode {

    fn as_bytes(&self) -> &[u8] {
        self
    }

}


macro_rules! declare_targets {
    (
        $(
            $target_name:ident $($directive:ident)?
        ),*
    ) => {

        #[derive(Default)]
        pub enum Targets {

            $(
                $(#[$directive])?
                $target_name
            ),*

        }

        impl Targets {

            pub fn list_targets() {

            }

            pub fn from_string(target: &str) -> Option<Self> {
                // Not particularly efficient, but it's fine since this is done at most once when parsing command line arguments
                match target.to_lowercase().as_str() {
                    $(
                        // stringify! expands before other macros because of rustc internal magic. This way we can create a lowercase version of the expanded string with double quotes. Double quotes won't be changed.
                        lower!(stringify!($target_name)) => Some(Targets::$target_name),
                    )*
                    _ => None
                }
            }


            pub fn generate(&self, symbol_table: &SymbolTable, function_graphs: Vec<FunctionGraph>, irid_gen: IRIDGenerator) -> impl CompiledBinary {
                match self {
                    Targets::RustyVM => rusty_vm::generate_bytecode(symbol_table, function_graphs, irid_gen),
                }
            }
        }

    };
}

declare_targets!(
    RustyVM default
);
