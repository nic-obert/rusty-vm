use core::range::Range;
use std::cell::RefCell;
use std::rc::Rc;

use rfd::FileDialog;
use slint::{CloseRequestResponse, Model, ModelRc, SharedString, ToSharedString, VecModel};

use rusty_vm_lib::registers::REGISTER_SIZE;

use crate::queue_model::QueueModel;
use crate::debugger::Debugger;


slint::include_modules!();


/// How many bytes to show for every row in the memory inspector area.
const BYTES_PER_MEMORY_ROW: usize = 16;
/// How many register sets to keep in the registers area.
const REGISTERS_HISTORY_LIMIT: usize = 10;


pub fn run_ui(debugger: Rc<RefCell<Debugger>>) -> Result<(), slint::PlatformError> {
    let main_window = MainWindow::new()?;

    create_ui(&main_window, debugger);

    main_window.run()
}


fn create_ui(main_window: &MainWindow, debugger: Rc<RefCell<Debugger>>) {

    // Create data models
    //

    // Initialize the register model
    let regs: Rc<QueueModel<ModelRc<SharedString>>> = Rc::new(QueueModel::default());
    let regs_model = ModelRc::from(regs);
    main_window.set_register_sets(regs_model);

    // Initialize the breakpoint model
    let breakpoints: Rc<VecModel<BreakPoint>> = Rc::new(VecModel::default());
    let breakpoints_model = ModelRc::from(breakpoints);
    main_window.set_breakpoints(breakpoints_model);


    // Initialize the models with initial values, if required
    //
    update_register_view(main_window, &debugger.borrow());

    // Bind UI to backend functionality
    //
    let debugger_ref = Rc::clone(&debugger);
    main_window.global::<Backend>().on_dump_core(move || {

        let pick_file = || {
            FileDialog::new()
                .set_title("Dump core to file")
                .save_file()
        };

        debugger_ref.borrow().dump_core(pick_file);
    });

    let debugger_ref = Rc::clone(&debugger);
    let window_weak = main_window.as_weak();
    main_window.global::<Backend>().on_stop(move || {
        let window = window_weak.unwrap();
        debugger_ref.borrow().stop_vm();
        update_register_view(&window, &debugger_ref.borrow());
    });

    let debugger_ref = Rc::clone(&debugger);
    main_window.global::<Backend>().on_continue(move || {
        debugger_ref.borrow_mut().continue_vm();
    });

    let debugger_ref = Rc::clone(&debugger);
    let window_weak = main_window.as_weak();
    main_window.global::<Backend>().on_step_in(move || {
        let window = window_weak.unwrap();
        debugger_ref.borrow_mut().step_in();
        update_all_views(&window, &debugger_ref.borrow());
    });

    let debugger_ref = Rc::clone(&debugger);
    main_window.window().on_close_requested(move || {
        debugger_ref.borrow().close();
        CloseRequestResponse::HideWindow
    });

    let debugger_ref = Rc::clone(&debugger);
    let window_weak = main_window.as_weak();
    main_window.global::<Backend>().on_memory_start_changed(move || {
        let window = window_weak.unwrap();
        update_memory_view(&window, &debugger_ref.borrow());
    });

    let debugger_ref = Rc::clone(&debugger);
    let window_weak = main_window.as_weak();
    main_window.global::<Backend>().on_memory_span_changed(move || {
        let window = window_weak.unwrap();
        update_memory_view(&window, &debugger_ref.borrow());
    });

    let debugger_ref = Rc::clone(&debugger);
    let window_weak = main_window.as_weak();
    main_window.global::<Backend>().on_memory_refresh(move || {
        let window = window_weak.unwrap();
        update_memory_view(&window, &debugger_ref.borrow());
    });

    let debugger_ref = Rc::clone(&debugger);
    let window_weak = main_window.as_weak();
    main_window.global::<Backend>().on_add_breakpoint_here(move || {
        let window = window_weak.unwrap();
        debugger_ref.borrow_mut().add_persistent_breakpoint_at_pc().unwrap();
        update_breakpoint_view(&window, &debugger_ref.borrow());
        todo!()
    });

}


fn update_all_views(window: &MainWindow, debugger: &Debugger) {
    update_memory_view(window, debugger);
    update_register_view(window, debugger);
    update_breakpoint_view(window, debugger);
}


fn update_breakpoint_view(window: &MainWindow, debugger: &Debugger) {
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


fn update_register_view(window: &MainWindow, debugger: &Debugger) {
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
}


fn update_memory_view(window: &MainWindow, debugger: &Debugger) {
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
            mem_str.push_str(format!("{:02X}", *byte).as_str());
            mem_str.push(' ');
        }
        mem_str.push_str(format!("{:02X}", full_row[BYTES_PER_MEMORY_ROW-1]).as_str());
        mem_str.push('\n');
        lines_str.push_str(format!("{:#X}\n", row_index).as_str());
        row_index += BYTES_PER_MEMORY_ROW;
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
