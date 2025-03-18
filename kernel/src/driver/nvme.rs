use alloc::vec::Vec;
use derive_more::{Deref, DerefMut};
use humansize::{BINARY, format_size};
use nvme::{Allocator, Device};
use pci_types::device_type::DeviceType;
use spin::{Lazy, Mutex};
use x86_64::PhysAddr;
use x86_64::structures::paging::PhysFrame;

use super::pci::PCI_DEVICES;
use crate::mem::{DmaManager, convert_physical_to_virtual};
use crate::mem::{KERNEL_PAGE_TABLE, MappingType, MemoryManager};

type Nvme = Device<NvmeAllocator>;

#[derive(Deref, DerefMut)]
pub struct NvmeManager(Vec<Nvme>);

pub static NVME: Lazy<Mutex<NvmeManager>> = Lazy::new(|| {
    let mut connections = Vec::new();

    for device in PCI_DEVICES.lock().iter() {
        if device.device_type == DeviceType::NvmeController {
            let Some(bar) = device.bars.get(0) else {
                continue;
            };
            let (address, size) = bar.unwrap().unwrap_mem();
            let physical_address = PhysAddr::new(address as u64);
            let virtual_address = convert_physical_to_virtual(physical_address);

            let _ = <MemoryManager>::map_range_to(
                virtual_address,
                PhysFrame::containing_address(physical_address),
                size as u64,
                MappingType::KernelData.flags(),
                &mut KERNEL_PAGE_TABLE.lock(),
            );

            let virtual_address = virtual_address.as_u64() as usize;
            let device = Device::init(virtual_address, NvmeAllocator).unwrap();
            connections.push(device);
        }
    }

    Mutex::new(NvmeManager(connections))
});

pub struct NvmeAllocator;

impl Allocator for NvmeAllocator {
    unsafe fn allocate(&self, size: usize) -> (usize, usize) {
        let address = DmaManager::allocate(size);
        (address.0.as_u64() as usize, address.1.as_u64() as usize)
    }
}

pub fn nvme_test() {
    let mut nvme_manager = NVME.lock();
    log::info!("NVMe disk count: {}", nvme_manager.len());
    let disk = nvme_manager.get_mut(0).unwrap();

    // let test_vec = Vec::<u8>::with_capacity(6000);
    // log::info!("Test vec address: {:?}", test_vec.as_ptr());

    let (model, serial, firmware) = disk.identify_controller().unwrap();
    log::info!("Model: {model}, Serial: {serial}, Firmware: {firmware}");

    let namespaces = disk.identify_namespace_list(0).unwrap();
    log::info!("NVMe namespaces: {:?}", namespaces);

    const TEST_LENGTH: usize = 16384;

    let namespace = disk.identify_namespace(namespaces[0]).unwrap();
    let disk_size = namespace.block_count * namespace.block_size;
    log::info!("NVMe disk size: {}", format_size(disk_size, BINARY));

    let mut qpair = disk.create_io_queue_pair(64).unwrap();

    let mut read_buffer = [0u8; TEST_LENGTH];
    qpair.read_copied(&mut read_buffer, 34).unwrap();
    crate::serial_println!("NVMe sector: {:?}", read_buffer);

    // let mut write_buffer = [0u8; TEST_LENGTH];
    // write_buffer[0] = 11;
    // write_buffer[1] = 45;
    // write_buffer[2] = 14;
    // qpair.write_copied(&write_buffer, 0).unwrap();

    // let mut read_buffer = [0u8; TEST_LENGTH];
    // qpair.read_copied(&mut read_buffer, 0).unwrap();
    // crate::serial_println!("NVMe sector: {:?}", read_buffer);
}
