use alloc::vec::Vec;
use derive_more::{Deref, DerefMut};
use nvme::memory::Allocator;
use nvme::nvme::NvmeDevice;
use pci_types::device_type::DeviceType;
use spin::{Lazy, Mutex};
use x86_64::PhysAddr;
use x86_64::structures::paging::PhysFrame;

use super::pci::PCI_DEVICES;
use crate::mem::convert_physical_to_virtual;
use crate::mem::{DmaManager, KERNEL_PAGE_TABLE, MappingType, MemoryManager};

pub static NVME: Lazy<Mutex<NvmeManager>> = Lazy::new(|| {
    let devices = PCI_DEVICES.lock();

    let connections = devices
        .iter()
        .filter(|x| x.device_type == DeviceType::NvmeController)
        .filter(|x| x.bars[0].is_some())
        .map(|device| {
            let (address, size) = device.bars[0].unwrap().unwrap_mem();
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

            let mut nvme_device =
                NvmeDevice::init(virtual_address.as_u64() as usize, size, NvmeAllocator)
                    .expect("Failed to init NVMe device");

            nvme_device
                .identify_controller()
                .expect("Failed to identify NVMe controller");

            let list = nvme_device.identify_namespace_list(0);
            log::info!("Namespace list: {:?}", list);

            let namespace = nvme_device.identify_namespace(1);
            log::info!("Namespace: {:?}", namespace);

            nvme_device
        })
        .collect::<Vec<_>>();

    Mutex::new(NvmeManager(connections))
});

pub struct NvmeAllocator;

impl Allocator for NvmeAllocator {
    unsafe fn allocate(&self, size: usize) -> (usize, usize) {
        let address = DmaManager::allocate(size);
        (address.0.as_u64() as usize, address.1.as_u64() as usize)
    }
}

type Nvme = NvmeDevice<NvmeAllocator>;

#[derive(Deref, DerefMut)]
pub struct NvmeManager(Vec<Nvme>);

impl NvmeManager {
    pub fn read_block(&mut self, disk_id: usize, start_sector: u64, buffer: &mut [u8]) {
        self.get_mut(disk_id)
            .expect("Cannot find disk")
            .read_copied(buffer, start_sector)
            .expect("Cannot read from disk");
    }

    pub fn write_block(&mut self, disk_id: usize, start_sector: u64, buffer: &[u8]) {
        self.get_mut(disk_id)
            .expect("Cannot find disk")
            .write_copied(buffer, start_sector)
            .expect("Cannot write to disk");
    }

    pub fn get_disk_size(&mut self, disk_id: usize) -> usize {
        let device = self.get_mut(disk_id).expect("Cannot find disk");

        device
            .identify_namespace_list(0)
            .iter()
            .map(|x| device.identify_namespace(*x).1 as usize)
            .sum()
    }
}
