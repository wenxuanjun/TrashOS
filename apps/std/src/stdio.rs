use core::fmt::Write;

use crate::syscall::write;
use alloc::fmt;

struct Writer;

impl fmt::Write for Writer {
    #[inline]
    fn write_str(&mut self, s: &str) -> fmt::Result {
        write(s.as_ptr(), s.len());
        Ok(())
    }
}

#[inline]
pub fn _print(args: fmt::Arguments) {
    Writer.write_fmt(args).unwrap();
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => (
        $crate::_print(format_args!($($arg)*))
    )
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)))
}
