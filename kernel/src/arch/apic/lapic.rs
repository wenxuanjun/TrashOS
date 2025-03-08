use core::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use core::time::Duration;
use x86_64::instructions::port::Port;

use derive_more::Deref;
use spin::{Lazy, Mutex};
use x2apic::lapic::{LocalApic, LocalApicBuilder, TimerMode};
use x86_64::PhysAddr;
use x86_64::structures::paging::PhysFrame;

use crate::arch::acpi::ACPI;
use crate::arch::interrupts::InterruptIndex;
use crate::driver::hpet::HPET;
use crate::mem::convert_physical_to_virtual;
use crate::mem::{KERNEL_PAGE_TABLE, MappingType, MemoryManager};

const TIMER_FREQUENCY_HZ: u32 = 200;
const TIMER_CALIBRATION_ITERATION: u32 = 50;

pub static APIC_INIT: AtomicBool = AtomicBool::new(false);
pub static LAPIC_TIMER_INITIAL: AtomicU32 = AtomicU32::new(0);

#[derive(Deref)]
pub struct LockedLocalApic(Mutex<LocalApic>);

unsafe impl Send for LockedLocalApic {}
unsafe impl Sync for LockedLocalApic {}

pub static LAPIC: Lazy<LockedLocalApic> = Lazy::new(|| unsafe {
    let physical_address = PhysAddr::new(ACPI.apic.local_apic_address);
    let virtual_address = convert_physical_to_virtual(physical_address);

    <MemoryManager>::map_range_to(
        virtual_address,
        PhysFrame::containing_address(physical_address),
        0x1000,
        MappingType::KernelData.flags(),
        &mut KERNEL_PAGE_TABLE.lock(),
    )
    .unwrap();

    let mut lapic = LocalApicBuilder::new()
        .timer_vector(InterruptIndex::Timer as usize)
        .timer_mode(TimerMode::OneShot)
        .timer_initial(0)
        .error_vector(InterruptIndex::ApicError as usize)
        .spurious_vector(InterruptIndex::ApicSpurious as usize)
        .set_xapic_base(virtual_address.as_u64())
        .build()
        .unwrap_or_else(|err| panic!("Failed to build local APIC: {:#?}", err));

    lapic.enable();

    LockedLocalApic(Mutex::new(lapic))
});

pub fn end_of_interrupt() {
    unsafe {
        LAPIC.lock().end_of_interrupt();
    }
}

pub unsafe fn disable_pic() {
    Port::<u8>::new(0x21).write(0xff);
    Port::<u8>::new(0xa1).write(0xff);
}

pub unsafe fn calibrate_timer() {
    let mut lapic = LAPIC.lock();
    let mut lapic_total_ticks = 0;

    for _ in 0..TIMER_CALIBRATION_ITERATION {
        let last_time = HPET.elapsed();
        lapic.set_timer_initial(u32::MAX);
        while HPET.elapsed() - last_time < Duration::from_millis(1) {}
        lapic_total_ticks += u32::MAX - lapic.timer_current();
    }

    let average_ticks_per_ms = lapic_total_ticks / TIMER_CALIBRATION_ITERATION;
    let calibrated_timer_initial = average_ticks_per_ms * 1000 / TIMER_FREQUENCY_HZ;
    log::debug!("Calibrated timer initial: {}", calibrated_timer_initial);

    lapic.set_timer_mode(TimerMode::Periodic);
    lapic.set_timer_initial(calibrated_timer_initial);
    LAPIC_TIMER_INITIAL.store(calibrated_timer_initial, Ordering::Relaxed);
}
