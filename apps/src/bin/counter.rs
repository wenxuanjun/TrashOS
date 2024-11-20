#![no_std]
#![no_main]

extern crate alloc;
use alloc::format;

#[no_mangle]
unsafe fn main() {
    for (counter, _) in (0..200).enumerate() {
        let buf = format!("[{}]", counter);
        apps::print!("{}", buf);
        for _ in 1..10000000 {
            core::arch::asm!("nop");
        }
    }
}
