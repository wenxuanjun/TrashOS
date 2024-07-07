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

pub static GLOBAL_MUTEX: spin::Mutex<()> = spin::Mutex::new(());

pub fn init() {
    console::log::init();
    arch::smp::CPUS.lock().init_bsp();
    arch::interrupts::IDT.load();
    memory::init_heap();
    arch::smp::CPUS.lock().init_ap();
    device::hpet::init();
    arch::apic::init();
    device::mouse::init();
    syscall::init();
    task::scheduler::init();
}
