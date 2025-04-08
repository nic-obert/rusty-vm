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


pub enum DebugSectionsTableParseError {
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
    pub fn try_parse(bytes: &[u8]) -> Result<Self, DebugSectionsTableParseError> {
        if !bytes.starts_with(DEBUG_SECTIONS_TABLE_ID) {
            return Err(DebugSectionsTableParseError::NoDebugInfo);
        }

        let mut chunks = bytes[DEBUG_SECTIONS_TABLE_ID.len()..].array_chunks::<ADDRESS_SIZE>();

        // Note that the order of the following operations is critical

        let label_names = {
            if let (Some(start), Some(end)) = (chunks.next(), chunks.next()) {
                Range { start: Address::from_le_bytes(*start), end: Address::from_le_bytes(*end) }
            } else {
                return Err(DebugSectionsTableParseError::InvalidLabelNamesRange);
            }
        };

        let source_files = {
            if let (Some(start), Some(end)) = (chunks.next(), chunks.next()) {
                Range { start: Address::from_le_bytes(*start), end: Address::from_le_bytes(*end) }
            } else {
                return Err(DebugSectionsTableParseError::InvalidSourceFilesRange);
            }
        };

        let labels = {
            if let (Some(start), Some(end)) = (chunks.next(), chunks.next()) {
                Range { start: Address::from_le_bytes(*start), end: Address::from_le_bytes(*end) }
            } else {
                return Err(DebugSectionsTableParseError::InvalidLabelsRange);
            }
        };

        let instructions = {
            if let (Some(start), Some(end)) = (chunks.next(), chunks.next()) {
                Range { start: Address::from_le_bytes(*start), end: Address::from_le_bytes(*end) }
            } else {
                return Err(DebugSectionsTableParseError::InvalidInstructionsRange);
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
    OutOfSectionBounds { start: Address, section_end: Address },
    Unterminated { start: Address },
    ExpectedUtf8 { start: Address, error: Utf8Error },
}

impl fmt::Display for StringParsingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StringParsingError::OutOfSectionBounds { start, section_end } => write!(f, "string out of section bounds (starts at byte {} {:#X}, section ends at byte {} {:#X})", *start, *start, *section_end, *section_end),
            StringParsingError::Unterminated { start } => write!(f, "string was not terminated (starts at byte {} {:#X})", *start, *start),
            StringParsingError::ExpectedUtf8 { start, error } => write!(f, "string uses an unsupported encoding (starts at byte {} {:#X}, error: {})", *start, *start, error),
        }
    }
}


pub fn has_debug_info(program: &[u8]) -> bool {
    program.len() >= DEBUG_SECTIONS_TABLE_ID.len()
    && &program[..DEBUG_SECTIONS_TABLE_ID.len()] == DEBUG_SECTIONS_TABLE_ID
}


/// Try to read the source files section.
/// This function does not check if the provided debug sections table is valid for the given program.
pub fn read_source_files_section<'a>(sections_table: &DebugSectionsTable, program: &'a [u8]) -> impl Iterator<Item = Result<&'a Path, StringParsingError>> {
    gen {
        let mut cursor = sections_table.source_files.start;

        while cursor < sections_table.source_files.end {

            let str_start = cursor;
            let slice = match try_read_c_string(str_start, sections_table.source_files.end, program) {
                Ok(s) => s,
                Err(err) => {
                    yield Err(err);
                    return;
                }
            };
            cursor += slice.len() + 1;

            // Convert the null-terminated string into a Path
            yield Ok(Path::new(OsStr::from_bytes(slice)));
        }
    }
}


pub fn read_label_names_section<'a>(sections_table: &DebugSectionsTable, program: &'a [u8]) -> impl Iterator<Item = Result<&'a str, StringParsingError>> {
    gen {
        let mut cursor = sections_table.label_names.start;

        while cursor < sections_table.label_names.end {

            let str_start = cursor;
            let slice = match try_read_c_string(str_start, sections_table.label_names.end, program) {
                Ok(s) => s,
                Err(err) => {
                    yield Err(err);
                    return;
                }
            };
            cursor += slice.len() + 1;

            match str::from_utf8(slice) {
                Ok(string) => yield Ok(string),
                Err(error) => yield Err(StringParsingError::ExpectedUtf8 { start: str_start, error })
            }
        }
    }
}


/// Try to read a null-terminated string.
/// The returned slice excludes the null termination byte.
fn try_read_c_string(start: Address, section_end: Address, program: &[u8]) -> Result<&[u8], StringParsingError> {

    let mut cursor = start;
    let null_terminator: usize;

    loop {
        if program.len() <= cursor {
            return Err(StringParsingError::Unterminated { start });
        }
        if cursor >= section_end {
            return Err(StringParsingError::OutOfSectionBounds { start, section_end });
        }

        if program[cursor] == 0 {
            null_terminator = cursor;
            break;
        }
        cursor += 1;
    }

    Ok(unsafe {
        slice::from_raw_parts(
            program.as_ptr().byte_add(start),
            null_terminator - start
        )
    })
}


pub enum SectionParsingError {
    SectionOutOfProgramBounds,
    InvalidLabelEntry { start: Address },
    InvalidLabelName { entry: Address, error: StringParsingError },
    InvalidSourceFile { entry: Address, error: StringParsingError },
    InvalidInstructionEntry { start: Address },
}

impl fmt::Display for SectionParsingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SectionParsingError::SectionOutOfProgramBounds => write!(f, "section out of program bounds"),
            SectionParsingError::InvalidLabelEntry { start } => write!(f, "invalid label entry (starts at byte {} {:#X})", *start, *start),
            SectionParsingError::InvalidLabelName { entry, error } => write!(f, "invalid label name (entry starts at byte {} {:#X}): {}", *entry, *entry, error),
            SectionParsingError::InvalidSourceFile { entry, error } => write!(f, "invalid source file (entry starts at byte {} {:#X}): {}", *entry, *entry, error),
            SectionParsingError::InvalidInstructionEntry { start } => write!(f, "invalid instruction entry (starts ar byte {} {:#X})", *start, *start),
        }
    }
}


pub fn read_labels_section<'a>(sections_table: &DebugSectionsTable, program: &'a [u8]) -> impl Iterator<Item = Result<LabelInfoView<'a>, SectionParsingError>> {
    gen {
        let mut cursor = sections_table.labels.start;

        while cursor < sections_table.labels.end {

            if program.len() <= cursor {
                yield Err(SectionParsingError::SectionOutOfProgramBounds);
                return;
            }

            let label_entry_start = cursor;
            let s = &program[label_entry_start..];
            let Some(label_info) = LabelInfo::try_parse(s) else {
                yield Err(SectionParsingError::InvalidLabelEntry { start: label_entry_start });
                return;
            };

            cursor += mem::size_of::<LabelInfo>();

            // Read the label name and source file
            //

            let label_name = match try_read_c_string(label_info.name, sections_table.label_names.end, program) {
                Ok(label_name) => label_name,
                Err(error) => {
                    yield Err(SectionParsingError::InvalidLabelName { entry: label_entry_start, error });
                    continue;
                },
            };

            let label_name = match str::from_utf8(label_name) {
                Ok(label_name) => label_name,
                Err(error) => {
                    yield Err(SectionParsingError::InvalidLabelName { entry: label_entry_start, error: StringParsingError::ExpectedUtf8 { start: label_info.name, error } });
                    continue;
                },
            };

            let source_file = match try_read_c_string(label_info.source_file, sections_table.source_files.end, program) {
                Ok(source_file) => Path::new(OsStr::from_bytes(source_file)),
                Err(error) => {
                    yield Err(SectionParsingError::InvalidSourceFile { entry: label_entry_start, error });
                    continue;
                }
            };

            yield Ok(LabelInfoView {
                name: label_name,
                address: label_info.address,
                source_file,
                source_line: label_info.source_line,
                source_column: label_info.source_column
            });
        }
    }
}


pub fn read_instructions_section<'a>(sections_table: &DebugSectionsTable, program: &'a [u8]) -> impl Iterator<Item = Result<InstructionInfoView<'a>, SectionParsingError>> {
    gen {
        let mut cursor = sections_table.instructions.start;

        while cursor < sections_table.instructions.end {

            if program.len() <= cursor {
                yield Err(SectionParsingError::SectionOutOfProgramBounds);
                return;
            }

            let instruction_entry_start = cursor;
            let s = &program[instruction_entry_start..];
            let Some(instruction_info) = InstructionInfo::try_parse(s) else {
                yield Err(SectionParsingError::InvalidInstructionEntry { start: instruction_entry_start });
                return;
            };

            cursor += mem::size_of::<InstructionInfo>();

            let source_file = match try_read_c_string(instruction_info.source_file, sections_table.source_files.end, program) {
                Ok(source_file) => Path::new(OsStr::from_bytes(source_file)),
                Err(error) => {
                    yield Err(SectionParsingError::InvalidSourceFile { entry: instruction_entry_start, error });
                    continue;
                }
            };

            yield Ok(InstructionInfoView {
                pc: instruction_info.pc,
                source_file,
                source_line: instruction_info.source_line,
                source_column: instruction_info.source_column
            });
        }
    }
}


pub struct LabelInfoView<'a> {
    pub name: &'a str,
    pub address: Address,
    pub source_file: &'a Path,
    pub source_line: usize,
    pub source_column: usize
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


    pub fn try_parse(bytes: &[u8]) -> Option<Self> {
        if bytes.len() < mem::size_of::<Self>() {
            return None;
        }

        let mut chunks = bytes[..mem::size_of::<Self>()].array_chunks::<ADDRESS_SIZE>();

        Some(Self {
            name: Address::from_le_bytes(*chunks.next()?),
            address: Address::from_le_bytes(*chunks.next()?),
            source_file: Address::from_le_bytes(*chunks.next()?),
            source_line: usize::from_le_bytes(*chunks.next()?),
            source_column: usize::from_le_bytes(*chunks.next()?)
        })
    }

}


pub struct InstructionInfoView<'a> {
    pub pc: Address,
    pub source_file: &'a Path,
    pub source_line: usize,
    pub source_column: usize,
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


    pub fn try_parse(bytes: &[u8]) -> Option<Self> {
        if bytes.len() < mem::size_of::<Self>() {
            return None;
        }

        let mut chunks = bytes[..mem::size_of::<Self>()].array_chunks::<ADDRESS_SIZE>();

        Some(Self {
            pc: Address::from_le_bytes(*chunks.next()?),
            source_file: Address::from_le_bytes(*chunks.next()?),
            source_line: usize::from_le_bytes(*chunks.next()?),
            source_column: usize::from_le_bytes(*chunks.next()?)
        })
    }

}
