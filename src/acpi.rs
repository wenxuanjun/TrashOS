use core::ptr::NonNull;
use acpi::InterruptModel;
use acpi::platform::interrupt::Apic;
use acpi::{AcpiHandler, AcpiTables, PhysicalMapping};
use alloc::alloc::Global;
use alloc::boxed::Box;
use bootloader_api::info::Optional;
use conquer_once::spin::OnceCell;

pub static ACPI: OnceCell<Acpi> = OnceCell::uninit();

#[derive(Clone)]
struct AcpiMemHandler;

impl AcpiHandler for AcpiMemHandler {
    unsafe fn map_physical_region<T>(
        &self,
        physical_address: usize,
        size: usize,
    ) -> PhysicalMapping<Self, T> {
        let notnull_address = {
            let physical_memory_offset = crate::memory::PHYSICAL_MEMORY_OFFSET.try_get().unwrap();
            let virtual_address = physical_memory_offset + physical_address as u64;
            NonNull::new_unchecked(virtual_address as *mut T)
        };
        PhysicalMapping::new(physical_address, notnull_address, size, size, self.clone())
    }

    fn unmap_physical_region<T>(_region: &PhysicalMapping<Self, T>) {}
}

#[derive(Debug)]
pub struct Acpi<'a> {
    pub apic_info: Apic<'a, Global>,
}

pub fn init(rsdp_addr: Optional<u64>) {
    let acpi_tables = unsafe {
        let rsdp_addr = rsdp_addr.into_option().unwrap();
        let tables = AcpiTables::from_rsdp(AcpiMemHandler, rsdp_addr as usize);
        Box::leak(Box::new(tables.unwrap()))
    };

    crate::info!("Find ACPI tables successfully!");
    let platform_info = acpi_tables.platform_info().expect("Failed to get platform info!");

    let apic_info = match platform_info.interrupt_model {
        InterruptModel::Unknown => panic!("No APIC support, cannot continue!"),
        InterruptModel::Apic(apic) => apic,
        _ => panic!("ACPI does not have interrupt model info!"),
    };
    
    ACPI.init_once(|| Acpi { apic_info });
}
