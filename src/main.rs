#![no_std]
#![no_main]
#![allow(non_snake_case)]
#![feature(abi_x86_interrupt)]

pub mod apic;

extern crate alloc;
use core::panic::PanicInfo;
use alloc::{boxed::Box, vec::Vec};
use TrashOS::log::LogLevel;
use TrashOS::{println, task::keyboard};
use TrashOS::task::{Task, executor::Executor};
use bootloader_api::{BootInfo, entry_point};
use bootloader_api::config::{BootloaderConfig, Mapping};

pub static BOOTLOADER_CONFIG: BootloaderConfig = {
    let mut config = BootloaderConfig::new_default();
    config.mappings.physical_memory = Some(Mapping::Dynamic);
    config
};

entry_point!(main, config = &BOOTLOADER_CONFIG);

fn main(boot_info: &'static mut BootInfo) -> ! {
    TrashOS::init(boot_info);

    TrashOS::log!(LogLevel::Error, "This is an error message!");
    TrashOS::log!(LogLevel::Warn, "This is a warning message!");
    TrashOS::log!(LogLevel::Debug, "This is a debug message!");

    let heap_value = Box::new(41);
    println!("The heap start at {:p}", heap_value);

    let mut vec = Vec::new();
    for i in 0..500 { vec.push(i); }
    println!("Now test the vec is at {:p}", vec.as_slice());

    println!("min0911_ TQL%%%!");

    let mut executor = Executor::new();
    executor.spawn(Task::new(keyboard::print_keypresses()));
    executor.run();
}

#[panic_handler]
fn panic(_panic_info: &PanicInfo<'_>) -> ! {
    println!("{}", _panic_info);
    loop { x86_64::instructions::hlt(); }
}