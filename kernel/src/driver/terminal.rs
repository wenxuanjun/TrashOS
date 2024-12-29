use alloc::boxed::Box;
use core::fmt::{self, Write};
use core::time::Duration;
use crossbeam_queue::ArrayQueue;
use os_terminal::Terminal;
use os_terminal::font::TrueTypeFont;
use spin::Lazy;

use super::{display::Display, speaker::SPEAKER};
use crate::syscall::r#yield;

const SCANCODE_QUEUE_SIZE: usize = 256;
const TERMINAL_BUFFER_SIZE: usize = 2048;

const FONT_BUFFER: &[u8] = include_bytes!("../../../builder/assets/SourceCodePro.otf");

pub static SCANCODE_QUEUE: Lazy<ArrayQueue<u8>> =
    Lazy::new(|| ArrayQueue::new(SCANCODE_QUEUE_SIZE));
pub static TERMINAL_BUFFER: Lazy<ArrayQueue<char>> =
    Lazy::new(|| ArrayQueue::new(TERMINAL_BUFFER_SIZE));

struct TerminalWriter;

impl Write for TerminalWriter {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        s.chars()
            .try_for_each(|c| TERMINAL_BUFFER.push(c))
            .map_err(|_| fmt::Error)
    }
}

#[inline]
pub fn _print(args: fmt::Arguments) {
    let _ = TerminalWriter.write_fmt(args);
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => (
        $crate::driver::terminal::_print(format_args!($($arg)*))
    )
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)))
}

pub fn terminal_thread() {
    let mut terminal = Terminal::new(Display::default());
    terminal.set_auto_crnl(true);
    terminal.set_auto_flush(false);

    terminal.set_font_manager(Box::new(TrueTypeFont::new(10.0, FONT_BUFFER)));
    let bell_handler = || SPEAKER.lock().beep(750, Duration::from_millis(200));
    terminal.set_bell_handler(Some(bell_handler));

    loop {
        while let Some(scancode) = SCANCODE_QUEUE.pop() {
            terminal
                .handle_keyboard(scancode)
                .map(|c| TerminalWriter.write_str(&c));
        }

        while let Some(character) = TERMINAL_BUFFER.pop() {
            let _ = terminal.write_char(character);
        }

        terminal.flush();
        r#yield();
    }
}
