#![no_std]
#![no_main]

extern crate alloc;
use alloc::string::{String, ToString};
use apps::syscall::write;

#[no_mangle]
unsafe fn main() {
    for (counter, _) in (0..200).enumerate() {
        let mut buf = String::from("[");
        buf.push_str(&counter.to_string());
        buf.push(']');
        write(buf.as_ptr(), buf.len());
        for _ in 1..10000000 {
            core::arch::asm!("nop");
        }
    }
}
