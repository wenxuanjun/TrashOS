use alloc::boxed::Box;
use core::fmt::{self, Write};
use core::time::Duration;
use crossbeam_queue::ArrayQueue;
use os_terminal::Terminal;
use os_terminal::font::TrueTypeFont;
use spin::{Lazy, Mutex};
use x86_64::instructions::interrupts;

use super::{display::Display, speaker::SPEAKER};

const FONT_BUFFER: &[u8] = include_bytes!("../../../builder/assets/SourceCodePro.ttf");

pub static TERMINAL: Lazy<Mutex<Terminal<Display>>> = Lazy::new(|| {
    let mut terminal = Terminal::new(Display::default());
    terminal.set_font_manager(Box::new(TrueTypeFont::new(10.0, FONT_BUFFER)));
    let bell_handler = || {
        crate::serial_println!("Bell!");
        SPEAKER.lock().beep(750, Duration::from_millis(200));
    };
    terminal.set_bell_handler(Some(bell_handler));
    Mutex::new(terminal)
});

pub static SCANCODE_QUEUE: Lazy<ArrayQueue<u8>> = Lazy::new(|| ArrayQueue::new(256));

#[inline]
pub fn _print(args: fmt::Arguments) {
    interrupts::without_interrupts(|| {
        TERMINAL.lock().write_fmt(args).unwrap();
    });
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => (
        $crate::driver::terminal::_print(
            format_args!($($arg)*)
        )
    )
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)))
}

pub fn terminal_thread() {
    TERMINAL.lock().set_auto_flush(false);
    loop {
        interrupts::without_interrupts(|| {
            while let Some(scancode) = SCANCODE_QUEUE.pop() {
                let result = TERMINAL.lock().handle_keyboard(scancode);
                result.map(|ansi_string| print!("{}", ansi_string));
            }
            TERMINAL.lock().flush();
        });
        crate::syscall::r#yield();
    }
}
