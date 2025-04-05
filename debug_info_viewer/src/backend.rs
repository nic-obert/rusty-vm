use rusty_vm_lib::debugger::DEBUG_SECTIONS_TABLE_ID;


pub fn has_debug_info(program: &[u8]) -> bool {

    program.len() >= DEBUG_SECTIONS_TABLE_ID.len()
    && &program[..DEBUG_SECTIONS_TABLE_ID.len()] == DEBUG_SECTIONS_TABLE_ID

}
