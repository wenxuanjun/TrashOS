#![no_std]
#![no_main]

extern crate alloc;
use alloc::string::String;
use apps::syscall::write;

#[no_mangle]
unsafe fn main() {
    let hello = String::from("Hello!");
    for _ in 0..100 {
        write(hello.as_ptr(), hello.len());
        for _ in 1..10000000 {
            core::arch::asm!("nop");
        }
    }
}
