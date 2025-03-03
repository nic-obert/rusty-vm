use std::rc::Rc;

use rfd::FileDialog;

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
}
