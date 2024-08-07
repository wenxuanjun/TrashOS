use core::sync::atomic::{AtomicBool, AtomicU32, Ordering};

use spin::{Lazy, Mutex};
use x2apic::ioapic::{IoApic, IrqMode, RedirectionTableEntry};
use x2apic::lapic::{LocalApic, LocalApicBuilder, TimerMode};
use x86_64::{instructions::port::Port, PhysAddr};

use super::acpi::ACPI;
use super::interrupts::InterruptIndex;
use crate::device::hpet::HPET;
use crate::memory::convert_physical_to_virtual;

const TIMER_FREQUENCY_HZ: u32 = 250;
const TIMER_CALIBRATION_ITERATION: u32 = 100;
const IOAPIC_INTERRUPT_INDEX_OFFSET: u8 = 32;

pub static APIC_INIT: AtomicBool = AtomicBool::new(false);
pub static CALIBRATED_TIMER_INITIAL: AtomicU32 = AtomicU32::new(0);

pub static LAPIC: Lazy<Mutex<LocalApic>> = Lazy::new(|| unsafe {
    let physical_address = PhysAddr::new(ACPI.apic.local_apic_address as u64);
    let virtual_address = convert_physical_to_virtual(physical_address);

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

    Mutex::new(lapic)
});

pub static IOAPIC: Lazy<Mutex<IoApic>> = Lazy::new(|| unsafe {
    let physical_address = PhysAddr::new(ACPI.apic.io_apics[0].address as u64);
    let virtual_address = convert_physical_to_virtual(physical_address);

    let mut ioapic = IoApic::new(virtual_address.as_u64());
    ioapic.init(IOAPIC_INTERRUPT_INDEX_OFFSET);
    Mutex::new(ioapic)
});

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum IrqVector {
    Keyboard = 1,
    Mouse = 12,
}

pub fn init() {
    unsafe {
        disable_pic();
        calibrate_timer();

        ioapic_add_entry(IrqVector::Keyboard, InterruptIndex::Keyboard);
        ioapic_add_entry(IrqVector::Mouse, InterruptIndex::Mouse);
    };

    APIC_INIT.store(true, Ordering::SeqCst);
    log::info!("APIC initialized successfully!");
}

#[inline]
pub fn end_of_interrupt() {
    unsafe {
        LAPIC.lock().end_of_interrupt();
    }
}

unsafe fn disable_pic() {
    Port::<u8>::new(0x21).write(0xff);
    Port::<u8>::new(0xa1).write(0xff);
}

unsafe fn ioapic_add_entry(irq: IrqVector, vector: InterruptIndex) {
    let lapic = LAPIC.lock();
    let mut ioapic = IOAPIC.lock();
    let mut entry = RedirectionTableEntry::default();
    entry.set_mode(IrqMode::Fixed);
    entry.set_dest(lapic.id() as u8);
    entry.set_vector(vector as u8);
    ioapic.set_table_entry(irq as u8, entry);
    ioapic.enable_irq(irq as u8);
}

pub unsafe fn calibrate_timer() {
    let mut lapic = LAPIC.lock();
    let mut lapic_total_ticks = 0;

    for _ in 0..TIMER_CALIBRATION_ITERATION {
        let last_time = HPET.elapsed_ns();
        lapic.set_timer_initial(!0);
        while HPET.elapsed_ns() - last_time < 1_000_000 {}
        lapic_total_ticks += !0 - lapic.timer_current();
    }

    let average_clock_per_ms = lapic_total_ticks / TIMER_CALIBRATION_ITERATION;
    let calibrated_timer_initial = average_clock_per_ms * 1000 / TIMER_FREQUENCY_HZ;
    log::debug!("Calibrated timer initial: {}", calibrated_timer_initial);

    lapic.set_timer_mode(TimerMode::Periodic);
    lapic.set_timer_initial(calibrated_timer_initial);
    CALIBRATED_TIMER_INITIAL.store(calibrated_timer_initial, Ordering::SeqCst);
}
