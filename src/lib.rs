#![no_std]
#![allow(non_snake_case)]
#![feature(abi_x86_interrupt)]
#![feature(alloc_error_handler)]

pub mod gdt;
pub mod interrupts;
//pub mod apic;
pub mod memory;
pub mod serial;
pub mod printk;
pub mod log;
pub mod allocator;
pub mod task;

extern crate alloc;
use bootloader_api::BootInfo;

pub fn init(boot_info: &'static BootInfo) {
    gdt::init_gdt();
    interrupts::IDT.load();
    let mut memory = memory::init(boot_info);
    allocator::init_heap(&mut memory);
    //apic::init(boot_info);
    printk::init(boot_info);
}
