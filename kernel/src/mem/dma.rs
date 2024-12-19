use x86_64::structures::paging::FrameDeallocator;
use x86_64::structures::paging::{PageSize, Size4KiB};
use x86_64::{PhysAddr, VirtAddr, structures::paging::PhysFrame};

use super::FRAME_ALLOCATOR;
use super::{convert_physical_to_virtual, convert_virtual_to_physical};

pub struct DmaManager;

impl DmaManager {
    pub const UNIT_SIZE: usize = Size4KiB::SIZE as usize;

    pub fn allocate(size: usize) -> (PhysAddr, VirtAddr) {
        let count = size.div_ceil(Self::UNIT_SIZE);

        let physical_address = FRAME_ALLOCATOR.lock().allocate_frames(count).unwrap();
        let physical_address = PhysAddr::new(physical_address.start_address().as_u64());

        let virtual_address = convert_physical_to_virtual(physical_address);
        (physical_address, virtual_address)
    }

    pub fn deallocate(virtual_address: VirtAddr) {
        let physical_address = convert_virtual_to_physical(virtual_address);
        let physical_address = PhysFrame::containing_address(physical_address);
        unsafe {
            FRAME_ALLOCATOR.lock().deallocate_frame(physical_address);
        }
    }
}
