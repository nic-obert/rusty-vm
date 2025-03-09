use core::range::Range;
use std::rc::Rc;

use rfd::FileDialog;
use rusty_vm_lib::registers::CPURegisters;
use slint::{CloseRequestResponse, Model, ModelRc, SharedString, SharedVector, ToSharedString, VecModel};

use crate::debugger::Debugger;


slint::include_modules!();


pub fn run_ui(debugger: Rc<Debugger>) -> Result<(), slint::PlatformError> {
    let main_window = MainWindow::new()?;

    create_ui(&main_window, debugger);

    main_window.run()
}


fn create_ui(main_window: &MainWindow, debugger: Rc<Debugger>) {

    // let reg_sets: Rc<VecModel<SharedVector<SharedString>>> = Rc::new(VecModel::from(vec![]));
    // let reg_sets = ModelRc::from(reg_sets.clone());
    // main_window.set_register_sets(reg_sets);

    let debugger_ref = Rc::clone(&debugger);
    main_window.global::<Backend>().on_dump_core(move || {

        let pick_file = || {
            FileDialog::new()
                .set_title("Dump core to file")
                .save_file()
        };

        debugger_ref.dump_core(pick_file);
    });

    let debugger_ref = Rc::clone(&debugger);
    let window_weak = main_window.as_weak();
    main_window.global::<Backend>().on_stop(move || {
        let window = window_weak.unwrap();
        debugger_ref.stop_vm();
        let current_regs = debugger_ref.read_registers();
        let reg_sets: ModelRc<ModelRc<SharedString>> = window.get_register_sets();
        let vec_model = reg_sets.as_any().downcast_ref::<VecModel<VecModel<SharedString>>>().unwrap();

    });

    let debugger_ref = Rc::clone(&debugger);
    main_window.global::<Backend>().on_continue(move || {
        debugger_ref.resume_vm();
        // TODO: deal with eventual breakpoints when they're implemented
    });

    let debugger_ref = Rc::clone(&debugger);
    main_window.window().on_close_requested(move || {
        debugger_ref.close();
        CloseRequestResponse::HideWindow
    });

    let debugger_ref = Rc::clone(&debugger);
    let window_weak = main_window.as_weak();
    main_window.global::<Backend>().on_memory_start_changed(move || {
        let window = window_weak.unwrap();
        memory_view_changed(&window, &debugger_ref);
    });

    let debugger_ref = Rc::clone(&debugger);
    let window_weak = main_window.as_weak();
    main_window.global::<Backend>().on_memory_span_changed(move || {
        let window = window_weak.unwrap();
        memory_view_changed(&window, &debugger_ref);
    });

}


fn memory_view_changed(window: &MainWindow, debugger: &Debugger) {
    let mem_start = window.get_memory_start();
    let mem_span = window.get_memory_span();

    let mem_start = usize::from_str_radix(mem_start.as_str(), 16);
    let mem_span = usize::from_str_radix(mem_span.as_str(), 16);

    window.set_memory_span_valid(mem_span.is_ok());
    window.set_memory_start_valid(mem_start.is_ok());

    if let (Ok(mem_start), Ok(mem_span)) = (mem_start, mem_span) {
        let vm_mem = debugger.read_vm_memory();
        let mem_range = Range { start: mem_start, end: mem_start + mem_span };
        let (lines_str, mem_str) = {
            if let Some((lines_str, mem_str)) = generate_memory_strings(vm_mem, mem_range) {
                (lines_str.to_shared_string(), mem_str.to_shared_string())
            } else {
                let string = SharedString::from("Memory range outside bounds");
                (string.clone(), string)
            }
        };
        window.set_memory_lines(lines_str);
        window.set_memory_view(mem_str);
    }
}


fn generate_registers_strings(regs: &CPURegisters) -> () {

}


fn generate_memory_strings(memory: &[u8], mut range: Range<usize>) -> Option<(String, String)> {
    // TODO: optimize this to reuse the old string buffers when they're the same size.
    // It seems however to be fast enough, for now.

    const BYTES_PER_ROW: usize = 16;

    if memory.len() < range.end {
        range.end = memory.len();
        if range.start > range.end {
            return None;
        }
    }

    let mem_view = &memory[range.start..range.end];

    let line_count = mem_view.len() / BYTES_PER_ROW + (mem_view.len() % BYTES_PER_ROW != 0) as usize;

    // TODO: pre-allocate the string
    let mut lines_str = String::new();
    // The *2 considers a space or newline character after every byte
    let mut mem_str = String::with_capacity(line_count * BYTES_PER_ROW * 2);
    let mut row_index: usize = range.start;

    let mut mem_rows = mem_view.array_chunks::<BYTES_PER_ROW>();
    for full_row in mem_rows.by_ref() {
        for byte in &full_row[..BYTES_PER_ROW-1] {
            mem_str.push_str(format!("{:02X}", *byte).as_str());
            mem_str.push(' ');
        }
        mem_str.push_str(format!("{:02X}", full_row[BYTES_PER_ROW-1]).as_str());
        mem_str.push('\n');
        lines_str.push_str(format!("{:#X}\n", row_index).as_str());
        row_index += BYTES_PER_ROW;
    }

    let remainder_row = mem_rows.remainder();
    if !remainder_row.is_empty() {
        lines_str.push_str(format!("{:#X}\n", row_index).as_str());
        for byte in remainder_row {
            mem_str.push_str(format!("{:02X}", *byte).as_str());
            mem_str.push(' ');
        }
    }

    Some((
        lines_str,
        mem_str
    ))
}
