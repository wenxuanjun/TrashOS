#![no_std]
#![no_main]
#![allow(non_snake_case)]
#![feature(abi_x86_interrupt)]

extern crate alloc;
use bootloader_api::config::{BootloaderConfig, Mapping};
use bootloader_api::{entry_point, BootInfo};
use core::panic::PanicInfo;
use kernel::arch::rtc::RtcDateTime;
use kernel::device::keyboard::print_keypresses;
use kernel::task::{Process, Thread};

pub static BOOTLOADER_CONFIG: BootloaderConfig = {
    let mut config = BootloaderConfig::new_default();
    config.mappings.physical_memory = Some(Mapping::Dynamic);
    config
};

entry_point!(main, config = &BOOTLOADER_CONFIG);

fn main(boot_info: &'static mut BootInfo) -> ! {
    kernel::init(boot_info);
    Thread::new_kernel_thread(print_keypresses);

    let current_time = RtcDateTime::new().to_datetime().unwrap();
    kernel::info!("Current time: {}", current_time);

    let hello_raw_elf = include_bytes!("../../target/x86_64-unknown-none/debug/hello");
    let counter_raw_elf = include_bytes!("../../target/x86_64-unknown-none/debug/counter");
    Process::new_user_process("Hello", hello_raw_elf).unwrap();
    Process::new_user_process("Counter", counter_raw_elf).unwrap();

    loop {
        x86_64::instructions::hlt();
    }
}

#[panic_handler]
fn panic(panic_info: &PanicInfo<'_>) -> ! {
    kernel::error!("{}", panic_info);
    loop {
        x86_64::instructions::hlt();
    }
}
