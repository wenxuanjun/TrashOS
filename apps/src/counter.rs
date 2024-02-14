#![no_std]
#![no_main]

use core::panic::PanicInfo;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

#[no_mangle]
pub unsafe extern "sysv64" fn _start() -> ! {
    let mut counter = 0;
    for _ in 1..500 {
        let mut buf = [0u8; 6];
        let mut cnt = counter;
        for i in (0..6).rev() {
            buf[i] = (cnt % 10 + 48) as u8;
            cnt /= 10;
        }
        unsafe {
            core::arch::asm!(
                "syscall",
                in("rax") 1,
                in("rdi") &buf,
                in("rsi") buf.len(),
                in("rdx") 0,
                in("r10") 0,
                in("r8") 0,
                in("r9") 0,
            );
        }
        counter += 1;
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
