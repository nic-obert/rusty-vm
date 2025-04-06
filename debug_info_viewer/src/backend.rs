use rusty_vm_lib::debugger::{DebugSectionsTable, SectionParseError, DEBUG_SECTIONS_TABLE_ID};


pub fn has_debug_info(program: &[u8]) -> bool {
    program.len() >= DEBUG_SECTIONS_TABLE_ID.len()
    && &program[..DEBUG_SECTIONS_TABLE_ID.len()] == DEBUG_SECTIONS_TABLE_ID
}


pub fn read_debug_sections_table(program: &[u8]) -> Result<DebugSectionsTable, SectionParseError> {
    DebugSectionsTable::try_parse(program)
}
