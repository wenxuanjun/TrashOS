#![no_std]
#![no_main]

extern crate alloc;
use alloc::string::String;
use apps::{println, syscall::sleep};

#[unsafe(no_mangle)]
fn main() {
    println!("Sleeping for 1 second...");
    sleep(1000);
    println!("Woke up!");
    let hello = String::from("Hello!");
    for _ in 0..50 {
        apps::print!("{}", hello);
        sleep(100);
    }
}
