#![no_std]
#![no_main]
#![feature(naked_functions)]
#![feature(alloc_error_handler)]
#![feature(stmt_expr_attributes)]

pub mod memory;
pub mod syscall;
pub mod stdio;
pub mod unwind;

extern crate alloc;

extern "C" {
    fn main();
}

#[no_mangle]
unsafe fn _start() -> ! {
    main();
    syscall::exit();
}
