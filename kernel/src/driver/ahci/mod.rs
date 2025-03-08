use alloc::vec::Vec;
use derive_more::{Deref, DerefMut};
use pci_types::device_type::DeviceType;
use spin::{Lazy, Mutex};
use x86_64::PhysAddr;

use super::pci::PCI_DEVICES;
use crate::mem::convert_physical_to_virtual;

pub mod cmd;
pub mod driver;
pub mod hba;
pub mod identify;

pub use driver::Ahci;
pub use hba::HbaMemory;

#[derive(Deref, DerefMut)]
pub struct AhciManager(Vec<Ahci>);

pub static AHCI: Lazy<Mutex<AhciManager>> = Lazy::new(|| {
    let devices = PCI_DEVICES.lock();
    let connections = devices
        .iter()
        .filter(|d| d.device_type == DeviceType::SataController)
        .flat_map(|d| d.bars.get(5).and_then(|b| b.as_ref()))
        .flat_map(|bar| {
            let (address, _) = bar.unwrap_mem();
            let physical_address = PhysAddr::new(address as u64);
            let virtual_address = convert_physical_to_virtual(physical_address);
            Ahci::new(virtual_address)
        })
        .collect();

    Mutex::new(AhciManager(connections))
});
