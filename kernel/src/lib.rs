#![no_std]
#![feature(abi_x86_interrupt)]
#![feature(alloc_error_handler)]
#![feature(naked_functions)]
#![feature(variant_count)]
#![feature(allocator_api)]
#![allow(unsafe_op_in_unsafe_fn)]

pub mod arch;
pub mod driver;
pub mod mem;
pub mod syscall;
pub mod task;
pub mod unwind;

extern crate alloc;

use arch::smp::BSP_LAPIC_ID;

pub fn init() {
    mem::init_heap();
    driver::log::init();
    arch::smp::CPUS.write().load(*BSP_LAPIC_ID);
    arch::interrupts::IDT.load();
    arch::smp::CPUS.write().init_ap();
    arch::apic::init();
    driver::mouse::init();
    syscall::init();
    task::scheduler::init();
}
