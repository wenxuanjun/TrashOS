use core::fmt::{self, Write};
use spin::{Lazy, Mutex};
use uart_16550::SerialPort;
use x86_64::instructions::interrupts;

#[macro_export]
macro_rules! serial_print {
    ($($arg: tt)*) => ($crate::device::serial::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! serial_println {
    () => ($crate::serial_print!("\n"));
    ($($arg: tt)*) => ($crate::serial_print!("{}\n", format_args!($($arg)*)));
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    interrupts::without_interrupts(|| {
        SERIAL.lock().write_fmt(args).unwrap();
    });
}

pub static SERIAL: Lazy<Mutex<SerialPort>> = Lazy::new(|| {
    let mut serial_port = unsafe { SerialPort::new(0x3f8) };
    serial_port.init();
    Mutex::new(serial_port)
});
