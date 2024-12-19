#![no_std]
#![no_main]

use std::*;

#[unsafe(no_mangle)]
fn main() {
    for (counter, _) in (0..100).enumerate() {
        print!("[{}]", counter);
        sleep(50);
    }
}
