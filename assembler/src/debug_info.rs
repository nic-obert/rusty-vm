use std::collections::HashMap;
use std::mem;
use std::os::unix::ffi::OsStrExt;
use std::range::Range;
use std::rc::Rc;

use rusty_vm_lib::debug::{self, DebugSectionsTable};
use rusty_vm_lib::vm::Address;
use rusty_vm_lib::assembly::{SourceToken, UnitPath};


pub struct LabelInfo<'a> {
    pub address: Address,
    pub name: &'a str,
    pub source: Rc<SourceToken<'a>>,
}


pub struct InstructionInfo<'a> {
    pub address: Address,
    pub source: Rc<SourceToken<'a>>,
}


pub struct DebugInfoTable<'a> {
    labels: Vec<LabelInfo<'a>>,
    instructions: Vec<InstructionInfo<'a>>,
    source_files: Vec<UnitPath<'a>>,
}

impl<'a> DebugInfoTable<'a> {

    pub fn new() -> Self {
        Self {
            labels: Default::default(),
            instructions: Default::default(),
            source_files: Default::default(),
        }
    }


    pub fn add_label(&mut self, label: LabelInfo<'a>) {
        self.labels.push(label);
    }


    pub fn add_instruction(&mut self, instruction: InstructionInfo<'a>) {
        self.instructions.push(instruction);
    }


    pub fn add_source_file(&mut self, unit_path: UnitPath<'a>) {
        self.source_files.push(unit_path);
    }


    pub fn generate_sections(&self, buf: &mut Vec<u8>) -> DebugSectionsTable {

        let mut label_names_table: HashMap<&str, Address> = HashMap::new();
        let mut source_files_table: HashMap<UnitPath, Address> = HashMap::new();

        // Write the label names section
        //
        let label_names_section_start = buf.len();

        for label in &self.labels {
            // Don't insert duplicate strings
            if !label_names_table.contains_key(label.name) {
                label_names_table.insert(label.name, buf.len());
                buf.extend(label.name.as_bytes());
                buf.push(0);
            }
        }

        let label_names_section_end = buf.len();

        // Write the source file paths section
        //
        let source_files_section_start = buf.len();

        for unit_path in &self.source_files {
            // There should be no duplicate unit paths
            source_files_table.insert(*unit_path, buf.len());
            buf.extend(unit_path.as_path().as_os_str().as_bytes());
            buf.push(0);
        }

        let source_files_section_end = buf.len();

        // Reserve some capacity for the known-sized sections
        buf.reserve(
            mem::size_of::<debug::LabelInfo>() * self.labels.len()
            + mem::size_of::<debug::InstructionInfo>() * self.instructions.len()
        );

        // Write the labels section
        //
        let labels_section_start = buf.len();

        for label in &self.labels {
            let label_info = debug::LabelInfo {
                name: *label_names_table.get(label.name).unwrap(),
                address: label.address,
                source_file: *source_files_table.get(&label.source.unit_path).unwrap(),
                source_line: label.source.line_number(),
                source_column: label.source.column
            };
            label_info.write(buf);
        }

        let labels_section_end = buf.len();

        // Write the instructions section
        //
        let instructions_section_start = buf.len();

        for instruction in &self.instructions {
            let instruction_info = debug::InstructionInfo {
                pc: instruction.address,
                source_file: *source_files_table.get(&instruction.source.unit_path).unwrap(),
                source_line: instruction.source.line_number(),
                source_column: instruction.source.column
            };
            instruction_info.write(buf);
        }

        let instructions_section_end = buf.len();


        DebugSectionsTable {
            label_names: Range { start: label_names_section_start, end: label_names_section_end },
            source_files: Range { start: source_files_section_start, end: source_files_section_end },
            labels: Range { start: labels_section_start, end: labels_section_end },
            instructions: Range { start: instructions_section_start, end: instructions_section_end }
        }
    }

}
