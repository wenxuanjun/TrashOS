#![no_std]
#![no_main]

extern crate alloc;
use alloc::string::String;
use apps::syscall::write;

#[no_mangle]
unsafe fn main() {
    let hello = String::from("Hello!");
    for _ in 0..200 {
        write(hello.as_ptr(), hello.len());
        for _ in 1..100000 {
            core::arch::asm!("nop");
        }
    }
}
