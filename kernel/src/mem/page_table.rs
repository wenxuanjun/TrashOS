use x86_64::structures::paging::FrameAllocator;
use x86_64::structures::paging::FrameDeallocator;
use x86_64::structures::paging::PhysFrame;
use x86_64::structures::paging::mapper::*;
use x86_64::structures::paging::{PageTable, PageTableFlags};
use x86_64::{PhysAddr, VirtAddr};

use super::FRAME_ALLOCATOR;
use super::MappingType;
use super::{BitmapFrameAllocator, PHYSICAL_MEMORY_OFFSET, convert_physical_to_virtual};

pub trait ExtendedPageTable {
    fn physical_address(&self) -> PhysAddr;
    fn write_to_mapped_address(&self, buffer: &[u8], address: VirtAddr);
    unsafe fn deep_copy(&self) -> OffsetPageTable<'static>;
    unsafe fn free_user_page_table(&self);
}

impl ExtendedPageTable for OffsetPageTable<'_> {
    fn physical_address(&self) -> PhysAddr {
        let virtual_address = self.level_4_table() as *const _;
        PhysAddr::new(virtual_address as u64 - self.phys_offset().as_u64())
    }

    fn write_to_mapped_address(&self, buffer: &[u8], address: VirtAddr) {
        for (offset, &byte) in buffer.iter().enumerate() {
            let address = address + offset as u64;
            let physical_address = self
                .translate_addr(address)
                .expect("Failed to translate address!");
            let virtual_address = convert_physical_to_virtual(physical_address).as_u64();
            unsafe { (virtual_address as *mut u8).write(byte) }
        }
    }

    unsafe fn deep_copy(&self) -> OffsetPageTable<'static> {
        let virtual_address = convert_physical_to_virtual(self.physical_address());
        let source_table = &*virtual_address.as_ptr::<PageTable>();

        let mut frame_allocator = FRAME_ALLOCATOR.lock();
        let mut new_page_table = new_from_allocate(&mut frame_allocator);
        let target_table = new_page_table.level_4_table_mut();

        new_from_recursion(&mut frame_allocator, source_table, target_table, 4);
        new_page_table
    }

    unsafe fn free_user_page_table(&self) {
        let mut frame_allocator = FRAME_ALLOCATOR.lock();
        free_from_recursion(&mut frame_allocator, self.physical_address(), 4);
    }
}

unsafe fn new_from_allocate(
    frame_allocator: &mut BitmapFrameAllocator,
) -> OffsetPageTable<'static> {
    let page_table_address = BitmapFrameAllocator::allocate_frame(frame_allocator)
        .expect("Failed to allocate frame for page table")
        .start_address();

    let new_page_table =
        &mut *convert_physical_to_virtual(page_table_address).as_mut_ptr::<PageTable>();

    let physical_memory_offset = VirtAddr::new(*PHYSICAL_MEMORY_OFFSET);
    let page_table = OffsetPageTable::new(new_page_table, physical_memory_offset);

    page_table
}

unsafe fn new_from_recursion(
    frame_allocator: &mut BitmapFrameAllocator,
    source_page_table: &PageTable,
    target_page_table: &mut PageTable,
    page_table_level: u8,
) {
    for (index, entry) in source_page_table.iter().enumerate() {
        if (page_table_level == 1)
            || entry.is_unused()
            || entry.flags().contains(PageTableFlags::HUGE_PAGE)
        {
            target_page_table[index].set_addr(entry.addr(), entry.flags());
            continue;
        }
        let mut new_page_table = new_from_allocate(frame_allocator);
        let new_page_table_address = PhysAddr::new(new_page_table.physical_address().as_u64());
        target_page_table[index].set_addr(new_page_table_address, entry.flags());

        let source_page_table_next = &*convert_physical_to_virtual(entry.addr()).as_ptr();
        let target_page_table_next = new_page_table.level_4_table_mut();

        new_from_recursion(
            frame_allocator,
            source_page_table_next,
            target_page_table_next,
            page_table_level - 1,
        );
    }
}

unsafe fn free_from_recursion(
    frame_allocator: &mut BitmapFrameAllocator,
    physical_address: PhysAddr,
    page_table_level: u8,
) {
    if page_table_level == 0 {
        frame_allocator.deallocate_frame(PhysFrame::containing_address(physical_address));
        return;
    }

    let virtual_address = convert_physical_to_virtual(physical_address);
    let page_table = &mut *(virtual_address.as_mut_ptr::<PageTable>());

    for entry in page_table.iter() {
        if entry.is_unused() {
            continue;
        }

        if page_table_level == 1 || entry.flags().contains(PageTableFlags::HUGE_PAGE) {
            if entry.flags().contains(MappingType::UserCode.flags()) {
                if let Ok(frame) = entry.frame() {
                    frame_allocator.deallocate_frame(frame);
                }
            }
        } else {
            free_from_recursion(frame_allocator, entry.addr(), page_table_level - 1);
        }
    }

    frame_allocator.deallocate_frame(PhysFrame::containing_address(physical_address));
}
