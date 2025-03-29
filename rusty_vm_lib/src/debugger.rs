use std::range::Range;
use std::time::Duration;
use std::mem;

use crate::registers::CPURegisters;
use crate::vm::{Address, ADDRESS_SIZE};


pub const DEBUGGER_ATTACH_SLEEP: Duration = Duration::from_millis(200);
pub const DEBUGGER_COMMAND_WAIT_SLEEP: Duration = Duration::from_millis(50);
pub const DEBUGGER_UPDATE_WAIT_SLEEP: Duration = Duration::from_millis(10);

pub const CPU_REGISTERS_OFFSET: usize = 0;
pub const RUNNING_FLAG_OFFSET: usize = CPU_REGISTERS_OFFSET + mem::size_of::<CPURegisters>();
pub const TERMINATE_COMMAND_OFFSET: usize = RUNNING_FLAG_OFFSET + mem::size_of::<bool>();
pub const VM_UPDATED_COUNTER_OFFSET: usize = TERMINATE_COMMAND_OFFSET + mem::size_of::<bool>();
pub const VM_MEM_OFFSET: usize = VM_UPDATED_COUNTER_OFFSET + mem::size_of::<u8>();

pub const DEBUGGER_PATH_ENV: &str = "RUSTYVM_DEBUGGER";


pub const DEBUG_SECTIONS_TABLE_ID: &[u8] = "DEBUG SECTIONS\0".as_bytes();


pub struct DebugSectionsTable {
    labels: Range<Address>,
    instructions: Range<Address>,
    source_files: Range<Address>,
    label_names: Range<Address>,
}

impl DebugSectionsTable {

    /// Try to parse the debug sections table, if present.
    /// If the table is not present at the beginning of the given slice, return `None`.
    /// If the table is present, try to parse and return it.
    /// Return an error if the table is malformed.
    pub fn try_parse(bytes: &[u8]) -> Option<Result<Self, ()>> {
        if !bytes.starts_with(DEBUG_SECTIONS_TABLE_ID) {
            return None;
        }

        let mut chunks = bytes[DEBUG_SECTIONS_TABLE_ID.len()..].array_chunks::<ADDRESS_SIZE>();

        let labels = {
            if let (Some(start), Some(end)) = (chunks.next(), chunks.next()) {
                Range { start: Address::from_le_bytes(*start), end: Address::from_le_bytes(*end) }
            } else {
                return Some(Err(()));
            }
        };

        let instructions = {
            if let (Some(start), Some(end)) = (chunks.next(), chunks.next()) {
                Range { start: Address::from_le_bytes(*start), end: Address::from_le_bytes(*end) }
            } else {
                return Some(Err(()));
            }
        };

        let source_files = {
            if let (Some(start), Some(end)) = (chunks.next(), chunks.next()) {
                Range { start: Address::from_le_bytes(*start), end: Address::from_le_bytes(*end) }
            } else {
                return Some(Err(()));
            }
        };

        let label_names = {
            if let (Some(start), Some(end)) = (chunks.next(), chunks.next()) {
                Range { start: Address::from_le_bytes(*start), end: Address::from_le_bytes(*end) }
            } else {
                return Some(Err(()));
            }
        };

        Some(Ok(DebugSectionsTable {
            labels, instructions, source_files, label_names
        }))
    }

}


pub struct LabelInfo {
    /// Address where the label name string is located
    name: Address,
    /// Address the label points to
    address: Address
}


pub struct InstructionInfo {
    /// Address where the instruction's first byte is located in the binary program.
    /// A source instruction may be comprised of multiple machine operations. This is the address of the first one of those operations.
    pc: Address,
    /// Line at which the instruction is found in the source code
    source_line: usize,
    /// Address of the source file path, located in the source files section
    source_file: Address
}
