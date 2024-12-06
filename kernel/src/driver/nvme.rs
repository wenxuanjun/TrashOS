use alloc::vec::Vec;
use nvme::memory::Allocator;
use nvme::nvme::NvmeDevice;
use pci_types::device_type::DeviceType;
use spin::{Lazy, Mutex};
use x86_64::structures::paging::PhysFrame;
use x86_64::PhysAddr;

use super::pci::PCI_DEVICES;
use crate::mem::convert_physical_to_virtual;
use crate::mem::{DmaManager, MappingType, MemoryManager, KERNEL_PAGE_TABLE};

pub static NVME: Lazy<Mutex<NvmeManager>> = Lazy::new(|| Mutex::new(NvmeManager::default()));

pub struct NvmeAllocator;

impl Allocator for NvmeAllocator {
    unsafe fn allocate(&self, size: usize) -> (usize, usize) {
        let address = DmaManager::allocate(size);
        (address.0.as_u64() as usize, address.1.as_u64() as usize)
    }
}

pub struct NvmeManager(Vec<NvmeDevice<NvmeAllocator>>);

impl NvmeManager {
    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn get_disk(&mut self, index: usize) -> Option<&mut NvmeDevice<NvmeAllocator>> {
        self.0.get_mut(index)
    }

    pub fn read_block(&mut self, disk_id: usize, start_sector: u64, buffer: &mut [u8]) {
        let device = self.0.get_mut(disk_id).expect("Cannot find disk");
        device
            .read_copied(buffer, start_sector)
            .expect("Cannot read");
    }

    pub fn write_block(&mut self, disk_id: usize, start_sector: u64, buffer: &[u8]) {
        let device = self.0.get_mut(disk_id).expect("Cannot find disk");
        device
            .write_copied(buffer, start_sector)
            .expect("Cannot write");
    }

    pub fn get_disk_size(&mut self, disk_id: usize) -> usize {
        let device = self.0.get_mut(disk_id).expect("Cannot find disk");

        device
            .identify_namespace_list(0)
            .iter()
            .map(|x| device.identify_namespace(*x).1 as usize)
            .sum()
    }
}

impl Default for NvmeManager {
    fn default() -> Self {
        let mut devices = PCI_DEVICES.lock();
        let devices = devices
            .iter_mut()
            .filter(|x| x.device_type == DeviceType::NvmeController)
            .collect::<Vec<_>>();

        let mut connections = Vec::new();
        for device in devices {
            if let Some(bar) = device.bars[0] {
                let (address, size) = bar.unwrap_mem();
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

                connections.push(nvme_device);
            }
        }

        Self(connections)
    }
}
