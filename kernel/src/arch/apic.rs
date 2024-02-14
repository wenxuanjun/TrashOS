use conquer_once::spin::OnceCell;
use spin::Mutex;
use x2apic::ioapic::{IoApic, IrqMode, RedirectionTableEntry};
use x2apic::lapic::{LocalApic, LocalApicBuilder};
use x86_64::{instructions::port::Port, PhysAddr};

use super::interrupts::InterruptIndex;
use crate::memory::{convert_physical_to_virtual, MemoryManager};

const IOAPIC_INTERRUPT_INDEX_OFFSET: u8 = 32;

pub static LAPIC: OnceCell<Mutex<LocalApic>> = OnceCell::uninit();
pub static IOAPIC: OnceCell<Mutex<IoApic>> = OnceCell::uninit();

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum IrqVector {
    Keyboard = 1,
    Mouse = 12,
}

pub fn init() {
    unsafe {
        disable_pic();
        init_apic();
        init_ioapic();
    };
    crate::info!("APIC initialized successfully!");
}

#[inline]
pub fn end_of_interrupt() {
    unsafe {
        LAPIC.try_get().unwrap().lock().end_of_interrupt();
    }
}

unsafe fn disable_pic() {
    Port::<u8>::new(0x21).write(0xff);
    Port::<u8>::new(0xa1).write(0xff);
}

unsafe fn init_apic() {
    let acpi = super::acpi::ACPI.try_get().unwrap();
    let physical_address = PhysAddr::new(acpi.apic_info.local_apic_address as u64);
    let virtual_address = convert_physical_to_virtual(physical_address);
    <MemoryManager>::map_exist(physical_address, virtual_address).unwrap();

    let mut lapic = LocalApicBuilder::new()
        .timer_vector(InterruptIndex::Timer as usize)
        .error_vector(InterruptIndex::ApicError as usize)
        .spurious_vector(InterruptIndex::ApicSpurious as usize)
        .set_xapic_base(virtual_address.as_u64())
        .build()
        .unwrap_or_else(|err| panic!("Failed to build local APIC: {:#?}", err));

    lapic.enable();

    LAPIC.init_once(|| Mutex::new(lapic));
}

unsafe fn init_ioapic() {
    let acpi = super::acpi::ACPI.try_get().unwrap();
    let physical_address = PhysAddr::new(acpi.apic_info.io_apics[0].address as u64);
    let virtual_address = convert_physical_to_virtual(physical_address);
    <MemoryManager>::map_exist(physical_address, virtual_address).unwrap();

    let mut ioapic = IoApic::new(virtual_address.as_u64());
    ioapic.init(IOAPIC_INTERRUPT_INDEX_OFFSET);
    IOAPIC.init_once(|| Mutex::new(ioapic));

    ioapic_add_entry(IrqVector::Keyboard, InterruptIndex::Keyboard);
    ioapic_add_entry(IrqVector::Mouse, InterruptIndex::Mouse);
}

unsafe fn ioapic_add_entry(irq: IrqVector, vector: InterruptIndex) {
    let lapic = LAPIC.try_get().unwrap().lock();
    let mut io_apic = IOAPIC.try_get().unwrap().lock();
    let mut entry = RedirectionTableEntry::default();
    entry.set_mode(IrqMode::Fixed);
    entry.set_dest(lapic.id() as u8);
    entry.set_vector(vector as u8);
    io_apic.set_table_entry(irq as u8, entry);
    io_apic.enable_irq(irq as u8);
}
