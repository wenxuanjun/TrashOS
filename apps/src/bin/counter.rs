#![no_std]
#![no_main]

extern crate alloc;
use alloc::format;
use apps::syscall::sleep;

#[unsafe(no_mangle)]
fn main() {
    for (counter, _) in (0..100).enumerate() {
        apps::print!("{}", format!("[{}]", counter));
        sleep(50);
    }
}
