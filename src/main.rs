#![no_std]
#![no_main]
#![allow(non_snake_case)]
#![feature(abi_x86_interrupt)]

extern crate alloc;
use bootloader_api::config::{BootloaderConfig, Mapping};
use bootloader_api::{entry_point, BootInfo};
use core::panic::PanicInfo;

pub static BOOTLOADER_CONFIG: BootloaderConfig = {
    let mut config = BootloaderConfig::new_default();
    config.mappings.physical_memory = Some(Mapping::Dynamic);
    config
};

entry_point!(main, config = &BOOTLOADER_CONFIG);

fn print_thread1() {
    loop {
        TrashOS::print!("[KernelThread1!]");
        x86_64::instructions::hlt();
    }
}

fn print_thread2() {
    loop {
        TrashOS::print!("[KernelThread2!]");
        x86_64::instructions::hlt();
    }
}

fn main(boot_info: &'static mut BootInfo) -> ! {
    TrashOS::init(boot_info);

    TrashOS::task::Thread::new_kernel_thread(print_thread1);
    TrashOS::task::Thread::new_kernel_thread(print_thread2);
    TrashOS::task::Process::new_user_process("Hello1", include_bytes!("../apps/src/hello")).unwrap();
    TrashOS::task::Process::new_user_process("Hello2", include_bytes!("../apps/src/hello2")).unwrap();
    x86_64::instructions::interrupts::enable();

    loop {
        x86_64::instructions::hlt();
    }
}

#[panic_handler]
fn panic(panic_info: &PanicInfo<'_>) -> ! {
    TrashOS::error!("{}", panic_info);
    loop {
        x86_64::instructions::hlt();
    }
}
