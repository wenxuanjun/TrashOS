use spin::{Lazy, Mutex};
use x2apic::ioapic::{IoApic, RedirectionTableEntry};
use x86_64::PhysAddr;
use x86_64::structures::paging::PhysFrame;

use crate::arch::acpi::ACPI;
use crate::arch::interrupts::InterruptIndex;
use crate::mem::convert_physical_to_virtual;
use crate::mem::{KERNEL_PAGE_TABLE, MappingType, MemoryManager};

use super::lapic::LAPIC;

const IOAPIC_INTERRUPT_INDEX_OFFSET: u8 = 32;

pub static IOAPIC: Lazy<Mutex<IoApic>> = Lazy::new(|| unsafe {
    let physical_address = PhysAddr::new(ACPI.apic.io_apics[0].address as u64);
    let virtual_address = convert_physical_to_virtual(physical_address);

    log::debug!("IoAPIC address: {:#x}", virtual_address.as_u64());

    <MemoryManager>::map_range_to(
        virtual_address,
        PhysFrame::containing_address(physical_address),
        0x1000,
        MappingType::KernelData.flags(),
        &mut KERNEL_PAGE_TABLE.lock(),
    )
    .unwrap();

    let mut ioapic = IoApic::new(virtual_address.as_u64());
    ioapic.init(IOAPIC_INTERRUPT_INDEX_OFFSET);
    Mutex::new(ioapic)
});

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum IrqVector {
    Keyboard = 1,
    Mouse = 12,
    HpetTimer = 20,
}

pub unsafe fn ioapic_add_entry(irq: IrqVector, vector: InterruptIndex) {
    let mut entry = RedirectionTableEntry::default();
    entry.set_dest(LAPIC.lock().id() as u8);
    entry.set_vector(vector as u8);
    let mut ioapic = IOAPIC.lock();
    ioapic.set_table_entry(irq as u8, entry);
    ioapic.enable_irq(irq as u8);
}
