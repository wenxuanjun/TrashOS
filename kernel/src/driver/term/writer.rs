use alloc::string::ToString;
use core::fmt::{self, Write};

use super::service::TERMINAL_BUFFER;

pub struct TerminalWriter;

impl Write for TerminalWriter {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        TERMINAL_BUFFER.force_push(s.to_string());
        Ok(())
    }
}

pub fn _print(args: fmt::Arguments) {
    let _ = TerminalWriter.write_fmt(args);
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => (
        $crate::driver::term::_print(format_args!($($arg)*))
    )
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)))
}
