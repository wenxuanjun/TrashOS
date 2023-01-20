use spin::Mutex;
use bootloader_api::BootInfo;
use acpi::platform::interrupt::Apic;
use crate::interrupts::InterruptIndex;
use x2apic::lapic::{LocalApic, LocalApicBuilder};
use x86_64::{instructions::port::Port};

pub static LAPIC: Mutex<Option<LocalApic>> = Mutex::new(None);

pub fn init(boot_info: &'static BootInfo, apic: &Apic) {
    unsafe {
        // Disable 8259 immediately
        let mut cmd_8259a = Port::<u8>::new(0x20);
        let mut data_8259a = Port::<u8>::new(0x21);
        let mut cmd_8259b = Port::<u8>::new(0xa0);
        let mut data_8259b = Port::<u8>::new(0xa1);

        let mut spin_port = Port::<u8>::new(0x80);
        let mut spin = || spin_port.write(0);

        cmd_8259a.write(0x11);
        cmd_8259b.write(0x11);
        spin();

        data_8259a.write(0xf8);
        data_8259b.write(0xff);
        spin();

        data_8259a.write(0b100);
        spin();

        data_8259b.write(0b10);
        spin();

        data_8259a.write(0x1);
        data_8259b.write(0x1);
        spin();

        data_8259a.write(u8::MAX);
        data_8259b.write(u8::MAX);
    };

    let physical_address = apic.local_apic_address;
    let physical_memory_offset = boot_info.physical_memory_offset.into_option().unwrap();
    let virtual_address = physical_memory_offset + physical_address;
    crate::memory::map_physical_to_virtual(physical_address, virtual_address);

    let mut lapic = LocalApicBuilder::new()
        .timer_vector(InterruptIndex::Timer as usize)
        .error_vector(InterruptIndex::ApicError as usize)
        .spurious_vector(InterruptIndex::ApicSpurious as usize)
        .set_xapic_base(virtual_address)
        .build()
        .unwrap_or_else(|err| panic!("Failed to build local APIC: {:#?}", err));

    unsafe { lapic.enable(); }
    crate::info!("LAPIC initialized successfully!");
    
    LAPIC.lock().replace(lapic);
    x86_64::instructions::interrupts::enable();
}
