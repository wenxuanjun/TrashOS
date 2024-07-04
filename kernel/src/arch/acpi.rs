use acpi::platform::interrupt::Apic;
use acpi::InterruptModel;
use acpi::{AcpiHandler, AcpiTables, HpetInfo, PhysicalMapping};
use alloc::alloc::Global;
use alloc::boxed::Box;
use conquer_once::spin::OnceCell;
use core::ptr::NonNull;
use limine::request::RsdpRequest;
use x86_64::{PhysAddr, VirtAddr};

use crate::memory::{convert_physical_to_virtual, convert_virtual_to_physical};

pub static ACPI: OnceCell<Acpi> = OnceCell::uninit();

#[used]
#[link_section = ".requests"]
static RSDP_REQUEST: RsdpRequest = RsdpRequest::new();

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
    pub apic: Apic<'a, Global>,
    pub hpet_info: HpetInfo,
}

pub fn init() {
    let rsdp_response = RSDP_REQUEST.get_response().unwrap();

    let acpi_tables = unsafe {
        let rsdp_addr = VirtAddr::new(rsdp_response.address() as u64);
        let tables = AcpiTables::from_rsdp(
            AcpiMemHandler,
            convert_virtual_to_physical(rsdp_addr).as_u64() as usize,
        );
        Box::leak(Box::new(tables.unwrap()))
    };

    log::info!("Find ACPI tables successfully!");

    let platform_info = acpi_tables
        .platform_info()
        .expect("Failed to get platform info!");

    let apic = match platform_info.interrupt_model {
        InterruptModel::Apic(apic) => apic,
        InterruptModel::Unknown => panic!("No APIC support, cannot continue!"),
        _ => panic!("ACPI does not have interrupt model info!"),
    };

    let hpet_info = HpetInfo::new(acpi_tables).expect("Failed to get HPET info!");

    ACPI.init_once(|| Acpi { apic, hpet_info });
}
