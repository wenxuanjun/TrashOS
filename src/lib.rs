#![no_std]
#![allow(non_snake_case)]
#![feature(abi_x86_interrupt)]
#![feature(alloc_error_handler)]
#![feature(naked_functions)]
#![feature(asm_const)]
#![feature(variant_count)]
#![feature(allocator_api)]

pub mod acpi;
pub mod apic;
pub mod device;
pub mod gdt;
pub mod interrupts;
pub mod log;
pub mod memory;
pub mod printk;
pub mod syscall;
pub mod task;

extern crate alloc;
use bootloader_api::BootInfo;

pub fn init(boot_info: &'static mut BootInfo) {
    let BootInfo {
        framebuffer,
        physical_memory_offset,
        memory_regions,
        rsdp_addr,
        ..
    } = boot_info;

    printk::init(framebuffer);
    gdt::init_gdt();
    interrupts::IDT.load();
    memory::init(physical_memory_offset, memory_regions);
    acpi::init(*rsdp_addr);
    apic::init();
    device::mouse::init();
    syscall::init();
    task::scheduler::init();
}
