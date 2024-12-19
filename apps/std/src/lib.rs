#![no_std]
#![no_main]
#![feature(naked_functions)]
#![feature(alloc_error_handler)]
#![feature(macro_metavar_expr)]

mod memory;
mod stdio;
mod syscall;
mod unwind;

pub use stdio::_print;
pub use syscall::*;

#[allow(hidden_glob_reexports)]
extern crate alloc;
pub use alloc::*;

unsafe extern "C" {
    unsafe fn main();
}

#[unsafe(no_mangle)]
fn _start() -> ! {
    unsafe { main() };
    syscall::exit();
}
