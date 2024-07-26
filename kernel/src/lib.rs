#![no_std]
#![feature(abi_x86_interrupt)]
#![feature(alloc_error_handler)]
#![feature(naked_functions)]
#![feature(variant_count)]
#![feature(allocator_api)]

pub mod arch;
pub mod device;
pub mod memory;
pub mod syscall;
pub mod task;

extern crate alloc;

pub fn init() {
    memory::init_heap();
    device::log::init();
    arch::smp::CPUS.write().init_bsp();
    arch::interrupts::IDT.load();
    arch::smp::CPUS.write().init_ap();
    arch::apic::init();
    device::mouse::init();
    syscall::init();
    task::scheduler::init();
}
