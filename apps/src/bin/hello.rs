#![no_std]
#![no_main]

use core::panic::PanicInfo;
use apps::syscall::write;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

#[no_mangle]
unsafe fn _start() {
    let hello = "Hello!";
    for _ in 1..500 {
        write(hello.as_ptr(), hello.len());
        for _ in 1..100000 {
            core::arch::asm!("nop");
        }
    }
    loop {
        core::arch::asm!("nop");
    }
}
