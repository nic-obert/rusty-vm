use std::rc::Rc;

use slint::{ModelRc, SharedString, ToSharedString, VecModel};

use rusty_vm_lib::debug;
use rusty_vm_lib::vm::Address;
use rusty_vm_lib::assembly;
use rusty_vm_lib::byte_code::{ByteCodes, OPCODE_SIZE};

use crate::backend::debugger::Debugger;

slint::include_modules!();


pub fn run_ui(debugger: &Debugger) -> Result<(), slint::PlatformError> {
    let main_window: DebugInfoViewerMainWindow = DebugInfoViewerMainWindow::new()?;

    let success = create_ui(&main_window, debugger);

    main_window.set_error(success.is_err());

    main_window.show()
}


fn create_ui(main_window: &DebugInfoViewerMainWindow, debugger: &Debugger) -> Result<(), ()> {

    let vm_mem = debugger.read_vm_memory();

    let Ok(sections_table) = debug::DebugSectionsTable::try_parse(vm_mem) else {
        return Err(())
    };

    let label_names = debug::read_label_names_section(&sections_table, vm_mem);
    let label_names_model: Rc<VecModel<SharedString>> = Rc::new(VecModel::default());
    for label in label_names {
        let Ok(label) = label else { return Err(()) };
        label_names_model.push(label.to_shared_string());
    }
    main_window.set_label_names(ModelRc::from(label_names_model));

    let source_files = debug::read_source_files_section(&sections_table, vm_mem);
    let source_files_model: Rc<VecModel<SharedString>> = Rc::new(VecModel::default());
    for source_file in source_files {
        let Ok(path) = source_file else { return Err(()) };
        source_files_model.push(path.display().to_shared_string());
    }
    main_window.set_source_files(ModelRc::from(source_files_model));

    let labels = debug::read_labels_section(&sections_table, vm_mem);
    let labels_model: Rc<VecModel<UILabelView>> = Rc::new(VecModel::default());
    for label in labels {
        let Ok(label) = label else { return Err(()) };
        let ui_label = UILabelView {
            address: format!("{:#X}", label.address).to_shared_string(),
            name: label.name.to_shared_string(),
            source_file: label.source_file.display().to_shared_string(),
            source_line: label.source_line.to_shared_string(),
            source_column: label.source_column.to_shared_string()
        };
        labels_model.push(ui_label);
    }
    main_window.set_labels(ModelRc::from(labels_model));

    let instructions = debug::read_instructions_section(&sections_table, vm_mem);
    let instructions_model: Rc<VecModel<UIInstructionView>> = Rc::new(VecModel::default());
    for instruction in instructions {
        let Ok(instruction) = instruction else { return Err(()) };
        let ui_instruction = UIInstructionView {
            disassembly: disassemble_instruction_at(vm_mem, instruction.pc).to_shared_string(),
            pc: format!("{:#X}", instruction.pc).to_shared_string(),
            source_file: instruction.source_file.display().to_shared_string(),
            source_line: instruction.source_line.to_shared_string(),
            source_column: instruction.source_column.to_shared_string()
        };
        instructions_model.push(ui_instruction);
    }
    main_window.set_instructions(ModelRc::from(instructions_model));

    Ok(())
}


fn disassemble_instruction_at(program: &[u8], pc: Address) -> String {

    let operator = ByteCodes::from(program[pc]);

    let (handled_size, args) = assembly::parse_bytecode_args(operator, &program[pc+OPCODE_SIZE..])
        .unwrap_or_else(|err| panic!("Could not parse arguments for opcode {operator}:\n{err}"));

    let mut disassembly = if handled_size != 0 {
        format!("{operator} ({handled_size})")
    } else {
        format!("{operator}")
    };
    for arg in args {
        disassembly.push(' ');
        disassembly.push_str(arg.to_string().as_str());
    }

    disassembly
}
