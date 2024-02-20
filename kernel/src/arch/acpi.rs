use acpi::platform::interrupt::Apic;
use acpi::InterruptModel;
use acpi::{AcpiHandler, AcpiTables, HpetInfo, PhysicalMapping};
use alloc::alloc::Global;
use alloc::boxed::Box;
use bootloader_api::info::Optional;
use conquer_once::spin::OnceCell;
use core::ptr::NonNull;
use x86_64::PhysAddr;

use crate::memory::convert_physical_to_virtual;

pub static ACPI: OnceCell<Acpi> = OnceCell::uninit();

#[derive(Clone)]
struct AcpiMemHandler;

impl AcpiHandler for AcpiMemHandler {
    unsafe fn map_physical_region<T>(
        &self,
        physical_address: usize,
        size: usize,
    ) -> PhysicalMapping<Self, T> {
        let virtual_address = {
            let physical_address = PhysAddr::new(physical_address as u64);
            let virtual_address = convert_physical_to_virtual(physical_address);
            NonNull::new_unchecked(virtual_address.as_u64() as *mut T)
        };
        PhysicalMapping::new(physical_address, virtual_address, size, size, self.clone())
    }

    fn unmap_physical_region<T>(_region: &PhysicalMapping<Self, T>) {}
}

#[derive(Debug)]
pub struct Acpi<'a> {
    pub apic_info: Apic<'a, Global>,
    pub hpet_info: HpetInfo,
}

pub fn init(rsdp_addr: &Optional<u64>) {
    let acpi_tables = unsafe {
        let rsdp_addr = rsdp_addr.into_option().unwrap();
        let tables = AcpiTables::from_rsdp(AcpiMemHandler, rsdp_addr as usize);
        Box::leak(Box::new(tables.unwrap()))
    };

    log::info!("Find ACPI tables successfully!");

    let platform_info = acpi_tables
        .platform_info()
        .expect("Failed to get platform info!");

    let apic_info = match platform_info.interrupt_model {
        InterruptModel::Unknown => panic!("No APIC support, cannot continue!"),
        InterruptModel::Apic(apic) => apic,
        _ => panic!("ACPI does not have interrupt model info!"),
    };

    let hpet_info = HpetInfo::new(acpi_tables).expect("Failed to get HPET info!");

    ACPI.init_once(|| Acpi {
        apic_info,
        hpet_info,
    });
}
