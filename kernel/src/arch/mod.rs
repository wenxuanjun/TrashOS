use apic::{APIC_INIT, LAPIC, LAPIC_TIMER_INITIAL};
use core::sync::atomic::Ordering;
use limine::mp::Cpu;
use x86_64::registers::control::{Cr0, Cr4};
use x86_64::registers::control::{Cr0Flags, Cr4Flags};

use crate::syscall;
use crate::tasks::scheduler::SCHEDULER_INIT;
use interrupts::IDT;
use smp::CPUS;

pub mod acpi;
pub mod apic;
pub mod gdt;
pub mod interrupts;
pub mod smp;

pub fn init_sse() {
    let mut cr0 = Cr0::read();
    cr0.remove(Cr0Flags::EMULATE_COPROCESSOR);
    cr0.insert(Cr0Flags::MONITOR_COPROCESSOR);
    unsafe { Cr0::write(cr0) };

    let mut cr4 = Cr4::read();
    cr4.insert(Cr4Flags::OSFXSR);
    cr4.insert(Cr4Flags::OSXMMEXCPT_ENABLE);
    unsafe { Cr4::write(cr4) };
}

unsafe extern "C" fn ap_entry(smp_info: &Cpu) -> ! {
    CPUS.write().load(smp_info.lapic_id);
    IDT.load();

    init_sse();

    while !APIC_INIT.load(Ordering::SeqCst) {
        core::hint::spin_loop()
    }
    LAPIC.lock().enable();

    let timer_initial = LAPIC_TIMER_INITIAL.load(Ordering::Relaxed);
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
