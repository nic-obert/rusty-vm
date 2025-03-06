use core::range::Range;
use std::rc::Rc;

use rfd::FileDialog;
use slint::CloseRequestResponse;

use crate::debugger::Debugger;


slint::include_modules!();


pub fn run_ui(debugger: Rc<Debugger>) -> Result<(), slint::PlatformError> {
    let main_window = MainWindow::new()?;

    create_ui(&main_window, debugger);

    main_window.run()
}


fn create_ui(main_window: &MainWindow, debugger: Rc<Debugger>) {

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
    main_window.global::<Backend>().on_stop(move || {
        debugger_ref.stop_vm();
    });

    let debugger_ref = Rc::clone(&debugger);
    main_window.global::<Backend>().on_continue(move || {
        debugger_ref.resume_vm();
        // TODO: deal with eventual breakpoints when they're implemented
    });

    let debugger_ref = Rc::clone(&debugger);
    main_window.window().on_close_requested(move|| {
        debugger_ref.close();
        CloseRequestResponse::HideWindow
    });
}


fn generate_memory_strings(memory: &[u8], range: Range<usize>) -> Option<(String, String)> {

    const BYTES_PER_ROW: usize = 16;

    if memory.len() < range.end {
        return None;
    }

    let mem_view = &memory[range.start..range.end];

    let line_count = mem_view.len() / BYTES_PER_ROW + (mem_view.len() % BYTES_PER_ROW != 0) as usize;

    // TODO: pre-allocate the string
    let mut lines_str = String::new();
    // The *2 considers a space or newline character after every byte
    let mut mem_str = String::with_capacity(line_count * BYTES_PER_ROW * 2);
    let mut row_index: usize = 0;

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
