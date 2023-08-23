use rust_vm_lib::assembly::{AssemblyCode, ByteCode};
use rust_vm_lib::byte_code::ByteCodes;
use rust_vm_lib::vm::{Address, ADDRESS_SIZE};

use crate::data_types::DataType;
use crate::error;
use crate::tokenizer::{tokenize_operands, is_identifier_name, is_reserved_name, is_identifier_char};
use crate::argmuments_table::get_arguments_table;
use crate::token_to_byte_code::{get_token_converter, use_converter};
use crate::files;
use crate::configs;

use std::collections::HashMap;
use std::io;
use std::path::{Path, PathBuf};


/// Maps a macro name to its definition
pub type MacroMap = HashMap<String, MacroDefinition>;


#[derive(Clone, Debug)]
/// Represents a macro definition in the assembly code
pub struct MacroDefinition {

    pub name: String,
    pub args: Vec<String>,
    pub body: AssemblyCode,
    pub unit_path: PathBuf,
    pub line_number: usize,
    pub to_export: bool,

}


impl MacroDefinition {

    pub fn new(name: String, args: Vec<String>, body: AssemblyCode, unit_path: PathBuf, line_number: usize, to_export: bool) -> MacroDefinition {
        MacroDefinition {
            name,
            args,
            body,
            unit_path,
            line_number,
            to_export,
        }
    }

}


/// Maps a contant macro name to its definition
pub type ConstMacroMap = HashMap<String, ConstMacroDefinition>;


#[derive(Clone, Debug)]
/// Represents a constant macro definition in the assembly code
pub struct ConstMacroDefinition {

    pub name: String,
    pub replace: String,
    pub unit_path: PathBuf,
    pub line_number: usize,

}


impl ConstMacroDefinition {

    pub fn new(name: String, replace: String, unit_path: PathBuf, line_number: usize) -> ConstMacroDefinition {
        ConstMacroDefinition {
            name,
            replace,
            unit_path,
            line_number,
        }
    }

}


#[derive(Clone)]
/// Represents a label declaration in the assembly code
pub struct LabelDeclaration {

    pub address: Address,
    pub unit_path: PathBuf,

}


impl LabelDeclaration {

    pub fn new(address: Address, unit_path: PathBuf) -> LabelDeclaration {
        LabelDeclaration {
            address,
            unit_path,
        }
    }

}


/// Maps a label name to its address in the binary
pub type LabelMap = HashMap<String, LabelDeclaration>;


/// Represents a label reference in the assembly code
pub struct LabelReference {

    pub name: String,
    pub location: Address,
    pub line_number: usize,

}


impl LabelReference {

    pub fn new(name: String, location: Address, line_number: usize) -> LabelReference {
        LabelReference {
            name,
            location,
            line_number,
        }
    }

}


/// Records the references to labels in the local Assembly unit
pub type LabelReferenceRegistry = Vec<LabelReference>;


pub trait AddLabelReference {

    fn add_reference(&mut self, name: String, location: Address, line_number: usize);

}


impl AddLabelReference for LabelReferenceRegistry {

    fn add_reference(&mut self, name: String, location: Address, line_number: usize) {
        self.push(LabelReference::new(name, location, line_number));
    }

}


/// Represents a section in the assembly code
enum ProgramSection {
    Data,
    Text,
    Include,
    Bss,

    /// Not really a program section, but works to tell the assembler that it's parsing a macro.
    MacroDefinition,

    None,
}


/// Information about the Assembly code sections
struct SectionInfo {

    pub has_data_section: bool,
    pub has_text_section: bool,
    pub has_include_section: bool,
    pub has_bss_section: bool,
    pub current_section: ProgramSection,

}


impl SectionInfo {

    pub fn new() -> SectionInfo {
        SectionInfo {
            has_data_section: false,
            has_text_section: false,
            has_include_section: false,
            has_bss_section: false,
            current_section: ProgramSection::None,
        }
    }

}


struct MacroInfo {

    pub current_macro: Option<MacroDefinition>,
    pub local_macros: MacroMap,
    pub export_macros: MacroMap,
    pub local_const_macros: ConstMacroMap,
    pub export_const_macros: ConstMacroMap,

}


impl MacroInfo {
    
    pub fn new() -> MacroInfo {
        MacroInfo {
            current_macro: None,
            local_macros: MacroMap::new(),
            export_macros: MacroMap::new(),
            local_const_macros: ConstMacroMap::new(),
            export_const_macros: ConstMacroMap::new(),
        }
    }

}


struct LabelInfo {

    pub local_labels: LabelMap,
    pub export_labels: LabelMap,
    pub label_references: LabelReferenceRegistry,

}


impl LabelInfo {

    pub fn new() -> LabelInfo {
        LabelInfo {
            local_labels: LabelMap::new(),
            export_labels: LabelMap::new(),
            label_references: LabelReferenceRegistry::new(),
        }
    }

}


struct ProgramInfo {

    pub included_units: ASMUnitMap,
    pub byte_code: ByteCode,

}


impl ProgramInfo {

    pub fn new() -> ProgramInfo {
        ProgramInfo {
            included_units: ASMUnitMap::new(),
            byte_code: ByteCode::new(),
        }
    }

}


type ASMUnitMap = HashMap<PathBuf, (LabelMap, MacroMap, ConstMacroMap)>;


struct AssemblyUnit<'a> {

    pub path: &'a Path,
    pub assembly: AssemblyCode,
    pub is_main_unit: bool,

}


impl AssemblyUnit<'_> {

    pub fn new(path: &Path, assembly: AssemblyCode, is_main_unit: bool) -> AssemblyUnit {
        AssemblyUnit {
            path,
            assembly,
            is_main_unit,
        }
    }

}


/// Try to load an assembly unit
/// 
/// If successful, return the assembly code and the absolute path to the unit
fn load_asm_unit(unit_name: &str, current_unit_path: &Path) -> io::Result<(PathBuf, AssemblyCode)> {

    let unit_path = Path::new(unit_name);

    if unit_path.is_absolute() {
        // The unit path is absolute, try to load it directly
        return Ok((unit_path.to_path_buf(), files::load_assembly(unit_path)?));
    }

    // The unit path is relative then

    // Try to load the unit from the standard library
    {
        let unit_path = configs::INCLUDE_LIB_PATH.join(unit_path);
        if let Ok(assembly) = files::load_assembly(&unit_path) {
            return Ok((unit_path.canonicalize().unwrap(), assembly))
        }
    }

    // Finally, try to load the unit from the current assembly unit directory

    let parent_dir = match current_unit_path.parent() {
        Some(parent_dir) => parent_dir,
        None => {
            return Err(
                io::Error::new(
                    io::ErrorKind::Other,
                    format!("Failed to get parent directory of \"{}\"", current_unit_path.display())
                )
            );
        }
    };  

    // Get the absolute path of the include unit
    let unit_path = parent_dir.join(unit_name).canonicalize()?;

    let assembly = files::load_assembly(&unit_path)?;

    Ok((unit_path, assembly))
}


/// Evaluates special compile time assembly symbols and returns the evaluated line
/// 
/// Substitutes $ symbols with the current binary address
/// 
/// Does not substitute $ symbols inside strings or character literals
/// 
/// Does not evaluate escape characters inside strings or character literals
fn evaluate_special_symbols(line: &str, current_binary_address: Address, line_number: usize, unit_path: &Path, local_const_macros: &ConstMacroMap) -> String {

    enum TextType {
        Asm,
        ConstMacro { starts_at: usize },
        String { starts_at: (usize, usize) },
        Char {starts_at: (usize, usize) },
    }

    let mut evaluated_line = String::with_capacity(line.len());

    let mut text_type = TextType::Asm;
    let mut escape_char = false;

    for (char_index, c) in line.chars().enumerate() {

        match text_type {

            TextType::Asm => {
                
                match c {
                    '$' => {
                        evaluated_line.push_str(format!("{}", current_binary_address).as_str());
                    },
                    '"' => {
                        evaluated_line.push('"');
                        text_type = TextType::String { starts_at: (line_number, char_index) };
                    },
                    '\'' => {
                        evaluated_line.push('\'');
                        text_type = TextType::Char { starts_at: (line_number, char_index) };
                    },
                    '#' => {
                        // Skip comments
                        break;
                    }
                    '=' => {
                        text_type = TextType::ConstMacro { starts_at: char_index };
                    }
                    _ => {
                        evaluated_line.push(c);
                    }
                }

            },

            TextType::String {..} => {

                evaluated_line.push(c);

                if escape_char {
                    escape_char = false;
                } else if c == '"' {
                    text_type = TextType::Asm;
                } else if c == '\\' {
                    escape_char = true;
                }
            },

            TextType::Char {..} => {

                evaluated_line.push(c);

                if escape_char {
                    escape_char = false;
                } else if c == '\'' {
                    text_type = TextType::Asm;
                } else if c == '\\' {
                    escape_char = true;
                }

            },

            TextType::ConstMacro { starts_at } => {

                if is_identifier_char(c, starts_at + 1 == char_index) {
                    continue;
                }

                // The const macro name is finished
                let const_macro_name = &line[starts_at + 1..char_index];

                let const_macro = local_const_macros.get(const_macro_name).unwrap_or_else(
                    || error::undeclared_const_macro(unit_path, const_macro_name, local_const_macros, line_number, line)
                );

                evaluated_line.push_str(const_macro.replace.as_str());

                text_type = TextType::Asm;
            },

        }
    }

    // Check for unclosed delimited literals
    match text_type {
        TextType::Asm => {
            // If the text type is Asm, then there were no unclosed strings or character literals
        },
        TextType::Char { starts_at } => {
            error::unclosed_char_literal(unit_path, starts_at.0, starts_at.1, line);
        },
        TextType::String { starts_at } => {
            error::unclosed_string_literal(unit_path, starts_at.0, starts_at.1, line);
        }
        TextType::ConstMacro { starts_at } => {

            // The const macro name is finished
            let const_macro_name = &line[starts_at + 1..];

            let const_macro = local_const_macros.get(const_macro_name).unwrap_or_else(
                || error::undeclared_const_macro(unit_path, const_macro_name, local_const_macros, line_number, line)
            );

            evaluated_line.push_str(const_macro.replace.as_str());
        }
    }

    evaluated_line
}


/// Assemble recursively an assembly unit and its dependencies
fn assemble_unit(asm_unit: AssemblyUnit, verbose: bool, program_info: &mut ProgramInfo) {

    // Check if the assembly unit has already been included
    if let Some((exported_labels, exported_macros, exported_const_macros)) = program_info.included_units.get(asm_unit.path) {
        if verbose {
            println!("Unit already included: {}", asm_unit.path.display());
            println!("Exported labels: {:?}\n", exported_labels.keys());
            println!("Exported macros: {:?}\n", exported_macros.keys());
            println!("Exported const macros: {:?}\n", exported_const_macros.keys());
        }

        program_info.included_units.insert(asm_unit.path.to_path_buf(), (exported_labels.clone(), exported_macros.clone(), exported_const_macros.clone()));
        return;
    }
    // Insert a temporary entry in the included units map to avoid infinite recursive unit inclusion
    program_info.included_units.insert(asm_unit.path.to_path_buf(), (LabelMap::new(), MacroMap::new(), ConstMacroMap::new()));


    if verbose {
        if asm_unit.is_main_unit {
            println!("\nMain assembly unit: {} ({})\n", asm_unit.path.file_name().unwrap().to_string_lossy(), asm_unit.path.display());
        } else {
            println!("\nAssembly unit: {} ({})\n", asm_unit.path.file_name().unwrap().to_string_lossy(), asm_unit.path.display());
        }
    }

    let mut label_info = LabelInfo::new();

    let mut macro_info = MacroInfo::new();

    let mut section_info = SectionInfo::new();

    for (i, line) in asm_unit.assembly.iter().enumerate() {
        let line_number = i + 1;

        if verbose {
            println!("Line {: >4}, Pos: {: >5} | {}", line_number, program_info.byte_code.len(), line);
        }

        // Evaluate the compile-time special symbols
        let evaluated_line = evaluate_special_symbols(line, program_info.byte_code.len(), line_number, asm_unit.path, &macro_info.local_const_macros);

        // Remove redundant whitespaces
        let trimmed_line = evaluated_line.trim();

        if trimmed_line.is_empty() || trimmed_line.starts_with('#') {
            // The line is either empty or a comment, skip it
            continue;
        }

        let last_byte_code_address: Address = program_info.byte_code.len();

        parse_line(trimmed_line, &mut macro_info, &asm_unit, line, line_number, program_info, &mut label_info, &mut section_info, verbose);
        
        if verbose && last_byte_code_address != program_info.byte_code.len() {
            println!(" => {:?}", &program_info.byte_code[last_byte_code_address..]);
        }
        
    }

    if matches!(section_info.current_section, ProgramSection::MacroDefinition) {
        let def = macro_info.current_macro.as_ref().unwrap_or_else(
            || panic!("Internal assembler error: current macro is None inside macro definition. This is a bug.")
        );
        error::unclosed_macro_definition(asm_unit.path, &def.name, def.line_number, asm_unit.assembly[def.line_number - 1].as_str());
    }

    // After tokenization and conversion to bytecode, substitute the labels with their real address

    for reference in label_info.label_references {

        let label = label_info.local_labels.get(&reference.name).unwrap_or_else(
            || error::undeclared_label(asm_unit.path, &reference.name, &label_info.local_labels, reference.line_number, &asm_unit.assembly[reference.line_number - 1])
        );

        // Substitute the label with the real address (little endian)
        program_info.byte_code[reference.location..reference.location + ADDRESS_SIZE].copy_from_slice(&label.address.to_le_bytes());

    }

    if verbose {
        if asm_unit.is_main_unit {
            println!("\nEnd of main assembly unit {} ({})\n", asm_unit.path.file_name().unwrap().to_string_lossy(), asm_unit.path.display());
        } else {
            println!("\nEnd of assembly unit {} ({})\n", asm_unit.path.file_name().unwrap().to_string_lossy(), asm_unit.path.display());
            println!("Exported labels: {:?}\n", label_info.export_labels.keys());
            println!("Exported macros: {:?}\n", macro_info.export_macros.keys());
            println!("Exported const macros: {:?}\n", macro_info.export_const_macros.keys());
        }
    }

    // Add the assembly unit to the included units
    program_info.included_units.insert(asm_unit.path.to_path_buf(), (label_info.export_labels, macro_info.export_macros, macro_info.export_const_macros));

}


#[allow(clippy::too_many_arguments)]
fn parse_line(trimmed_line: &str, macro_info: &mut MacroInfo, asm_unit: &AssemblyUnit, line: &str, line_number: usize, program_info: &mut ProgramInfo, label_info: &mut LabelInfo, section_info: &mut SectionInfo, verbose: bool) {

    if let Some(mut section_name) = trimmed_line.strip_prefix('.') {
        // This line specifies a program section
        section_name = section_name.strip_suffix(':').unwrap_or_else(
            || error::invalid_section_declaration(asm_unit.path, section_name, line_number, line, "Assembly sections must end with a colon.")
        );

        match section_name {

            "bss" => {
                // Check for duplicate sections
                if section_info.has_bss_section {
                    error::invalid_section_declaration(asm_unit.path, section_name, line_number, line, "An assembly unit can only have one bss section.")
                }
                section_info.current_section = ProgramSection::Bss;
                section_info.has_bss_section = true;
            }

            "include" => {
                // Check for duplicate sections
                if section_info.has_include_section {
                    error::invalid_section_declaration(asm_unit.path, section_name, line_number, line, "An assembly unit can only have one include section.")
                }
                section_info.current_section = ProgramSection::Include;
                section_info.has_include_section = true;
            },

            "data" => {
                // Check for duplicate sections
                if section_info.has_data_section {
                    error::invalid_section_declaration(asm_unit.path, section_name, line_number, line, "An assembly unit can only have one data section.")
                }
                section_info.current_section = ProgramSection::Data;
                section_info.has_data_section = true;
            },

            "text" => {
                // Check for duplicate sections
                if section_info.has_text_section {
                    error::invalid_section_declaration(asm_unit.path, section_name, line_number, line, "An assembly unit can only have one text section.")
                }
                section_info.current_section = ProgramSection::Text;
                section_info.has_text_section = true;
            },

            _ => error::invalid_section_declaration(asm_unit.path, section_name, line_number, line, format!("Unknown assembly section name: \"{}\"", section_name).as_str())
        }

        // There's nothing after the section declaration, so return
        return;
    }

    // Handle the assembly code depending on the current section
    match section_info.current_section {

        ProgramSection::MacroDefinition => {
            
            let macro_definition = macro_info.current_macro.as_mut().unwrap_or_else(
                || panic!("Internal assembler error: current macro is None inside macro definition. This is a bug.")
            );
            
            // Check for macro definition end
            if let Some(mline) = trimmed_line.strip_prefix('%') {

                if mline.trim() != "endmacro" {
                    error::invalid_macro_declaration(asm_unit.path, macro_definition.name.as_str(), line_number, line, "Macro definitions must end with \"%endmacro\".");
                }

                if macro_info.local_macros.insert(macro_definition.name.clone(), macro_definition.clone()).is_some() {
                    error::macro_redeclaration(asm_unit.path, macro_definition.name.as_str(), line_number, line);
                }

                if macro_definition.to_export {
                    macro_info.export_macros.insert(macro_definition.name.clone(), macro_info.current_macro.take().unwrap());
                }

                // Macros can only be defined inside text sections, so the section is assumed to be text
                section_info.current_section = ProgramSection::Text;

            } else {
                // The line is part of the macro body
                macro_definition.body.push(trimmed_line.to_string());
            }

        },

        ProgramSection::Include => {

            // Check if the include is to re-export (prefix with @@)
            let (include_unit_raw, to_export) = {
                if let Some(include_unit_raw) = trimmed_line.strip_prefix("@@") {
                    (include_unit_raw.trim(), true)
                } else {
                    (trimmed_line, false)
                }
            };

            let (include_path, include_asm) = match load_asm_unit(include_unit_raw, asm_unit.path) {
                Ok(x) => x,
                Err(error) => error::include_error(asm_unit.path, &error, include_unit_raw, line_number, line)
            };

            let new_asm_unit = AssemblyUnit::new(&include_path, include_asm, false);

            // Assemble the included assembly unit
            assemble_unit(new_asm_unit, verbose, program_info);
                
            let (exported_labels, exported_macros, exported_const_macros): &(LabelMap, MacroMap, ConstMacroMap) = program_info.included_units.get(&include_path).unwrap_or_else(
                || panic!("Internal assembler error: included unit not found in included units map. This is a bug.")
            );

            if to_export {
                label_info.export_labels.extend(exported_labels.clone());
                macro_info.export_macros.extend(exported_macros.clone());
                macro_info.export_const_macros.extend(exported_const_macros.clone());
            }

            label_info.local_labels.extend(exported_labels.clone());
            macro_info.local_macros.extend(exported_macros.clone());
            macro_info.local_const_macros.extend(exported_const_macros.clone());

        },

        ProgramSection::Data => {

            // Parse the static data declaration

            // Check if the data label has to be exported (double consecutive @)
            let (to_export, trimmed_line) = {
                if let Some(trimmed_line) = trimmed_line.strip_prefix("@@") {
                    // Trim the line again to remove eventual extra spaces
                    (true, trimmed_line.trim())
                } else {
                    (false, trimmed_line)
                }
            };

            // Extract the label name
            let (label, other) = trimmed_line.split_once(char::is_whitespace).unwrap_or_else(
                || error::invalid_data_declaration(asm_unit.path, line_number, line, "Static data declarations must have a label")
            );

            // Check if the label is a reserved keyword
            if is_reserved_name(label) {
                error::invalid_label_name(asm_unit.path, label, line_number, line, format!("\"{}\" is a reserved name.", label).as_str());
            }

            let (data_type_name, other) = other.split_once(char::is_whitespace).unwrap_or_else(
                || error::invalid_data_declaration(asm_unit.path, line_number, line, "Static data declarations must have a type")
            );

            let data_type = DataType::from_name(data_type_name).unwrap_or_else(
                || error::invalid_data_declaration(asm_unit.path, line_number, line, format!("Unknown data type \"{}\"", data_type_name).as_str())
            );

            // The data string is everything following the data type
            let data_string = other.trim();

            // Encode the string data into byte code
            let encoded_data: ByteCode = data_type.encode(data_string, line_number, line, asm_unit.path);

            let label_declaration = LabelDeclaration::new(program_info.byte_code.len(), asm_unit.path.to_path_buf());

            // Add the data name and its address in the binary to the data map
            label_info.local_labels.insert(label.to_string(), label_declaration.clone());

            if to_export {
                label_info.export_labels.insert(label.to_string(), label_declaration);
            }

            // Add the data to the byte code
            program_info.byte_code.extend(encoded_data);

        },

        ProgramSection::Bss => {

            // Check if the bss label has to be exported (double consecutive @)
            let (to_export, trimmed_line) = {
                if let Some(trimmed_line) = trimmed_line.strip_prefix("@@") {
                    // Trim the line again to remove eventual extra spaces
                    (true, trimmed_line.trim())
                } else {
                    (false, trimmed_line)
                }
            };

            // Extract the label name
            let (label, other) = trimmed_line.split_once(char::is_whitespace).unwrap_or_else(
                || error::invalid_bss_declaration(asm_unit.path, line_number, line, "Static BSS declarations must have a label")
            );

            // Check if the label is a reserved keyword
            if is_reserved_name(label) {
                error::invalid_label_name(asm_unit.path, label, line_number, line, format!("\"{}\" is a reserved name.", label).as_str());
            }

            let (data_type_name, other) = other.split_once(char::is_whitespace).unwrap_or_else(
                || error::invalid_bss_declaration(asm_unit.path, line_number, line, "Static BSS declarations must have a type")
            );

            let data_type = DataType::from_name(data_type_name).unwrap_or_else(
                || error::invalid_bss_declaration(asm_unit.path, line_number, line, format!("Unknown data type \"{}\"", data_type_name).as_str())
            );

            let data_size = data_type.size().unwrap_or_else(
                || error::invalid_bss_declaration(asm_unit.path, line_number, line, format!("Static size for {} is unknown", data_type_name).as_str())
            );

            if other.trim() != "" {
                error::invalid_bss_declaration(asm_unit.path, line_number, line, "Static BSS declarations cannot have anything after the data type");
            }

            let label_declaration = LabelDeclaration::new(program_info.byte_code.len(), asm_unit.path.to_path_buf());

            // Add the data name and its address in the binary to the data map
            label_info.local_labels.insert(label.to_string(), label_declaration.clone());

            if to_export {
                label_info.export_labels.insert(label.to_string(), label_declaration);
            }

            // Add a data placeholder to the byte code
            // Doesn't take advantage of the .bss section optimization, though
            program_info.byte_code.extend(vec![0; data_size]);

        },

        ProgramSection::Text => {

            // Check for label declarations
            if let Some(label) = trimmed_line.strip_prefix('@') {

                // Check if the label is to be exported (double consecutive @)
                let (label, to_export): (&str, bool) = {
                    if let Some(label) = label.strip_prefix('@') {
                        (label.trim(), true)
                    } else {
                        (label.trim(), false)
                    }
                };
                
                if !is_identifier_name(label) {
                    error::invalid_label_name(asm_unit.path, label, line_number, line, "Label names can only contain alphabetic characters, numbers (except for the first character), and underscores.");
                }

                if is_reserved_name(label) {
                    error::invalid_label_name(asm_unit.path, label, line_number, line, format!("\"{}\" is a reserved name.", label).as_str());
                }

                let label_declaration = LabelDeclaration::new(program_info.byte_code.len(), asm_unit.path.to_path_buf());

                if asm_unit.is_main_unit && label == "start" || to_export {
                    label_info.export_labels.insert(label.to_string(), label_declaration.clone());
                }

                if label_info.local_labels.insert(label.to_string(), label_declaration).is_some() {
                    error::label_redeclaration(asm_unit.path, label, line_number, line);
                }

                return;
            }


            // Check for macro declaration
            if let Some(macro_declaration) = trimmed_line.strip_prefix('%') {

                // Check if the macro is to be exported (double consecutive %)
                let (macro_declaration, to_export): (&str, bool) = {
                    if let Some(macro_declaration) = macro_declaration.strip_prefix('%') {
                        (macro_declaration.trim_start(), true)
                    } else {
                        (macro_declaration.trim_start(), false)
                    }
                };

                // Check if this is a const macro declaration
                let (macro_declaration, is_const_macro): (&str, bool) = {
                    if let Some(macro_declaration) = macro_declaration.strip_prefix('-') {
                        (macro_declaration.trim_start(), true)
                    } else {
                        (macro_declaration.trim_start(), false)
                    }
                };

                let (macro_name, post) = macro_declaration.split_once(
                    |c: char| c.is_whitespace() || c == ':'
                ).unwrap_or_else(
                    || error::invalid_macro_declaration(asm_unit.path, "", line_number, line, "Macro declarations must have a name")
                );

                if !is_identifier_name(macro_name) {
                    error::invalid_macro_declaration(asm_unit.path, macro_name, line_number, line, "Macro names can only contain alphabetic characters, numbers (except for the first character), and underscores.");
                }

                if is_reserved_name(macro_name) {
                    error::invalid_macro_declaration(asm_unit.path, macro_name, line_number, line, format!("\"{}\" is a reserved name.", macro_name).as_str());
                }

                // Constant macros don't accept arguments
                if is_const_macro {

                    let replace = post.trim_start();

                    if replace.is_empty() {
                        error::invalid_macro_declaration(asm_unit.path, macro_name, line_number, line, "Constant macros must have a replacement text value");
                    }

                    let macro_declaration = ConstMacroDefinition::new(
                        macro_name.to_string(), 
                        replace.to_string(), 
                        asm_unit.path.to_path_buf(), 
                        line_number
                    );

                    if macro_info.local_const_macros.insert(macro_name.to_string(), macro_declaration.clone()).is_some() {
                        error::macro_redeclaration(asm_unit.path, macro_name, line_number, line);
                    }

                    if to_export {
                        macro_info.export_const_macros.insert(macro_name.to_string(), macro_declaration);
                    }

                    return;
                }

                let post = post.trim();
                let mut macro_args = Vec::new();
                if let Some(post) = post.strip_prefix(':') {
                    // The macro declaration is finished

                    if !post.is_empty() {
                        error::invalid_macro_declaration(asm_unit.path, macro_name, line_number, line, "Cannot have anything after the colon in a macro declaration");
                    }

                } else {
                    // The macro declaration has arguments, parse them

                    macro_args.extend(
                        post.split(|c: char| c.is_whitespace() || c == ':')
                        .map(|s| s.trim())
                        .filter(|s| !s.is_empty())
                        .map(|arg_name| {

                            if !is_identifier_name(arg_name) {
                                error::invalid_macro_declaration(asm_unit.path, macro_name, line_number, line, format!("Macro argument names can only contain alphabetic characters, numbers (except for the first character), and underscores.\nInvalid argument name: \"{}\"", arg_name).as_str());
                            }

                            if is_reserved_name(arg_name) {
                                error::invalid_macro_declaration(asm_unit.path, macro_name, line_number, line, format!("\"{}\" is a reserved name.", arg_name).as_str());
                            }

                            arg_name.to_string()
                        })
                    );                     

                }

                macro_info.current_macro = Some(MacroDefinition::new(
                    macro_name.to_string(),
                    macro_args,
                    AssemblyCode::new(),
                    asm_unit.path.to_path_buf(),
                    line_number,
                    to_export
                ));

                section_info.current_section = ProgramSection::MacroDefinition;

                return;
            }


            // Check for macro call (starts with !)
            if let Some(macro_call) = trimmed_line.strip_prefix('!').map(|s| s.trim()) {

                let (macro_name, post) = macro_call.split_once(char::is_whitespace).unwrap_or((macro_call, ""));

                if macro_name.is_empty() {
                    error::invalid_macro_call(asm_unit.path, line_number, line, "Macro calls must have a name");
                }

                // Get the macro definition
                let def = macro_info.local_macros.get(macro_name).unwrap_or_else(
                    || error::undeclared_macro(asm_unit.path, macro_name, &macro_info.local_macros, line_number, line)
                );

                // Get the arguments of the macro call
                let macro_args: Vec<&str> = post.split_whitespace()
                    .map(|s| s.trim())
                    .filter(|s| !s.is_empty())
                    .collect();             
                
                if macro_args.len() != def.args.len() {
                    error::invalid_macro_call(asm_unit.path, line_number, line, format!("Macro \"{}\" ({}, {}) expects {} arguments, but {} were given.", macro_name, def.unit_path.display(), def.line_number, def.args.len(), macro_args.len()).as_str());
                }

                // Replace the macro argument placeholders in the macro body with the macro call arguments
                let mut mlines: Vec<String> = def.body.clone();
                for mline in mlines.iter_mut() {
                
                    // Not very efficient, but works
                    for (i, arg) in def.args.iter().enumerate() {
                        *mline = mline.replace(format!("{{{}}}", arg).as_str(), macro_args[i]);
                    }

                }

                // Parse the macro body
                for mline in mlines {

                    if verbose {
                        println!("Macro {: >3}, Pos: {: >5} | {}", line_number, program_info.byte_code.len(), mline);
                    }

                    parse_line(&mline, macro_info, asm_unit, line, line_number, program_info, label_info, section_info, verbose);

                }

                return;
            }


            // The line doesn't contain label declarations, macros, or special operators, so it's an instruction
            assemble_instruction(asm_unit, trimmed_line, line, line_number, &mut program_info.byte_code, &mut label_info.label_references);

        },

        // Code cannot be put outside of a program section
        ProgramSection::None => error::out_of_section(asm_unit.path, line_number, line)

    }

}


#[derive(Clone, Copy)]
/// Enum representing assembly pseudo instructions
enum PseudoInstruction {

    DefineData = 0,

}


fn get_pseudo_instruction(name: &str) -> Option<PseudoInstruction> {

    match name {

        "dd" => Some(PseudoInstruction::DefineData),

        _ => None
    }
}


#[inline(always)]
fn handle_pseudo_instruction(pi: PseudoInstruction, asm_unit: &AssemblyUnit, args: &str, line: &str, line_number: usize, byte_code: &mut ByteCode) {

    match pi {

        PseudoInstruction::DefineData => {

            let (data_type_name, other) = args.split_once(char::is_whitespace).unwrap_or_else(
                || error::invalid_data_declaration(asm_unit.path, line_number, line, "Static data declarations must have a type")
            );

            let data_type = DataType::from_name(data_type_name).unwrap_or_else(
                || error::invalid_data_declaration(asm_unit.path, line_number, line, format!("Unknown data type \"{}\"", data_type_name).as_str())
            );

            // The data string is everything following the data type
            let data_string = other.trim();

            // Encode the string data into byte code
            let encoded_data: ByteCode = data_type.encode(data_string, line_number, line, asm_unit.path);

            // Add the data to the byte code
            byte_code.extend(encoded_data);
        }

    }

}


/// Assemble an instruction into byte code
fn assemble_instruction(asm_unit: &AssemblyUnit, trimmed_line: &str, line: &str, line_number: usize, byte_code: &mut ByteCode, label_reference_registry: &mut LabelReferenceRegistry) {

    // Split the operator from its arguments
    let (operator_name, raw_tokens): (&str, &str) = trimmed_line.split_once(char::is_whitespace).unwrap_or((
        trimmed_line, ""
    ));


    // Check for pseudo instructions
    if let Some(pi) = get_pseudo_instruction(operator_name) {
        handle_pseudo_instruction(pi, asm_unit, raw_tokens.trim_start(), line, line_number, byte_code);
        return;
    }

    // The instruction is a regular instruction

    let operands = tokenize_operands(raw_tokens, line_number, line, asm_unit.path);
    
    let arg_table = get_arguments_table(operator_name).unwrap_or_else(
        || error::invalid_instruction_name(asm_unit.path, operator_name, line_number, line)
    );

    let operation = arg_table.get_operation(operator_name, &operands, asm_unit.path, line_number, line);

    // Convert the operands to byte code and append them to the byte code
    let converter = get_token_converter(operation.instruction);

    // Add the instruction code to the byte code
    byte_code.push(operation.instruction as u8);

    // Add the operands to the byte code
    let operand_bytes = use_converter(converter, operands, operation.handled_size, label_reference_registry, byte_code.len(), line_number, asm_unit.path, line);
    
    if operand_bytes.len() != operation.total_arg_size as usize {
        panic!("The generated operand byte code size {} for instruction \"{}\" does not match the expected size {}. This is a bug.", operand_bytes.len(), operation.instruction, operation.total_arg_size);
    }

    byte_code.extend(operand_bytes);

}


/// Assembles the assembly code into byte code
pub fn assemble(assembly: AssemblyCode, verbose: bool, unit_path: &Path, just_check: bool) -> ByteCode {

    let mut program_info = ProgramInfo::new();

    let asm_unit = AssemblyUnit::new(unit_path, assembly, true);

    // Assemble recursively the main assembly unit and its dependencies
    assemble_unit(asm_unit, verbose, &mut program_info);

    if just_check {
        std::process::exit(0);
    }

    // Append the exit instruction to the end of the binary
    program_info.byte_code.push(ByteCodes::EXIT as u8);

    // Append the address of the program start to the end of the binary

    let (label_map, _macro_map, _const_macro_map) = program_info.included_units.get(unit_path).unwrap_or_else(
        || panic!("Internal assembler error: main assembly unit not found in included units map. This is a bug.")
    );

    let program_start = label_map.get("start").unwrap_or_else(
        || error::undeclared_label(unit_path, "start", label_map, 0, "The program must have a start label.")
    );

    program_info.byte_code.extend(program_start.address.to_le_bytes());

    if verbose {
        println!("Byte code size is {} bytes", program_info.byte_code.len());
        println!("Start address is {}", program_start.address);
    }

    program_info.byte_code
}

