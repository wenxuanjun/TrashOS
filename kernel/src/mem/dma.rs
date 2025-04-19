use alloc::alloc::Layout;
use alloc::alloc::{alloc, dealloc};
use core::ops::{Deref, DerefMut};
use core::ptr::NonNull;
use core::slice::{from_raw_parts, from_raw_parts_mut};
use x86_64::structures::paging::FrameDeallocator;
use x86_64::structures::paging::{PageSize, Size4KiB};
use x86_64::{PhysAddr, VirtAddr, structures::paging::PhysFrame};

use super::FRAME_ALLOCATOR;
use super::{convert_physical_to_virtual, convert_virtual_to_physical};

pub struct AlignedBuffer {
    ptr: NonNull<u8>,
    layout: Layout,
}

impl AlignedBuffer {
    pub fn as_ptr(&self) -> *const u8 {
        self.ptr.as_ptr()
    }

    pub fn as_mut_ptr(&mut self) -> *mut u8 {
        self.ptr.as_ptr()
    }
}

impl AlignedBuffer {
    pub fn new(size: usize, align: usize) -> Option<Self> {
        let layout = Layout::from_size_align(size, align).ok()?;

        let ptr = NonNull::new(unsafe { alloc(layout) })?;
        Some(Self { ptr, layout })
    }
}

impl Drop for AlignedBuffer {
    fn drop(&mut self) {
        unsafe {
            dealloc(self.ptr.as_ptr(), self.layout);
        }
    }
}

impl Deref for AlignedBuffer {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        unsafe { from_raw_parts(self.ptr.as_ptr(), self.layout.size()) }
    }
}

impl DerefMut for AlignedBuffer {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { from_raw_parts_mut(self.ptr.as_ptr(), self.layout.size()) }
    }
}

pub struct DmaManager;

impl DmaManager {
    pub const UNIT_SIZE: usize = Size4KiB::SIZE as usize;

    pub fn allocate(size: usize) -> (PhysAddr, VirtAddr) {
        let count = size.div_ceil(Self::UNIT_SIZE);
        let address = FRAME_ALLOCATOR.lock().allocate_frames(count).unwrap();
        let physical_address = address.start_address();

        (physical_address, convert_physical_to_virtual(physical_address))
    }

    pub fn deallocate(address: VirtAddr) {
        let physical_address = convert_virtual_to_physical(address);
        let address = PhysFrame::containing_address(physical_address);
        unsafe { FRAME_ALLOCATOR.lock().deallocate_frame(address) };
    }
}
