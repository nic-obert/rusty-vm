use rust_vm_lib::vm::{ErrorCodes, Address};
use termion::cursor;
use termion::input::TermRead;
use termion::raw::IntoRawMode;
use termion::cursor::DetectCursorPos;

use crate::processor::Processor;

use rust_vm_lib::registers::Registers;


pub fn handle_code(code: usize, processor: &mut Processor) -> ErrorCodes {
    match CODE_HANDLERS[code](processor) {
        Ok(_) => ErrorCodes::NoError,
        Err(_) => ErrorCodes::GenericError
    }
}


fn handle_goto(processor: &mut Processor) -> std::io::Result<()> {
    let column = processor.get_register(Registers::R1);
    let row = processor.get_register(Registers::R2);

    print!("{}", cursor::Goto(column as u16, row as u16));

    Ok(())
}


fn handle_clear(_processor: &mut Processor) -> std::io::Result<()> {
    print!("{}", termion::clear::All);

    Ok(())
}


fn handle_blink(_processor: &mut Processor) -> std::io::Result<()> {
    print!("{}", termion::style::Blink);

    Ok(())
}


fn handle_bold(_processor: &mut Processor) -> std::io::Result<()> {
    print!("{}", termion::style::Bold);

    Ok(())
}


fn handle_underline(_processor: &mut Processor) -> std::io::Result<()> {
    print!("{}", termion::style::Underline);

    Ok(())
}


fn handle_reset(_processor: &mut Processor) -> std::io::Result<()> {
    print!("{}", termion::style::Reset);

    Ok(())
}


fn handle_hide_cursor(_processor: &mut Processor) -> std::io::Result<()> {
    print!("{}", termion::cursor::Hide);

    Ok(())
}


fn handle_show_cursor(_processor: &mut Processor) -> std::io::Result<()> {
    print!("{}", termion::cursor::Show);

    Ok(())
}


fn handle_down(processor: &mut Processor) -> std::io::Result<()> {
    let n = processor.get_register(Registers::R1);
    print!("{}", cursor::Down(n as u16));

    Ok(())
}


fn handle_up(processor: &mut Processor) -> std::io::Result<()> {
    let n = processor.get_register(Registers::R1);
    print!("{}", cursor::Up(n as u16));

    Ok(())
}


fn handle_right(processor: &mut Processor) -> std::io::Result<()> {
    let n = processor.get_register(Registers::R1);
    print!("{}", cursor::Right(n as u16));

    Ok(())
}


fn handle_left(processor: &mut Processor) -> std::io::Result<()> {
    let n = processor.get_register(Registers::R1);
    print!("{}", cursor::Left(n as u16));

    Ok(())
}


fn handle_blinking_block(_processor: &mut Processor) -> std::io::Result<()> {
    print!("{}", cursor::BlinkingBlock);

    Ok(())
}


fn handle_steady_block(_processor: &mut Processor) -> std::io::Result<()> {
    print!("{}", cursor::SteadyBlock);

    Ok(())
}


fn handle_blinking_underline(_processor: &mut Processor) -> std::io::Result<()> {
    print!("{}", cursor::BlinkingUnderline);

    Ok(())
}


fn handle_steady_underline(_processor: &mut Processor) -> std::io::Result<()> {
    print!("{}", cursor::SteadyUnderline);

    Ok(())
}


fn handle_blinking_bar(_processor: &mut Processor) -> std::io::Result<()> {
    print!("{}", cursor::BlinkingBar);

    Ok(())
}


fn handle_steady_bar(_processor: &mut Processor) -> std::io::Result<()> {
    print!("{}", cursor::SteadyBar);

    Ok(())
}


fn handle_save_cursor_position(_processor: &mut Processor) -> std::io::Result<()> {
    print!("{}", cursor::Save);

    Ok(())
}


fn handle_restore_cursor_position(_processor: &mut Processor) -> std::io::Result<()> {
    print!("{}", cursor::Restore);

    Ok(())
}


fn handle_clear_line(_processor: &mut Processor) -> std::io::Result<()> {
    print!("{}", termion::clear::CurrentLine);

    Ok(())
}


fn handle_clear_after(_processor: &mut Processor) -> std::io::Result<()> {
    print!("{}", termion::clear::AfterCursor);

    Ok(())
}


fn handle_clear_before(_processor: &mut Processor) -> std::io::Result<()> {
    print!("{}", termion::clear::BeforeCursor);

    Ok(())
}


fn handle_clear_until_newline(_processor: &mut Processor) -> std::io::Result<()> {
    print!("{}", termion::clear::UntilNewline);

    Ok(())
}


fn handle_get_terminal_size(processor: &mut Processor) -> std::io::Result<()> {
    let (width, height) = termion::terminal_size()?;
    processor.set_register(Registers::R1, width as u64);
    processor.set_register(Registers::R2, height as u64);

    Ok(())
}


fn handle_get_terminal_size_pixels(processor: &mut Processor) -> std::io::Result<()> {
    let (width, height) = termion::terminal_size_pixels()?;
    processor.set_register(Registers::R1, width as u64);
    processor.set_register(Registers::R2, height as u64);

    Ok(())
}


fn handle_get_cursor_position(processor: &mut Processor) -> std::io::Result<()> {
    let mut stdout = std::io::stdout().into_raw_mode()?;
    let (column, row) = stdout.cursor_pos()?;
    processor.set_register(Registers::R1, column as u64);
    processor.set_register(Registers::R2, row as u64);

    Ok(())
}


/// Start a thread that listens for key events and writes them to the given address.
/// A key event is 2 bytes: the first byte is the modifier code, the second byte is the key code
fn handle_get_key_listener(processor: &mut Processor) -> std::io::Result<()> {
    const KEY_DATA_SIZE: usize = 2;
    let key_store_address = processor.get_register(Registers::R1) as Address;
    let key_data_slice = processor.memory.get_bytes_mut(key_store_address, KEY_DATA_SIZE).as_mut_ptr() as Address;
    
    std::thread::spawn(move || {
        use termion::event::Key;

        let key_data_slice: &mut [u8; KEY_DATA_SIZE] = unsafe {
            &mut *(key_data_slice as *mut [u8; KEY_DATA_SIZE])
        };

        let stdin = std::io::stdin();
        let _stdout = std::io::stdout().into_raw_mode().unwrap();

        for c in stdin.keys() {
            let key_data: [u8; KEY_DATA_SIZE] = match c.unwrap() {
                Key::Backspace => [1, 0],
                Key::Left => [2, 0],
                Key::Right => [3, 0],
                Key::Up => [4, 0],
                Key::Down => [5, 0],
                Key::Home => [6, 0],
                Key::End => [7, 0],
                Key::PageUp => [8, 0],
                Key::PageDown => [9, 0],
                Key::BackTab => [10, 0],
                Key::Delete => [11, 0],
                Key::Insert => [12, 0],
                Key::F(n) => [13, n],
                Key::Char(c) => [14, c as u8],
                Key::Alt(c) => [15, c as u8],
                Key::Ctrl(c) => [16, c as u8],
                Key::Null => [17, 0],
                Key::Esc => [18, 0],
                Key::__IsNotComplete => [19, 0],
            };

            key_data_slice.copy_from_slice(&key_data);
        }
    });

    Ok(())
}


const CODE_HANDLERS: [ fn(&mut Processor) -> std::io::Result<()>; 28 ] = [
    handle_goto,
    handle_clear,
    handle_blink,
    handle_bold,
    handle_underline,
    handle_reset,
    handle_hide_cursor,
    handle_show_cursor,
    handle_down,
    handle_up,
    handle_right,
    handle_left,
    handle_blinking_block,
    handle_steady_block,
    handle_blinking_underline,
    handle_steady_underline,
    handle_blinking_bar,
    handle_steady_bar,
    handle_save_cursor_position,
    handle_restore_cursor_position,
    handle_clear_line,
    handle_clear_after,
    handle_clear_before,
    handle_clear_until_newline,
    handle_get_terminal_size,
    handle_get_terminal_size_pixels,
    handle_get_cursor_position,
    handle_get_key_listener,
];

