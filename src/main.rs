#![no_std]
#![no_main]
#![allow(non_snake_case)]
#![feature(abi_x86_interrupt)]

use TrashOS::println;
use core::panic::PanicInfo;

#[no_mangle]
pub extern "C" fn _start() -> ! {
    TrashOS::init();
    println!("min0911_ TQL%%%!");
    loop { x86_64::instructions::hlt(); }
}

#[panic_handler]
fn panic(_panic_info: &PanicInfo<'_>) -> ! {
    println!("{}", _panic_info);
    loop { x86_64::instructions::hlt(); }
}