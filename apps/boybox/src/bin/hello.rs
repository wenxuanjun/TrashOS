#![no_std]
#![no_main]

use std::*;

#[unsafe(no_mangle)]
fn main() {
    println!("Sleeping for 1 second...");
    sleep(1000);
    println!("Woke up!");
    for _ in 0..10 {
        print!("{}", "Hello!");
        sleep(100);
    }
}
