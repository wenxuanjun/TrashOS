use core::ptr::NonNull;
use bootloader_api::BootInfo;
use acpi::InterruptModel;
use acpi::platform::interrupt::Apic;
use acpi::{AcpiHandler, AcpiTables, PhysicalMapping};

#[derive(Clone)]
struct AcpiMemHandler;

impl AcpiHandler for AcpiMemHandler {
    unsafe fn map_physical_region<T>(
        &self,
        physical_address: usize,
        size: usize,
    ) -> PhysicalMapping<Self, T> {
        let phys_mem_offset = crate::memory::PHYS_MEM_OFFSET.try_get().unwrap();
        let virtual_address = phys_mem_offset + physical_address as u64;
        let notnull_address = NonNull::new_unchecked(virtual_address as *mut T);
        PhysicalMapping::new(physical_address, notnull_address, size, size, self.clone())
    }

    fn unmap_physical_region<T>(_region: &PhysicalMapping<Self, T>) {}
}

pub fn init(boot_info: &'static BootInfo) -> Apic {
    let rsdp_addr = boot_info.rsdp_addr.into_option().unwrap();
    let acpi_tables = unsafe { AcpiTables::from_rsdp(AcpiMemHandler, rsdp_addr as usize) }.unwrap();

    crate::info!("Find ACPI tables successfully!");
    let platform_info = acpi_tables.platform_info().expect("Failed to get platform info!");

    let apic_info = match platform_info.interrupt_model {
        InterruptModel::Unknown => panic!("No APIC support, cannot continue!"),
        InterruptModel::Apic(apic) => apic,
        _ => panic!("ACPI does not have interrupt model info!"),
    };
    return apic_info;
}
