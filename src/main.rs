#![no_std]
#![no_main]
#![allow(non_snake_case)]
#![feature(abi_x86_interrupt)]

extern crate alloc;
use core::panic::PanicInfo;
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

    let mut executor = Executor::new();
    executor.spawn(Task::new(TrashOS::keyboard::print_keypresses()));
    executor.run();
}

#[panic_handler]
fn panic(_panic_info: &PanicInfo<'_>) -> ! {
    TrashOS::error!("{}", _panic_info);
    loop { x86_64::instructions::hlt(); }
}