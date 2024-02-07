#![no_std]
#![allow(non_snake_case)]
#![feature(abi_x86_interrupt)]
#![feature(alloc_error_handler)]
#![feature(naked_functions)]
#![feature(variant_count)]
#![feature(allocator_api)]

pub mod arch;
pub mod device;
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
    arch::gdt::init();
    arch::interrupts::IDT.load();
    memory::init(physical_memory_offset, memory_regions);
    arch::acpi::init(rsdp_addr);
    arch::apic::init();
    device::mouse::init();
    syscall::init();
    task::scheduler::init();
}
