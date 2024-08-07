use alloc::boxed::Box;
use core::fmt::{self, Write};
use os_terminal::font::BitmapFont;
use os_terminal::Terminal;
use spin::{Lazy, Mutex};
use x86_64::instructions::interrupts;

use crate::device::display::Display;

pub static TERMINAL: Lazy<Mutex<Terminal<Display>>> = Lazy::new(|| {
    let mut terminal = Terminal::new(Display::new());
    terminal.set_font_manager(Box::new(BitmapFont));
    Mutex::new(terminal)
});

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

pub fn terminal_manual_flush() {
    TERMINAL.lock().set_auto_flush(false);
    loop {
        interrupts::without_interrupts(|| TERMINAL.lock().flush());
        x86_64::instructions::hlt();
    }
}
