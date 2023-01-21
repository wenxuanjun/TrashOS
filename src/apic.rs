use spin::Mutex;
use conquer_once::spin::OnceCell;
use acpi::platform::interrupt::Apic;
use x86_64::{instructions::port::Port};
use x2apic::lapic::{LocalApic, LocalApicBuilder};
use x2apic::ioapic::{IoApic, IrqFlags, IrqMode, RedirectionTableEntry};
use crate::interrupts::InterruptIndex;

const IOAPIC_INTERRUPT_INDEX_OFFSET: u8 = 64;

pub static LAPIC: OnceCell<Mutex<LocalApic>> = OnceCell::uninit();
pub static IOAPIC: OnceCell<Mutex<IoApic>> = OnceCell::uninit();

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum IrqVector {
    Keyboard = 1,
    Mouse = 12,
}

pub fn init(apic: &Apic) {
    unsafe {
        disable_pic();
        init_apic(apic);
        init_ioapic(apic);
    };
    crate::info!("APIC initialized successfully!");
    x86_64::instructions::interrupts::enable();
}

unsafe fn disable_pic() {
    Port::<u8>::new(0xa1).write(0xff);
    Port::<u8>::new(0x21).write(0xff);
}

unsafe fn init_apic(apic: &Apic) {
    let physical_address = apic.local_apic_address;
    let phys_mem_offset = crate::memory::PHYS_MEM_OFFSET.try_get().unwrap().as_u64();
    let virtual_address = phys_mem_offset + physical_address;
    crate::memory::map_physical_to_virtual(physical_address, virtual_address);

    let mut lapic = LocalApicBuilder::new()
        .timer_vector(InterruptIndex::Timer as usize)
        .error_vector(InterruptIndex::ApicError as usize)
        .spurious_vector(InterruptIndex::ApicSpurious as usize)
        .set_xapic_base(virtual_address)
        .build()
        .unwrap_or_else(|err| panic!("Failed to build local APIC: {:#?}", err));
    crate::info!("Local APIC ID: {}, Version: {}", lapic.id(), lapic.version());

    lapic.enable();
    LAPIC.init_once(|| Mutex::new(lapic));
}

unsafe fn init_ioapic(apic: &Apic) {
    let physical_address = apic.io_apics[0].address as u64;
    let phys_mem_offset = crate::memory::PHYS_MEM_OFFSET.try_get().unwrap().as_u64();
    let virtual_address = phys_mem_offset + physical_address;
    crate::memory::map_physical_to_virtual(physical_address, virtual_address);

    let mut ioapic = IoApic::new(virtual_address);
    ioapic.init(IOAPIC_INTERRUPT_INDEX_OFFSET);
    crate::info!("IoApic ID: {}, Version: {}", ioapic.id(), ioapic.version());
    IOAPIC.init_once(|| Mutex::new(ioapic));

    io_apic_add_entry(IrqVector::Keyboard, InterruptIndex::Keyboard);
    io_apic_add_entry(IrqVector::Mouse, InterruptIndex::Mouse);
}

unsafe fn io_apic_add_entry(irq: IrqVector, vector: InterruptIndex) {
    let lapic = LAPIC.try_get().unwrap().lock();
    let mut io_apic = IOAPIC.try_get().unwrap().lock();
    let mut entry = RedirectionTableEntry::default();
    entry.set_mode(IrqMode::Fixed);
    entry.set_dest(lapic.id() as u8);
    entry.set_vector(vector as u8);
    entry.set_flags(IrqFlags::LEVEL_TRIGGERED | IrqFlags::LOW_ACTIVE | IrqFlags::MASKED);
    io_apic.set_table_entry(irq as u8, entry);
    io_apic.enable_irq(irq as u8);
}
