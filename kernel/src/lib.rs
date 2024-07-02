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

pub fn init() {
    console::printk::init();
    console::log::init();
    arch::gdt::init();
    arch::interrupts::IDT.load();
    memory::init();
    arch::acpi::init();
    arch::hpet::init();
    arch::apic::init();
    device::mouse::init();
    syscall::init();
    task::scheduler::init();
}
