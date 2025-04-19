use alloc::collections::btree_map::BTreeMap;
use alloc::sync::Arc;
use alloc::vec::Vec;
use nvme::{Allocator, Device, IoQueuePair, Namespace};
use pci_types::device_type::DeviceType;
use spin::{Lazy, Mutex};
use x86_64::structures::paging::{PhysFrame, Translate};
use x86_64::{PhysAddr, VirtAddr};

use super::pcie::PCI_DEVICES;
use crate::mem::convert_physical_to_virtual;
use crate::mem::{DmaManager, KERNEL_PAGE_TABLE, MappingType, MemoryManager};

type SharedNvmeDevice = Arc<Mutex<Device<NvmeAllocator>>>;
type LockedQueuePair = Mutex<IoQueuePair<NvmeAllocator>>;

pub struct NvmeAllocator;

impl Allocator for NvmeAllocator {
    unsafe fn allocate(&self, size: usize) -> usize {
        let (_, virtual_address) = DmaManager::allocate(size);
        virtual_address.as_u64() as usize
    }

    unsafe fn deallocate(&self, addr: usize) {
        DmaManager::deallocate(VirtAddr::new(addr as u64));
    }

    fn translate(&self, addr: usize) -> usize {
        let page_table = KERNEL_PAGE_TABLE.lock();
        let address = VirtAddr::new(addr as u64);
        page_table.translate_addr(address).unwrap().as_u64() as usize
    }
}

pub struct NvmeBlockDevice {
    pub namespace: Namespace,
    pub qpairs: BTreeMap<u16, LockedQueuePair>,
}

pub struct NvmeManager(Vec<SharedNvmeDevice>);

impl NvmeManager {
    pub fn iter(&self) -> impl Iterator<Item = Vec<NvmeBlockDevice>> {
        self.0.iter().map(|device| {
            let mut controller = device.lock();
            let namespaces = controller.identify_namespaces(0).unwrap();

            let mapper = |namespace: Namespace| {
                let qpair = controller
                    .create_io_queue_pair(namespace.clone(), 64)
                    .ok()?;

                Some(NvmeBlockDevice {
                    namespace,
                    qpairs: BTreeMap::from([(*qpair.id(), Mutex::new(qpair))]),
                })
            };

            namespaces.into_iter().filter_map(mapper).collect()
        })
    }
}

pub static NVME: Lazy<NvmeManager> = Lazy::new(|| {
    let mut connections = Vec::new();

    for device in PCI_DEVICES.lock().iter() {
        if device.device_type == DeviceType::NvmeController {
            let Some(bar) = device.bars.first() else {
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

            let virtual_address = virtual_address.as_u64() as usize;
            let device = Device::init(virtual_address, NvmeAllocator).unwrap();
            connections.push(Arc::new(Mutex::new(device)));
        }
    }

    NvmeManager(connections)
});
