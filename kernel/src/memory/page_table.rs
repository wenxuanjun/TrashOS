use x86_64::registers::control::Cr3;
use x86_64::structures::paging::mapper::*;
use x86_64::structures::paging::FrameAllocator;
use x86_64::structures::paging::{PageTable, PageTableFlags};
use x86_64::{PhysAddr, VirtAddr};

use super::{convert_physical_to_virtual, BitmapFrameAllocator, PHYSICAL_MEMORY_OFFSET};

pub trait ExtendedPageTable {
    unsafe fn new_from_allocate(frame_allocator: &mut BitmapFrameAllocator) -> Self;

    unsafe fn new_from_address(
        frame_allocator: &mut BitmapFrameAllocator,
        physical_address: PhysAddr,
    ) -> Self;

    unsafe fn new_from_recursion(
        frame_allocator: &mut BitmapFrameAllocator,
        source_page_table: &PageTable,
        target_page_table: &mut PageTable,
        page_table_level: u8,
    );

    unsafe fn ref_from_current() -> Self;

    fn write_to_mapped_address(
        &self,
        source_buffer: &[u8],
        target_address: VirtAddr,
    ) -> Result<(), &'static str>;

    fn physical_address(&self) -> PhysAddr;
}

impl ExtendedPageTable for OffsetPageTable<'_> {
    fn physical_address(&self) -> PhysAddr {
        let virtual_address = self.level_4_table() as *const _;
        PhysAddr::new(virtual_address as u64 - self.phys_offset().as_u64())
    }

    fn write_to_mapped_address(
        &self,
        source_buffer: &[u8],
        target_address: VirtAddr,
    ) -> Result<(), &'static str> {
        for (offset, &byte) in source_buffer.iter().enumerate() {
            let address = target_address + offset as u64;
            let physical_address = self
                .translate_addr(address)
                .ok_or("Failed to translate address!")?;
            let virtual_address = convert_physical_to_virtual(physical_address).as_u64();
            unsafe { (virtual_address as *mut u8).write(byte) }
        }
        Ok(())
    }

    unsafe fn new_from_allocate(frame_allocator: &mut BitmapFrameAllocator) -> Self {
        let page_table_address = BitmapFrameAllocator::allocate_frame(frame_allocator)
            .expect("Failed to allocate frame for page table")
            .start_address();

        let new_page_table =
            &mut *convert_physical_to_virtual(page_table_address).as_mut_ptr::<PageTable>();

        let physical_memory_offset = VirtAddr::new(PHYSICAL_MEMORY_OFFSET.clone());
        let page_table = OffsetPageTable::new(new_page_table, physical_memory_offset);

        page_table
    }

    unsafe fn new_from_address(
        frame_allocator: &mut BitmapFrameAllocator,
        physical_address: PhysAddr,
    ) -> Self {
        let source_page_table =
            &*convert_physical_to_virtual(physical_address).as_ptr::<PageTable>();
        let mut new_page_table = Self::new_from_allocate(frame_allocator);
        let target_page_table = new_page_table.level_4_table_mut();

        Self::new_from_recursion(frame_allocator, source_page_table, target_page_table, 4);
        new_page_table
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
            let mut new_page_table = Self::new_from_allocate(frame_allocator);
            let new_page_table_address = PhysAddr::new(new_page_table.physical_address().as_u64());
            target_page_table[index].set_addr(new_page_table_address, entry.flags());

            let source_page_table_next = &*convert_physical_to_virtual(entry.addr()).as_ptr();
            let target_page_table_next = new_page_table.level_4_table_mut();

            Self::new_from_recursion(
                frame_allocator,
                source_page_table_next,
                target_page_table_next,
                page_table_level - 1,
            );
        }
    }

    unsafe fn ref_from_current() -> Self {
        let physical_address = Cr3::read().0.start_address();
        let page_table = convert_physical_to_virtual(physical_address).as_mut_ptr::<PageTable>();
        let physical_memory_offset = VirtAddr::new(PHYSICAL_MEMORY_OFFSET.clone());
        OffsetPageTable::new(&mut *page_table, physical_memory_offset)
    }
}
