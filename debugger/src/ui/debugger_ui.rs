use std::fmt::Write;
use core::range::Range;
use std::rc::Rc;
use std::sync::{Arc, RwLock};
use std::thread;
use std::time::Duration;

use rfd::FileDialog;
use rusty_vm_lib::debug::DEBUGGER_COMMAND_WAIT_SLEEP;
use slint::{CloseRequestResponse, Model, ModelRc, SharedString, ToSharedString, VecModel};

use rusty_vm_lib::registers::REGISTER_SIZE;

use super::debug_info_viewer_ui;
use super::queue_model::QueueModel;
use crate::backend::debugger::Debugger;


slint::include_modules!();


/// How many bytes to show for every row in the memory inspector area.
const BYTES_PER_MEMORY_ROW: usize = 16;
/// How many register sets to keep in the registers area.
const REGISTERS_HISTORY_LIMIT: usize = 10;
/// How many disassembled instruction lines to keep in the instructions area.
const INSTRUCTIONS_DISASSEMBLY_HISTORY_LIMIT: usize = 10;


pub fn run_ui(debugger: Arc<RwLock<Debugger>>) -> Result<(), slint::PlatformError> {
    let main_window: DebuggerMainWindow = DebuggerMainWindow::new()?;

    create_ui(&main_window, Arc::clone(&debugger));

    // Regularly check if the VM process was terminated and notify the UI accordingly
    //
    let debugger_ref = Arc::clone(&debugger);
    let window_weak = main_window.as_weak();
    thread::spawn(move || {
        const CHECK_FOR_TERMINATED_VM_SLEEP: Duration = Duration::from_millis(300);
        let debugger = debugger_ref;

        while !debugger.read().unwrap().is_terminated() {
            thread::sleep(CHECK_FOR_TERMINATED_VM_SLEEP);
        }

        // Cannot just upgrade the weak reference to the window from the main thread.
        window_weak.upgrade_in_event_loop(|window| {
            window.global::<Backend>().set_terminated(true);
        }).unwrap();
    });

    // Regularly check if the VM is stopped and notify the UI accordingly
    //
    let window_weak = main_window.as_weak();
    thread::spawn(move || {

        // Continue checking until the VM process is terminated
        while !debugger.read().unwrap().is_terminated() {

            // Wait until the VM is stopped
            while debugger.read().unwrap().is_running() {
                thread::sleep(DEBUGGER_COMMAND_WAIT_SLEEP);
            }

            // Cannot just upgrade the weak reference to the window from the main thread.
            let debugger_moved = Arc::clone(&debugger);
            window_weak.upgrade_in_event_loop(move |window| {
                window.global::<Backend>().set_running(false);
                update_all_views(&window, &debugger_moved.read().unwrap());
            }).unwrap();

            // Don't continue updating the UI until the VM is resumed again.
            // While the VM is stopped, its state won't change.
            while {
                let debugger = debugger.read().unwrap();
                !debugger.is_running() && !debugger.is_terminated()
            } {
                thread::sleep(DEBUGGER_COMMAND_WAIT_SLEEP);
            }
        }
    });

    main_window.run()
}


fn create_ui(main_window: &DebuggerMainWindow, debugger: Arc<RwLock<Debugger>>) {

    // Initialize the models and views
    //
    initialize_all_views(main_window, &debugger.read().unwrap());

    // Bind UI to backend functionality
    //
    let debugger_ref = Arc::clone(&debugger);
    main_window.global::<Backend>().on_dump_core(move || {

        let pick_file = || {
            FileDialog::new()
                .set_title("Dump core to file")
                .save_file()
        };

        debugger_ref.read().unwrap().dump_core(pick_file);
    });

    let debugger_ref = Arc::clone(&debugger);
    main_window.global::<Backend>().on_view_debug_info(move || {
        debug_info_viewer_ui::run_ui(&debugger_ref.read().unwrap()).expect("Failed to run debug info viewer");
    });

    let debugger_ref = Arc::clone(&debugger);
    let window_weak = main_window.as_weak();
    main_window.global::<Backend>().on_stop(move || {
        let window = window_weak.unwrap();
        let debugger = debugger_ref.read().unwrap();
        debugger.stop_vm();
        update_all_views(&window, &debugger);
    });

    let debugger_ref = Arc::clone(&debugger);
    let window_weak = main_window.as_weak();
    main_window.global::<Backend>().on_continue(move || {
        let window = window_weak.unwrap();
        debugger_ref.write().unwrap().continue_vm();
        update_vm_status_view(&window, &debugger_ref.read().unwrap());
    });

    let debugger_ref = Arc::clone(&debugger);
    let window_weak = main_window.as_weak();
    main_window.global::<Backend>().on_step_in(move || {
        let window = window_weak.unwrap();
        let mut debugger = debugger_ref.write().unwrap();
        let disassembly = debugger.step_in();
        add_disassembly_line(&window, disassembly);
        update_all_views(&window, &debugger);
    });

    let debugger_ref = Arc::clone(&debugger);
    main_window.window().on_close_requested(move || {
        debugger_ref.read().unwrap().close();
        CloseRequestResponse::HideWindow
    });

    let debugger_ref = Arc::clone(&debugger);
    let window_weak = main_window.as_weak();
    main_window.global::<Backend>().on_memory_start_changed(move || {
        let window = window_weak.unwrap();
        update_memory_view(&window, &debugger_ref.read().unwrap());
    });

    let debugger_ref = Arc::clone(&debugger);
    let window_weak = main_window.as_weak();
    main_window.global::<Backend>().on_memory_span_changed(move || {
        let window = window_weak.unwrap();
        update_memory_view(&window, &debugger_ref.read().unwrap());
    });

    let debugger_ref = Arc::clone(&debugger);
    let window_weak = main_window.as_weak();
    main_window.global::<Backend>().on_memory_refresh(move || {
        let window = window_weak.unwrap();
        update_memory_view(&window, &debugger_ref.read().unwrap());
    });

    let debugger_ref = Arc::clone(&debugger);
    let window_weak = main_window.as_weak();
    main_window.global::<Backend>().on_add_breakpoint_here(move || {
        let window = window_weak.unwrap();
        let mut debugger = debugger_ref.write().unwrap();
        debugger.add_persistent_breakpoint_at_pc().unwrap();
        update_breakpoint_view(&window, &debugger);
        todo!()
    });

}


fn initialize_all_views(window: &DebuggerMainWindow, debugger: &Debugger) {
    initialize_vm_status_view(window, debugger);
    initialize_breakpoint_view(window);
    initialize_registers_view(window);
    initialize_instructions_view(window);
}


fn initialize_breakpoint_view(window: &DebuggerMainWindow) {
    let breakpoints: Rc<VecModel<BreakPoint>> = Rc::new(VecModel::default());
    let breakpoints_model = ModelRc::from(breakpoints);
    window.set_breakpoints(breakpoints_model);
}


fn initialize_registers_view(window: &DebuggerMainWindow) {
    let regs: Rc<QueueModel<ModelRc<SharedString>>> = Rc::new(QueueModel::default());
    let regs_model = ModelRc::from(regs);
    window.set_register_sets(regs_model);
}


fn update_all_views(window: &DebuggerMainWindow, debugger: &Debugger) {
    update_vm_status_view(window, debugger);
    update_memory_view(window, debugger);
    update_register_view(window, debugger);
    update_breakpoint_view(window, debugger);
}


fn initialize_instructions_view(window: &DebuggerMainWindow) {
    let instructions: Rc<QueueModel<SharedString>> = Rc::new(QueueModel::default());
    let instructions_model = ModelRc::from(instructions);
    window.set_instructions_disassembly(instructions_model);
}


fn initialize_vm_status_view(window: &DebuggerMainWindow, debugger: &Debugger) {
    window.global::<Backend>().set_running(debugger.is_running());
    window.global::<Backend>().set_total_memory(debugger.vm_memory_size().to_shared_string());
}

fn update_vm_status_view(window: &DebuggerMainWindow, debugger: &Debugger) {
    window.global::<Backend>().set_running(debugger.is_running());
}


fn update_breakpoint_view(window: &DebuggerMainWindow, debugger: &Debugger) {
    let breakpoints = debugger.breakpoint_table().breakpoints();
    let bp_model = window.get_breakpoints();
    let bp_vec = bp_model.as_any().downcast_ref::<VecModel<BreakPoint>>().unwrap();

    bp_vec.clear();

    for bp in breakpoints {
        let bp_view_model = BreakPoint {
            location: format!("{:X}", bp.location).to_shared_string(),
            name: bp.name.as_ref().map_or(SharedString::new(), |s| s.clone()),
            replaced_value: bp.replaced_value as i32
        };
        bp_vec.push(bp_view_model);
    }
}


fn add_disassembly_line(window: &DebuggerMainWindow, new_instruction: SharedString) {
    let instructions_model: ModelRc<SharedString> = window.get_instructions_disassembly();
    let instructions_queue = instructions_model.as_any().downcast_ref::<QueueModel<SharedString>>().unwrap();

    if instructions_queue.len() >= INSTRUCTIONS_DISASSEMBLY_HISTORY_LIMIT-1 {
        instructions_queue.pop_front();
    }
    instructions_queue.push_back(new_instruction);

    // Tell the UI to update the view (autoscroll feature)
    window.invoke_scroll_to_bottom_instructions_view();
}


fn update_register_view(window: &DebuggerMainWindow, debugger: &Debugger) {
    let current_regs = debugger.read_registers();
    let reg_sets_model: ModelRc<ModelRc<SharedString>> = window.get_register_sets();
    let reg_sets_queue = reg_sets_model.as_any().downcast_ref::<QueueModel<ModelRc<SharedString>>>().unwrap();

    let new_reg_set: VecModel<SharedString> = current_regs
        .as_bytes()
        .as_chunks::<REGISTER_SIZE>()
        .0
        .iter()
        .map(|bytes| usize::from_le_bytes(*bytes).to_shared_string())
        .collect();
    let new_reg_set_model = ModelRc::from(Rc::new(new_reg_set));

    if reg_sets_queue.len() >= REGISTERS_HISTORY_LIMIT-1 {
        reg_sets_queue.pop_front();
    }
    reg_sets_queue.push_back(new_reg_set_model);

    // Tell the UI to update the view (autoscroll feature)
    window.invoke_scroll_to_bottom_registers_view();
}


fn update_memory_view(window: &DebuggerMainWindow, debugger: &Debugger) {
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


fn generate_memory_strings(memory: &[u8], mut range: Range<usize>) -> Option<(String, String)> {
    // Note: benchmarks show pre-allocating the strings or reusing old strings does not impact performance
    // TODO: test it within the application context and see if there's any difference

    if memory.len() < range.end {
        range.end = memory.len();
        if range.start > range.end {
            return None;
        }
    }

    let mem_view = &memory[range.start..range.end];

    let mut lines_str = String::new();
    let mut mem_str = String::new();
    let mut row_index: usize = range.start;

    let mut mem_rows = mem_view.array_chunks::<BYTES_PER_MEMORY_ROW>();
    for full_row in mem_rows.by_ref() {
        for byte in &full_row[..BYTES_PER_MEMORY_ROW-1] {
            write!(mem_str, "{:02X} ", *byte).unwrap();
            // mem_str.push_str(format!("{:02X}", *byte).as_str());
            // mem_str.push(' ');
        }
        writeln!(mem_str, "{:02X}", full_row[BYTES_PER_MEMORY_ROW-1]).unwrap();
        // mem_str.push_str(format!("{:02X}", full_row[BYTES_PER_MEMORY_ROW-1]).as_str());
        // mem_str.push('\n');
        writeln!(lines_str, "{:#X}", row_index).unwrap();
        // lines_str.push_str(format!("{:#X}\n", row_index).as_str());
        row_index += BYTES_PER_MEMORY_ROW;
    }

    let remainder_row = mem_rows.remainder();
    if !remainder_row.is_empty() {
        writeln!(lines_str, "{:#X}", row_index).unwrap();
        // lines_str.push_str(format!("{:#X}\n", row_index).as_str());
        for byte in remainder_row {
            write!(mem_str, "{:02X} ", *byte).unwrap();
            // mem_str.push_str(format!("{:02X}", *byte).as_str());
            // mem_str.push(' ');
        }
    }

    Some((
        lines_str,
        mem_str
    ))
}


#[cfg(test)]
mod tests {
    use super::*;
    extern crate test;
    use test::Bencher;


    #[bench]
    fn generate_mem_view(b: &mut Bencher) {
        let size = 100000;
        let mem = vec![0;size];
        let range = Range { start: 0, end: size };

        b.iter(|| generate_memory_strings(&mem, range));
    }

}
