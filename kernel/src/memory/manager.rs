use core::marker::PhantomData;
use x86_64::structures::paging::mapper::MapToError;
use x86_64::structures::paging::{FrameAllocator, PhysFrame};
use x86_64::structures::paging::{Mapper, PageTableFlags};
use x86_64::structures::paging::{Page, PageSize, Size4KiB};
use x86_64::{PhysAddr, VirtAddr};

use super::BootInfoFrameAllocator;
use super::GeneralPageTable;

pub struct MemoryManager<S: PageSize = Size4KiB> {
    size: PhantomData<S>,
}

impl<S: PageSize> MemoryManager<S> {
    pub fn alloc_range(
        start_address: VirtAddr,
        length: u64,
        flags: PageTableFlags,
        page_table: &mut GeneralPageTable,
    ) -> Result<(), MapToError<S>>
    where
        GeneralPageTable: Mapper<S>,
        BootInfoFrameAllocator: FrameAllocator<S>,
    {
        let page_range = {
            let start_page = Page::containing_address(start_address);
            let end_address = start_address + length.into() - 1u64;
            let end_page = Page::containing_address(end_address);
            Page::range_inclusive(start_page, end_page)
        };
        let mut frame_allocator = super::FRAME_ALLOCATOR.try_get().unwrap().lock();
        for page in page_range {
            let frame = frame_allocator
                .allocate_frame()
                .expect("Failed to allocate frame!");
            Self::map_frame_to_page(frame, page, flags, page_table, &mut *frame_allocator);
        }
        Ok(())
    }

    pub fn map_exist(
        physical_address: PhysAddr,
        virtual_address: VirtAddr,
    ) -> Result<(), MapToError<S>>
    where
        GeneralPageTable: Mapper<S>,
        BootInfoFrameAllocator: FrameAllocator<S>,
    {
        let page_table = &mut super::KERNEL_PAGE_TABLE.try_get().unwrap().lock();
        let mut frame_allocator = super::FRAME_ALLOCATOR.try_get().unwrap().lock();
        let page = Page::<S>::containing_address(virtual_address);
        let frame = PhysFrame::containing_address(physical_address);
        let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;
        Self::map_frame_to_page(frame, page, flags, page_table, &mut *frame_allocator);
        Ok(())
    }

    fn map_frame_to_page(
        frame: PhysFrame<S>,
        page: Page<S>,
        flags: PageTableFlags,
        page_table: &mut GeneralPageTable,
        frame_allocator: &mut BootInfoFrameAllocator,
    ) where
        GeneralPageTable: Mapper<S>,
        BootInfoFrameAllocator: FrameAllocator<S>,
    {
        let result = unsafe { page_table.map_to(page, frame, flags, frame_allocator) };
        match result {
            Ok(flush) => flush.flush(),
            Err(err) => match err {
                MapToError::FrameAllocationFailed => {
                    panic!("Failed to allocate frame for page table!");
                }
                MapToError::PageAlreadyMapped(frame) => {
                    crate::serial_println!("Already mapped to frame: {:?}", frame);
                }
                MapToError::ParentEntryHugePage => {
                    crate::serial_println!("Already mapped in a huge page!");
                }
            },
        };
    }
}
