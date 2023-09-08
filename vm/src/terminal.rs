use std::sync::mpsc::{self, Sender};
use std::io;
use std::thread::{self, JoinHandle};
use std::time::Duration;

use rust_vm_lib::vm::{ErrorCodes, Address};
use rust_vm_lib::registers::Registers;

use termion::cursor;
use termion::input::TermRead;
use termion::raw::IntoRawMode;
use termion::cursor::DetectCursorPos;

use crate::error;
use crate::memory::Memory;
use crate::register::CPURegisters;


const KEY_DATA_SIZE: usize = 2;
const KEYBOARD_LISTENER_INTERVAL: Duration = Duration::from_millis(50);

// This type definition is placed here because it's unstable to place it inside the Terminal impl block
type CodeHanlder = fn(&mut Terminal, &mut CPURegisters, &mut Memory) -> io::Result<()>;


pub struct Terminal {

    key_listener: Option<(Sender<()>, JoinHandle<()>)>,

}


impl Terminal {

    pub fn new() -> Self {
        Self {
            key_listener: None,
        }
    }
 

    pub fn handle_code(&mut self, code: usize, registers: &mut CPURegisters, memory: &mut Memory) -> ErrorCodes {
        match Self::CODE_HANDLERS[code](self, registers, memory) {
            Ok(_) => ErrorCodes::NoError,
            Err(_) => ErrorCodes::GenericError
        }
    }


    fn handle_goto(&mut self, registers: &mut CPURegisters, _memory: &mut Memory) -> io::Result<()> {
        let column = registers.get(Registers::R1);
        let row = registers.get(Registers::R2);

        print!("{}", cursor::Goto(column as u16, row as u16));

        Ok(())
    }


    fn handle_clear(&mut self, _registers: &mut CPURegisters, _memory: &mut Memory) -> io::Result<()> {
        print!("{}", termion::clear::All);

        Ok(())
    }


    fn handle_blink(&mut self, _registers: &mut CPURegisters, _memory: &mut Memory) -> io::Result<()> {
        print!("{}", termion::style::Blink);

        Ok(())
    }


    fn handle_bold(&mut self, _registers: &mut CPURegisters, _memory: &mut Memory) -> io::Result<()> {
        print!("{}", termion::style::Bold);

        Ok(())
    }


    fn handle_underline(&mut self, _registers: &mut CPURegisters, _memory: &mut Memory) -> io::Result<()> {
        print!("{}", termion::style::Underline);

        Ok(())
    }


    fn handle_reset(&mut self, _registers: &mut CPURegisters, _memory: &mut Memory) -> io::Result<()> {
        print!("{}", termion::style::Reset);

        Ok(())
    }


    fn handle_hide_cursor(&mut self, _registers: &mut CPURegisters, _memory: &mut Memory) -> io::Result<()> {
        print!("{}", termion::cursor::Hide);

        Ok(())
    }


    fn handle_show_cursor(&mut self, _registers: &mut CPURegisters, _memory: &mut Memory) -> io::Result<()> {
        print!("{}", termion::cursor::Show);

        Ok(())
    }


    fn handle_down(&mut self, registers: &mut CPURegisters, _memory: &mut Memory) -> io::Result<()> {
        let n = registers.get(Registers::R1);
        print!("{}", cursor::Down(n as u16));

        Ok(())
    }


    fn handle_up(&mut self, registers: &mut CPURegisters, _memory: &mut Memory) -> io::Result<()> {
        let n = registers.get(Registers::R1);
        print!("{}", cursor::Up(n as u16));

        Ok(())
    }


    fn handle_right(&mut self, registers: &mut CPURegisters, _memory: &mut Memory) -> io::Result<()> {
        let n = registers.get(Registers::R1);
        print!("{}", cursor::Right(n as u16));

        Ok(())
    }


    fn handle_left(&mut self, registers: &mut CPURegisters, _memory: &mut Memory) -> io::Result<()> {
        let n = registers.get(Registers::R1);
        print!("{}", cursor::Left(n as u16));

        Ok(())
    }


    fn handle_blinking_block(&mut self, _registers: &mut CPURegisters, _memory: &mut Memory) -> io::Result<()> {
        print!("{}", cursor::BlinkingBlock);

        Ok(())
    }


    fn handle_steady_block(&mut self, _registers: &mut CPURegisters, _memory: &mut Memory) -> io::Result<()> {
        print!("{}", cursor::SteadyBlock);

        Ok(())
    }


    fn handle_blinking_underline(&mut self, _registers: &mut CPURegisters, _memory: &mut Memory) -> io::Result<()> {
        print!("{}", cursor::BlinkingUnderline);

        Ok(())
    }


    fn handle_steady_underline(&mut self, _registers: &mut CPURegisters, _memory: &mut Memory) -> io::Result<()> {
        print!("{}", cursor::SteadyUnderline);

        Ok(())
    }


    fn handle_blinking_bar(&mut self, _registers: &mut CPURegisters, _memory: &mut Memory) -> io::Result<()> {
        print!("{}", cursor::BlinkingBar);

        Ok(())
    }


    fn handle_steady_bar(&mut self, _registers: &mut CPURegisters, _memory: &mut Memory) -> io::Result<()> {
        print!("{}", cursor::SteadyBar);

        Ok(())
    }


    fn handle_save_cursor_position(&mut self, _registers: &mut CPURegisters, _memory: &mut Memory) -> io::Result<()> {
        print!("{}", cursor::Save);

        Ok(())
    }


    fn handle_restore_cursor_position(&mut self, _registers: &mut CPURegisters, _memory: &mut Memory) -> io::Result<()> {
        print!("{}", cursor::Restore);

        Ok(())
    }


    fn handle_clear_line(&mut self, _registers: &mut CPURegisters, _memory: &mut Memory) -> io::Result<()> {
        print!("{}", termion::clear::CurrentLine);

        Ok(())
    }


    fn handle_clear_after(&mut self, _registers: &mut CPURegisters, _memory: &mut Memory) -> io::Result<()> {
        print!("{}", termion::clear::AfterCursor);

        Ok(())
    }


    fn handle_clear_before(&mut self, _registers: &mut CPURegisters, _memory: &mut Memory) -> io::Result<()> {
        print!("{}", termion::clear::BeforeCursor);

        Ok(())
    }


    fn handle_clear_until_newline(&mut self, _registers: &mut CPURegisters, _memory: &mut Memory) -> io::Result<()> {
        print!("{}", termion::clear::UntilNewline);

        Ok(())
    }


    fn handle_get_terminal_size(&mut self, registers: &mut CPURegisters, _memory: &mut Memory) -> io::Result<()> {
        let (width, height) = termion::terminal_size()?;
        registers.set(Registers::R1, width as u64);
        registers.set(Registers::R2, height as u64);

        Ok(())
    }


    fn handle_get_terminal_size_pixels(&mut self, registers: &mut CPURegisters, _memory: &mut Memory) -> io::Result<()> {
        let (width, height) = termion::terminal_size_pixels()?;
        registers.set(Registers::R1, width as u64);
        registers.set(Registers::R2, height as u64);

        Ok(())
    }


    fn handle_get_cursor_position(&mut self, registers: &mut CPURegisters, _memory: &mut Memory) -> io::Result<()> {
        let mut stdout = io::stdout().into_raw_mode()?;
        let (column, row) = stdout.cursor_pos()?;
        registers.set(Registers::R1, column as u64);
        registers.set(Registers::R2, row as u64);

        Ok(())
    }


    /// Start a thread that listens for key events and writes them to the given address.
    /// A key event is 2 bytes: the first byte is the modifier code, the second byte is the key code
    fn handle_get_key_listener(&mut self, registers: &mut CPURegisters, memory: &mut Memory) -> io::Result<()> {
        
        // Check if a listener is already active
        if self.key_listener.is_some() {
            return Err(io::ErrorKind::AlreadyExists.into());
        }

        let key_store_address = registers.get(Registers::R1) as Address;
        let key_data_slice = memory.get_bytes_mut(key_store_address, KEY_DATA_SIZE).as_mut_ptr() as Address;
        
        let (tx, rx) = mpsc::channel::<()>();

        let join_handle = thread::spawn(move || {
            use termion::event::Key;

            let key_data_slice: &mut [u8; KEY_DATA_SIZE] = unsafe {
                &mut *(key_data_slice as *mut [u8; KEY_DATA_SIZE])
            };

            let mut key_events = termion::async_stdin().keys();
            let _stdout = io::stdout().into_raw_mode().unwrap();
            
            loop {

                // Check for the stop signal
                if rx.try_recv().is_ok() {
                    break;
                }

                if let Some(key_event) = key_events.next() {

                    let key_data: [u8; KEY_DATA_SIZE] = match key_event.unwrap() {
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

                thread::sleep(KEYBOARD_LISTENER_INTERVAL);
            }
        });

        self.key_listener = Some((tx, join_handle));

        Ok(())
    }


    fn handle_stop_key_listener(&mut self, _registers: &mut CPURegisters, _memory: &mut Memory) -> io::Result<()> {

        match self.key_listener.take() {
            Some((tx, join_handle)) => {
                if tx.send(()).is_err() {
                    Err(io::ErrorKind::Other.into())
                } else {
                    join_handle.join().unwrap_or_else(
                        |err| error::error(format!("Could not join the keyboard listener thread\n{:?}", err).as_str())
                    );
                    Ok(())
                }
            },
            None => Err(io::ErrorKind::NotFound.into())
        }
    }


    const CODE_HANDLERS: [ CodeHanlder; 29 ] = [
        Self::handle_goto,
        Self::handle_clear,
        Self::handle_blink,
        Self::handle_bold,
        Self::handle_underline,
        Self::handle_reset,
        Self::handle_hide_cursor,
        Self::handle_show_cursor,
        Self::handle_down,
        Self::handle_up,
        Self::handle_right,
        Self::handle_left,
        Self::handle_blinking_block,
        Self::handle_steady_block,
        Self::handle_blinking_underline,
        Self::handle_steady_underline,
        Self::handle_blinking_bar,
        Self::handle_steady_bar,
        Self::handle_save_cursor_position,
        Self::handle_restore_cursor_position,
        Self::handle_clear_line,
        Self::handle_clear_after,
        Self::handle_clear_before,
        Self::handle_clear_until_newline,
        Self::handle_get_terminal_size,
        Self::handle_get_terminal_size_pixels,
        Self::handle_get_cursor_position,
        Self::handle_get_key_listener,
        Self::handle_stop_key_listener,
    ];

}

