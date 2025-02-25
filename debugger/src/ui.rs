

slint::include_modules!();


pub fn run_ui() -> Result<(), slint::PlatformError> {
    let main_window = MainWindow::new()?;

    create_ui(&main_window);

    main_window.run()
}


fn create_ui(main_window: &MainWindow) {

}
