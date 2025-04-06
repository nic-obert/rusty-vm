use core::str;
use std::ffi::OsStr;
use std::fmt;
use std::os::unix::ffi::OsStrExt;
use std::path::Path;
use std::range::Range;
use std::str::Utf8Error;
use std::time::Duration;
use std::slice;
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


pub enum SectionParseError {
    NoDebugInfo,
    InvalidLabelNamesRange,
    InvalidSourceFilesRange,
    InvalidLabelsRange,
    InvalidInstructionsRange,
}


pub struct DebugSectionsTable {
    pub label_names: Range<Address>,
    pub source_files: Range<Address>,
    pub labels: Range<Address>,
    pub instructions: Range<Address>,
}

impl DebugSectionsTable {

    pub const SECTION_SIZE_ON_DISK: usize = DEBUG_SECTIONS_TABLE_ID.len() + mem::size_of::<DebugSectionsTable>();


    /// Try to parse the debug sections table, if present.
    /// If the table is not present at the beginning of the given slice, return `None`.
    /// If the table is present, try to parse and return it.
    /// Return an error if the table is malformed.
    pub fn try_parse(bytes: &[u8]) -> Result<Self, SectionParseError> {
        if !bytes.starts_with(DEBUG_SECTIONS_TABLE_ID) {
            return Err(SectionParseError::NoDebugInfo);
        }

        let mut chunks = bytes[DEBUG_SECTIONS_TABLE_ID.len()..].array_chunks::<ADDRESS_SIZE>();

        // Note that the order of the following operations is critical

        let label_names = {
            if let (Some(start), Some(end)) = (chunks.next(), chunks.next()) {
                Range { start: Address::from_le_bytes(*start), end: Address::from_le_bytes(*end) }
            } else {
                return Err(SectionParseError::InvalidLabelNamesRange);
            }
        };

        let source_files = {
            if let (Some(start), Some(end)) = (chunks.next(), chunks.next()) {
                Range { start: Address::from_le_bytes(*start), end: Address::from_le_bytes(*end) }
            } else {
                return Err(SectionParseError::InvalidSourceFilesRange);
            }
        };

        let labels = {
            if let (Some(start), Some(end)) = (chunks.next(), chunks.next()) {
                Range { start: Address::from_le_bytes(*start), end: Address::from_le_bytes(*end) }
            } else {
                return Err(SectionParseError::InvalidLabelsRange);
            }
        };

        let instructions = {
            if let (Some(start), Some(end)) = (chunks.next(), chunks.next()) {
                Range { start: Address::from_le_bytes(*start), end: Address::from_le_bytes(*end) }
            } else {
                return Err(SectionParseError::InvalidInstructionsRange);
            }
        };

        Ok(DebugSectionsTable {
            labels, instructions, source_files, label_names
        })
    }


    pub fn write_header_section(&self, buf: &mut [u8]) {

        struct UnsafeWriter<'a> {
            buf: &'a mut [u8],
            cursor: usize,
        }
        impl<'a> UnsafeWriter<'a> {
            pub fn new(buf: &'a mut [u8], max_write_size: usize) -> Self {
                assert!(buf.len() >= max_write_size);
                Self {
                    buf,
                    cursor: 0
                }
            }
            pub fn write(&mut self, data: &[u8]) {
                self.buf[self.cursor..self.cursor + data.len()].copy_from_slice(data);
                self.cursor += data.len();
            }
            pub fn write_address(&mut self, addr: Address) {
                self.buf[self.cursor..self.cursor + ADDRESS_SIZE].copy_from_slice(&addr.to_le_bytes());
                self.cursor += ADDRESS_SIZE;
            }
        }

        let mut writer = UnsafeWriter::new(buf, Self::SECTION_SIZE_ON_DISK);

        writer.write(DEBUG_SECTIONS_TABLE_ID);
        writer.write_address(self.label_names.start);
        writer.write_address(self.label_names.end);
        writer.write_address(self.source_files.start);
        writer.write_address(self.source_files.end);
        writer.write_address(self.labels.start);
        writer.write_address(self.labels.end);
        writer.write_address(self.instructions.start);
        writer.write_address(self.instructions.end);
    }

}


pub enum StringParsingError {
    OutOfSectionBounds { start: usize },
    Unterminated { start: usize },
    ExpectedUtf8 { start: usize, error: Utf8Error },
}

impl fmt::Display for StringParsingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StringParsingError::OutOfSectionBounds { start } => write!(f, "string out of section bounds (starts at byte {})", *start),
            StringParsingError::Unterminated { start } => write!(f, "string was not terminated (starts at byte {})", *start),
            StringParsingError::ExpectedUtf8 { start, error } => write!(f, "string uses an unsupported encoding (starts at byte {}, error: {})", *start, error),
        }
    }
}

/// Try to read the source files section.
/// This function does not check if the provided debug sections table is valid for the given program.
pub fn read_source_files<'a>(sections_table: &DebugSectionsTable, program: &'a [u8]) -> impl Iterator<Item = Result<&'a Path, StringParsingError>> {
    gen {
        let mut cursor = sections_table.source_files.start;

        while cursor < sections_table.source_files.end {

            // Read the null-terminated string
            let str_start = cursor;
            let null_terminator: usize;
            loop {
                if program.len() <= cursor {
                    yield Err(StringParsingError::Unterminated { start: str_start });
                    return;
                }
                if cursor >= sections_table.source_files.end {
                    yield Err(StringParsingError::OutOfSectionBounds { start: str_start });
                    return;
                }

                if program[cursor] == 0 {
                    null_terminator = cursor;
                    cursor += 1;
                    break;
                }
                cursor += 1;
            }

            // Convert the null-terminated string into a Path
            yield Ok(Path::new(OsStr::from_bytes(unsafe {
                slice::from_raw_parts(
                    program.as_ptr().byte_add(str_start),
                    null_terminator - str_start
                )
            })));
        }
    }
}


pub fn read_label_names<'a>(sections_table: &DebugSectionsTable, program: &'a [u8]) -> impl Iterator<Item = Result<&'a str, StringParsingError>> {
    gen {
        let mut cursor = sections_table.label_names.start;

        while cursor < sections_table.label_names.end {

            // Read the null-terminated string
            let str_start = cursor;
            let null_terminator: usize;
            loop {
                if program.len() <= cursor {
                    yield Err(StringParsingError::Unterminated { start: str_start });
                    return;
                }
                if cursor >= sections_table.source_files.end {
                    yield Err(StringParsingError::OutOfSectionBounds { start: str_start });
                    return;
                }

                if program[cursor] == 0 {
                    null_terminator = cursor;
                    cursor += 1;
                    break;
                }
                cursor += 1;
            }

            let s = unsafe {
                slice::from_raw_parts(
                    program.as_ptr().byte_add(str_start),
                    null_terminator - str_start
                )
            };
            match str::from_utf8(s) {
                Ok(string) => yield Ok(string),
                Err(error) => yield Err(StringParsingError::ExpectedUtf8 { start: str_start, error })
            }
        }
    }
}


pub struct LabelInfo {
    /// Address where the label name string is located
    pub name: Address,
    /// Address the label points to
    pub address: Address,
    /// Address of the source file path this label was originally defined in
    pub source_file: Address,
    /// Source line this label was originally defined at. This is the line number (not the line index)
    pub source_line: usize,
    /// Source column this label was originally defined at
    pub source_column: usize,
}

impl LabelInfo {

    pub fn write(&self, buf: &mut Vec<u8>) {
        buf.extend(self.name.to_le_bytes());
        buf.extend(self.address.to_le_bytes());
        buf.extend(self.source_file.to_le_bytes());
        buf.extend(self.source_line.to_le_bytes());
        buf.extend(self.source_column.to_le_bytes());
    }

}


pub struct InstructionInfo {
    /// Address where the instruction's first byte is located in the binary program.
    /// A source instruction may be comprised of multiple machine operations. This is the address of the first one of those operations.
    pub pc: Address,
    /// Address of the source file path, located in the source files section
    pub source_file: Address,
    /// Line at which the instruction is found in the source code. This is the line numer (not the line index)
    pub source_line: usize,
    /// Source column this instruction is found at
    pub source_column: usize,
}

impl InstructionInfo {

    pub fn write(&self, buf: &mut Vec<u8>) {
        buf.extend(self.pc.to_le_bytes());
        buf.extend(self.source_file.to_le_bytes());
        buf.extend(self.source_line.to_le_bytes());
        buf.extend(self.source_column.to_le_bytes());
    }

}
