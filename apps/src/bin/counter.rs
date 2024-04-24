#![no_std]
#![no_main]

use apps::syscall::write;
use core::panic::PanicInfo;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

#[no_mangle]
unsafe fn _start() -> ! {
    let mut counter = 0;
    for _ in 0..500 {
        let mut buf = [0; 7];
        buf[0] = b'[';
        buf[6] = b']';
        let num_buf = &mut buf[1..6];
        let mut cnt = counter;
        for i in (0..5).rev() {
            num_buf[i] = (cnt % 10 + 48) as u8;
            cnt /= 10;
        }
        counter += 1;
        write(buf.as_ptr(), buf.len());
        for _ in 1..100000 {
            core::arch::asm!("nop");
        }
    }
    loop {
        core::arch::asm!("nop");
    }
}
