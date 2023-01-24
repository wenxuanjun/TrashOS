#![no_std]
#![allow(non_snake_case)]
#![feature(abi_x86_interrupt)]
#![feature(alloc_error_handler)]

pub mod gdt;
pub mod interrupts;
pub mod memory;
pub mod allocator;
pub mod serial;
pub mod printk;
pub mod log;
pub mod acpi;
pub mod apic;
pub mod task;
pub mod keyboard;
pub mod mouse;

extern crate alloc;
use bootloader_api::BootInfo;

pub fn init(boot_info: &'static BootInfo) {
    gdt::init_gdt();
    printk::init(boot_info);
    interrupts::IDT.load();
    memory::init(boot_info);
    allocator::init_heap();
    let apic = acpi::init(boot_info);
    apic::init(&apic);
    mouse::init();
}
