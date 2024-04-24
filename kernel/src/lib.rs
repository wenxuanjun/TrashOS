#![no_std]
#![feature(abi_x86_interrupt)]
#![feature(alloc_error_handler)]
#![feature(naked_functions)]
#![feature(variant_count)]
#![feature(allocator_api)]

pub mod arch;
pub mod console;
pub mod device;
pub mod memory;
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

    console::printk::init(framebuffer);
    console::log::init();
    arch::gdt::init();
    arch::interrupts::IDT.load();
    memory::init(physical_memory_offset, memory_regions);
    arch::acpi::init(rsdp_addr);
    arch::hpet::init();
    arch::apic::init();
    device::mouse::init();
    syscall::init();
    task::scheduler::init();
}
