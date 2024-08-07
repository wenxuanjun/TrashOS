#![no_std]
#![no_main]
#![feature(naked_functions)]
#![feature(alloc_error_handler)]
#![feature(stmt_expr_attributes)]

use core::panic::PanicInfo;

pub mod memory;
pub mod syscall;

extern "C" {
    fn main() -> ();
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

#[no_mangle]
unsafe extern "sysv64" fn _start() -> ! {
    memory::init_heap();
    main();
    syscall::exit();
}
