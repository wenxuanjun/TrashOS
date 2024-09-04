use crate::memory::FRAME_ALLOCATOR;
use crate::memory::{convert_physical_to_virtual, convert_virtual_to_physical};
use x86_64::structures::paging::{FrameAllocator, FrameDeallocator};
use x86_64::{structures::paging::PhysFrame, PhysAddr, VirtAddr};

pub mod ahci;
pub mod display;
pub mod hpet;
pub mod keyboard;
pub mod log;
pub mod mouse;
pub mod pci;
pub mod rtc;
pub mod serial;
pub mod terminal;
pub mod xhci;

pub fn alloc_for_dma() -> (PhysAddr, VirtAddr) {
    let physical_address = FRAME_ALLOCATOR.lock().allocate_frame().unwrap();
    let physical_address = physical_address.start_address();
    let virtual_address = convert_physical_to_virtual(physical_address);
    (physical_address, virtual_address)
}

pub fn dealloc_for_dma(virtual_address: VirtAddr) {
    let physical_address = convert_virtual_to_physical(virtual_address);
    let physical_address = PhysFrame::containing_address(physical_address);
    unsafe {
        FRAME_ALLOCATOR.lock().deallocate_frame(physical_address);
    }
}
