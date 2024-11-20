#![no_std]
#![no_main]

extern crate alloc;
use alloc::string::String;

#[no_mangle]
unsafe fn main() {
    let hello = String::from("Hello!");
    for _ in 0..100 {
        apps::print!("{}", hello);
        for _ in 1..10000000 {
            core::arch::asm!("nop");
        }
    }
}
