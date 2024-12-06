#![no_std]
#![no_main]
#![feature(naked_functions)]
#![feature(alloc_error_handler)]
#![feature(stmt_expr_attributes)]

pub mod memory;
pub mod stdio;
pub mod syscall;
pub mod unwind;

extern crate alloc;

unsafe extern "C" {
    fn main();
}

#[unsafe(no_mangle)]
fn _start() -> ! {
    unsafe { main() };
    syscall::exit();
}
