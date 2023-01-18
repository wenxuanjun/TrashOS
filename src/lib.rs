#![no_std]
#![allow(non_snake_case)]
#![feature(abi_x86_interrupt)]
#![feature(alloc_error_handler)]

pub mod gdt;
pub mod interrupts;
pub mod memory;
pub mod serial;
pub mod printk;
pub mod allocator;
pub mod task;

extern crate alloc;
use bootloader_api::BootInfo;

pub fn init(boot_info: &'static mut BootInfo) {
    gdt::init_gdt();
    interrupts::init_idt();
    unsafe { interrupts::PICS.lock().initialize() };
    x86_64::instructions::interrupts::enable();
    printk::init(boot_info);
}
