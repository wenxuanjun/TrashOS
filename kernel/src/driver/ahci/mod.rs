use alloc::{sync::Arc, vec::Vec};
use identify::IdentifyData;
use pci_types::device_type::DeviceType;
use spin::{Lazy, Mutex};
use x86_64::PhysAddr;
use x86_64::structures::paging::PhysFrame;

use super::pcie::PCI_DEVICES;
use crate::mem::convert_physical_to_virtual;
use crate::mem::{KERNEL_PAGE_TABLE, MappingType, MemoryManager};

pub mod cmd;
pub mod driver;
pub mod hba;
pub mod identify;

pub use driver::{Ahci, BLOCK_SIZE};
pub use hba::HbaMemory;

pub struct AhciBlockDevice {
    pub device: Arc<Mutex<Ahci>>,
    pub identify: IdentifyData,
}

impl AhciManager {
    pub fn iter(&self) -> impl Iterator<Item = AhciBlockDevice> {
        self.0.iter().map(|device| AhciBlockDevice {
            device: device.clone(),
            identify: device.lock().identity(),
        })
    }
}

pub struct AhciManager(Vec<Arc<Mutex<Ahci>>>);

pub static AHCI: Lazy<AhciManager> = Lazy::new(|| {
    let mut connections = Vec::new();

    for device in PCI_DEVICES.lock().iter() {
        if device.device_type == DeviceType::SataController {
            let Some(bar) = device.bars.get(5) else {
                continue;
            };
            let (address, size) = bar.unwrap().unwrap_mem();
            let physical_address = PhysAddr::new(address as u64);
            let virtual_address = convert_physical_to_virtual(physical_address);

            <MemoryManager>::map_range_to(
                virtual_address,
                PhysFrame::containing_address(physical_address),
                size as u64,
                MappingType::KernelData.flags(),
                &mut KERNEL_PAGE_TABLE.lock(),
            )
            .unwrap();

            for ahci_device in Ahci::new(virtual_address) {
                connections.push(Arc::new(Mutex::new(ahci_device)));
            }
        }
    }

    AhciManager(connections)
});
