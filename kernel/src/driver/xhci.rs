use core::num::NonZeroUsize;
use x86_64::PhysAddr;
use x86_64::structures::paging::PhysFrame;
use xhci::accessor::Mapper;

use crate::mem::convert_physical_to_virtual;
use crate::mem::{KERNEL_PAGE_TABLE, MappingType, MemoryManager};

#[derive(Clone)]
pub struct XHCIMapper;

impl Mapper for XHCIMapper {
    unsafe fn map(&mut self, physical_start: usize, length: usize) -> NonZeroUsize {
        let physical_address = PhysAddr::new(physical_start as u64);
        let virtual_address = convert_physical_to_virtual(physical_address);

        MemoryManager::map_range_to(
            virtual_address,
            PhysFrame::containing_address(physical_address),
            length as u64,
            MappingType::KernelData.flags(),
            &mut KERNEL_PAGE_TABLE.lock(),
        )
        .unwrap();

        NonZeroUsize::new(virtual_address.as_u64() as usize).unwrap()
    }

    fn unmap(&mut self, _virt_start: usize, _bytes: usize) {}
}
