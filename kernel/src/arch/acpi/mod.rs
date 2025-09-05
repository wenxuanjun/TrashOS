use acpi::aml::Interpreter;
use acpi::platform::AcpiPlatform;
use acpi::platform::InterruptModel;
use acpi::platform::PciConfigRegions;
use acpi::platform::interrupt::Apic;
use acpi::sdt::fadt::Fadt;
use acpi::{AcpiError, AcpiTables, HpetInfo};
use alloc::alloc::Global;
use limine::request::RsdpRequest;
use spin::Lazy;
use x86_64::instructions::port::Port;

mod handler;
mod power;

use handler::AcpiHandler;
pub use power::{reboot, shutdown};

#[used]
#[unsafe(link_section = ".requests")]
static RSDP_REQUEST: RsdpRequest = RsdpRequest::new();

pub static ACPI: Lazy<Acpi> = Lazy::new(|| init_acpi().unwrap());

pub struct Acpi {
    pub apic: Apic<Global>,
    pub hpet_info: HpetInfo,
    pub pci_regions: PciConfigRegions<Global>,
    pub fadt: Fadt,
    pub aml_engine: Interpreter<AcpiHandler>,
}

fn init_acpi() -> Result<Acpi, AcpiError> {
    let response = RSDP_REQUEST.get_response().unwrap();

    let platform_info = unsafe {
        let rsdp_address = response.address();
        let tables = AcpiTables::from_rsdp(AcpiHandler, rsdp_address)?;
        AcpiPlatform::new(tables, AcpiHandler)?
    };

    let aml_engine = Interpreter::new_from_platform(&platform_info)?;

    let apic = match platform_info.interrupt_model {
        InterruptModel::Apic(apic) => apic,
        InterruptModel::Unknown => panic!("No APIC support!"),
        _ => panic!("ACPI does not have interrupt model info!"),
    };

    let acpi_tables = &platform_info.tables;
    let fadt = *acpi_tables.find_table::<Fadt>().unwrap();
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
        aml_engine,
    })
}
