use core::sync::atomic::Ordering;

mod ioapic;
mod lapic;

use super::interrupts::InterruptIndex;
pub use ioapic::{IrqVector, ioapic_add_entry};
pub use lapic::{APIC_INIT, CALIBRATED_TIMER_INITIAL, LAPIC};
pub use lapic::{disable_pic, end_of_interrupt};

pub fn init() {
    unsafe {
        disable_pic();
        lapic::calibrate_timer();

        ioapic_add_entry(IrqVector::Keyboard, InterruptIndex::Keyboard);
        ioapic_add_entry(IrqVector::Mouse, InterruptIndex::Mouse);
        ioapic_add_entry(IrqVector::HpetTimer, InterruptIndex::HpetTimer);
    };

    APIC_INIT.store(true, Ordering::SeqCst);
    log::info!("APIC initialized successfully!");
}
