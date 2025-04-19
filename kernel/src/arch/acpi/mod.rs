use acpi::fadt::Fadt;
use acpi::platform::interrupt::Apic;
use acpi::{AcpiError, AcpiHandler, PhysicalMapping};
use acpi::{AcpiTables, AmlTable, HpetInfo};
use acpi::{InterruptModel, PciConfigRegions};
use alloc::alloc::Global;
use alloc::boxed::Box;
use core::ptr::NonNull;
use limine::request::RsdpRequest;
use spin::Lazy;
use x86_64::PhysAddr;
use x86_64::instructions::port::Port;
use x86_64::structures::paging::PhysFrame;

use crate::mem::convert_physical_to_virtual;
use crate::mem::{KERNEL_PAGE_TABLE, MappingType, MemoryManager};

mod power;
pub use power::{reboot, shutdown};

#[used]
#[unsafe(link_section = ".requests")]
static RSDP_REQUEST: RsdpRequest = RsdpRequest::new();

pub static ACPI: Lazy<Acpi> = Lazy::new(|| init_acpi().unwrap());

pub struct Acpi<'a> {
    pub apic: Apic<'a, Global>,
    pub hpet_info: HpetInfo,
    pub pci_regions: PciConfigRegions<'a, Global>,
    pub fadt: Fadt,
    pub aml_table: AmlTable,
}

fn init_acpi() -> Result<Acpi<'static>, AcpiError> {
    let response = RSDP_REQUEST.get_response().unwrap();

    let acpi_tables = unsafe {
        let rsdp_address = response.address();
        let tables = AcpiTables::from_rsdp(AcpiMemHandler, rsdp_address)?;
        Box::leak(Box::new(tables))
    };

    let apic = match acpi_tables.platform_info()?.interrupt_model {
        InterruptModel::Apic(apic) => apic,
        InterruptModel::Unknown => panic!("No APIC support!"),
        _ => panic!("ACPI does not have interrupt model info!"),
    };

    let fadt = *acpi_tables.find_table::<Fadt>()?;

    let pm1a = fadt.pm1a_control_block().unwrap();
    let mut pm1a_port = Port::<u16>::new(pm1a.address as u16);

    unsafe {
        if fadt.smi_cmd_port != 0
            && (fadt.acpi_enable == 0 || fadt.acpi_disable == 0)
            && pm1a_port.read() & 1 == 0
        {
            Port::new(fadt.smi_cmd_port as u16).write(fadt.acpi_enable);
            while pm1a_port.read() & 1 == 0 {}
        }
    }

    Ok(Acpi {
        apic,
        hpet_info: HpetInfo::new(acpi_tables)?,
        pci_regions: PciConfigRegions::new(acpi_tables)?,
        fadt,
        aml_table: acpi_tables.dsdt()?,
    })
}

#[derive(Clone)]
struct AcpiMemHandler;

impl AcpiHandler for AcpiMemHandler {
    unsafe fn map_physical_region<T>(
        &self,
        physical_address: usize,
        size: usize,
    ) -> PhysicalMapping<Self, T> {
        let address = PhysAddr::new(physical_address as u64);
        let virtual_address = convert_physical_to_virtual(address);

        <MemoryManager>::map_range_to(
            virtual_address,
            PhysFrame::containing_address(address),
            size as u64,
            MappingType::KernelData.flags(),
            &mut KERNEL_PAGE_TABLE.lock(),
        )
        .unwrap();

        PhysicalMapping::new(
            physical_address,
            NonNull::new_unchecked(virtual_address.as_mut_ptr()),
            size,
            size,
            self.clone(),
        )
    }

    fn unmap_physical_region<T>(_region: &PhysicalMapping<Self, T>) {}
}
