#![no_std]
#![feature(abi_x86_interrupt)]
#![feature(alloc_error_handler)]
#![feature(naked_functions)]
#![feature(variant_count)]
#![feature(allocator_api)]
#![feature(panic_can_unwind)]
#![allow(unsafe_op_in_unsafe_fn)]
#![allow(clippy::missing_safety_doc)]
#![allow(clippy::not_unsafe_ptr_arg_deref)]

pub mod arch;
pub mod driver;
pub mod io;
pub mod mem;
pub mod syscall;
pub mod task;
pub mod unwind;

extern crate alloc;

use arch::smp::BSP_LAPIC_ID;
use spin::Lazy;

pub fn init() {
    mem::init_heap();
    driver::log::init();
    Lazy::force(&driver::hpet::HPET);
    arch::smp::CPUS.write().load(*BSP_LAPIC_ID);
    arch::interrupts::IDT.load();
    arch::init_sse();
    arch::smp::CPUS.write().init_ap();
    arch::apic::init();
    driver::mouse::init();
    syscall::init();
    task::scheduler::init();
    Lazy::force(&driver::pcie::PCI_DEVICES);
}
