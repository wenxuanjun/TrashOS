#![no_std]
#![no_main]

use core::panic::PanicInfo;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

#[no_mangle]
pub unsafe extern "sysv64" fn _start() -> ! {
    for _ in 1..500 {
        let hello = "Hello!";
        unsafe {
            core::arch::asm!(
                "syscall",
                in("rax") 1,
                in("rdi") hello.as_ptr(),
                in("rsi") hello.len(),
            );
        }
        for _ in 1..100000 {
            unsafe {
                core::arch::asm!("nop");
            }
        }
    }
    loop {
        unsafe {
            core::arch::asm!("nop");
        }
    }
}
