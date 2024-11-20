use crate::syscall::write;
use alloc::{fmt, format};

#[inline]
pub fn _print(args: fmt::Arguments) {
    let buf = format!("{}", args);
    write(buf.as_ptr(), buf.len());
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => (
        $crate::stdio::_print(
            format_args!($($arg)*)
        )
    )
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)))
}
