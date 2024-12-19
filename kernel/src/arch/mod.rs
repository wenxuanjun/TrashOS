use apic::{APIC_INIT, CALIBRATED_TIMER_INITIAL, LAPIC};
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
    CPUS.write().load(smp_info.lapic_id);
    IDT.load();

    while !APIC_INIT.load(Ordering::SeqCst) {
        core::hint::spin_loop()
    }
    LAPIC.lock().enable();

    let timer_initial = CALIBRATED_TIMER_INITIAL.load(Ordering::SeqCst);
    LAPIC.lock().set_timer_initial(timer_initial);

    syscall::init();

    while !SCHEDULER_INIT.load(Ordering::SeqCst) {
        core::hint::spin_loop()
    }
    x86_64::instructions::interrupts::enable();
    log::debug!("Application Processor {} started", smp_info.id);

    loop {
        x86_64::instructions::hlt();
    }
}
