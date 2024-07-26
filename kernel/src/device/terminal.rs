use core::fmt::{self, Write};
use spin::{Lazy, Mutex};
use x86_64::instructions::interrupts;
use os_terminal::Terminal;

use crate::device::display::Display;

pub static TERMINAL: Lazy<Mutex<Terminal<Display>>> =
    Lazy::new(|| Mutex::new(Terminal::new(Display::new())));

#[inline]
pub fn _print(args: fmt::Arguments) {
    interrupts::without_interrupts(|| {
        TERMINAL.lock().write_fmt(args).unwrap();
    });
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => (
        $crate::device::terminal::_print(
            format_args!($($arg)*)
        )
    )
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)))
}
