#![no_std]
#![no_main]
#![allow(non_snake_case)]

mod vga_buffer;
use core::panic::PanicInfo;

#[no_mangle]
pub extern "C" fn _start() -> ! {
    println!("Some dirty words that you will not see.");
    clear_screen!();
    println!("Hello, World!");
    println!("Some numbers: {} {}", 42, 1.337);
    
    loop {}
}

#[panic_handler]
fn panic(_panic_info: &PanicInfo<'_>) -> ! {
    println!("{}", _panic_info);
    loop {}
}