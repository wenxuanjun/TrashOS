#![no_std]
#![feature(abi_x86_interrupt)]
#![feature(alloc_error_handler)]
#![feature(naked_functions)]
#![feature(variant_count)]
#![feature(allocator_api)]
#![feature(panic_can_unwind)]
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
    init_sse();
    panic!("A panic test");
}

fn init_sse() {
    use x86_64::registers::control::{Cr0, Cr4};
    use x86_64::registers::control::{Cr0Flags, Cr4Flags};

    let mut cr0 = Cr0::read();
    cr0.remove(Cr0Flags::EMULATE_COPROCESSOR);
    cr0.insert(Cr0Flags::MONITOR_COPROCESSOR);
    unsafe { Cr0::write(cr0) };

    let mut cr4 = Cr4::read();
    cr4.insert(Cr4Flags::OSFXSR);
    cr4.insert(Cr4Flags::OSXMMEXCPT_ENABLE);
    unsafe { Cr4::write(cr4) };
}
