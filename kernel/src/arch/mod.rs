use apic::{calibrate_timer, APIC_INIT, LAPIC};
use core::sync::atomic::Ordering;
use interrupts::IDT;
use limine::smp::Cpu;
use smp::CPUS;

use crate::syscall;
use crate::task::scheduler::SCHEDULER_INIT;

pub mod acpi;
pub mod apic;
pub mod gdt;
pub mod interrupts;
pub mod smp;

unsafe extern "C" fn ap_entry(smp_info: &Cpu) -> ! {
    CPUS.write().get(smp_info.lapic_id).load();
    IDT.load();

    while !APIC_INIT.load(Ordering::Relaxed) {}
    LAPIC.lock().enable();
    calibrate_timer();

    syscall::init();

    while !SCHEDULER_INIT.load(Ordering::Relaxed) {}
    x86_64::instructions::interrupts::enable();
    log::debug!("Application Processor {} started", smp_info.id);

    loop {
        x86_64::instructions::hlt();
    }
}
