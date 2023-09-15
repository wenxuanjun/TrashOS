#![no_std]
#![no_main]

use core::panic::PanicInfo;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

#[no_mangle]
pub unsafe extern "sysv64" fn _start() -> ! {
    loop {
        let hello = "Hello 2!";
        unsafe {
            core::arch::asm!(
                "syscall",
                in("rax") 1,
                in("rdi") hello.as_ptr(),
                in("rsi") hello.len(),
                in("rdx") 0,
                in("r10") 0,
                in("r8") 0,
                in("r9") 0,
            );
        }
        for _ in 1..100000 {
            unsafe {
                core::arch::asm!("nop");
            }
        }
    }
}
