#![no_std]
#![feature(abi_x86_interrupt)]
#![feature(alloc_error_handler)]
#![feature(variant_count)]
#![feature(allocator_api)]
#![feature(panic_can_unwind)]
#![allow(unsafe_op_in_unsafe_fn)]

pub mod arch;
pub mod drivers;
pub mod io;
pub mod mem;
pub mod syscall;
pub mod tasks;
pub mod unwind;

extern crate alloc;

use arch::smp::BSP_LAPIC_ID;
use spin::Lazy;

pub fn init() {
    mem::init_heap();
    drivers::log::init();
    Lazy::force(&drivers::hpet::HPET);
    arch::smp::CPUS.write().load(*BSP_LAPIC_ID);
    arch::interrupts::IDT.load();
    arch::init_sse();
    arch::smp::CPUS.write().init_ap();
    arch::apic::init();
    drivers::mouse::init();
    syscall::init();
    tasks::scheduler::init();
}
