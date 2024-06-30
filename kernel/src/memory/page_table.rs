use super::BootInfoFrameAllocator;
use x86_64::registers::control::Cr3;
use x86_64::structures::paging::mapper::*;
use x86_64::structures::paging::page::PageRangeInclusive;
use x86_64::structures::paging::{FrameAllocator, FrameDeallocator};
use x86_64::structures::paging::{PhysFrame, PageTable, PageTableFlags};
use x86_64::structures::paging::{Page, Size1GiB, Size2MiB, Size4KiB};
use x86_64::{PhysAddr, VirtAddr};

#[derive(Debug)]
pub struct GeneralPageTable {
    pub inner: OffsetPageTable<'static>,
    pub physical_address: PhysAddr,
}

impl GeneralPageTable {
    pub unsafe fn new(
        frame_allocator: &mut BootInfoFrameAllocator,
        physical_memory_offset: VirtAddr,
    ) -> Self {
        let page_table_address: Option<PhysFrame<Size4KiB>> =
            BootInfoFrameAllocator::allocate_frame(frame_allocator);

        let page_table_address = page_table_address
            .expect("Failed to allocate frame for page table!")
            .start_address();

        let new_page_table = {
            let virtual_address = physical_memory_offset + page_table_address.as_u64();
            let page_table_pointer = virtual_address.as_u64() as *mut PageTable;
            page_table_pointer.as_mut().unwrap()
        };

        let page_table = OffsetPageTable::new(new_page_table, physical_memory_offset);

        GeneralPageTable {
            inner: page_table,
            physical_address: page_table_address,
        }
    }

    pub unsafe fn new_from(
        frame_allocator: &mut BootInfoFrameAllocator,
        physical_address: PhysAddr,
        physical_memory_offset: VirtAddr,
    ) -> GeneralPageTable {
        let source_page_table = {
            let physical_address = physical_address.as_u64();
            let physical_memory_offset = physical_memory_offset.as_u64();
            let page_table_address = physical_address + physical_memory_offset;
            (page_table_address as *mut PageTable).as_mut().unwrap()
        };

        let mut new_page_table = Self::new(frame_allocator, physical_memory_offset);
        let target_page_table = new_page_table.inner.level_4_table_mut();

        Self::new_from_recursion(
            frame_allocator,
            source_page_table,
            target_page_table,
            4,
            physical_memory_offset,
        );

        new_page_table
    }

    unsafe fn new_from_recursion(
        frame_allocator: &mut BootInfoFrameAllocator,
        source_page_table: &PageTable,
        target_page_table: &mut PageTable,
        page_table_level: u8,
        physical_memory_offset: VirtAddr,
    ) {
        for (index, entry) in source_page_table.iter().enumerate() {
            if (page_table_level == 1)
                || entry.is_unused()
                || entry.flags().contains(PageTableFlags::HUGE_PAGE)
            {
                target_page_table[index].set_addr(entry.addr(), entry.flags());
                continue;
            }
            let mut new_page_table = Self::new(frame_allocator, physical_memory_offset);
            let new_page_table_address = new_page_table.physical_address;
            target_page_table[index].set_addr(new_page_table_address, entry.flags());

            let source_page_table_next = {
                let virtual_address = physical_memory_offset + entry.addr().as_u64();
                unsafe { &*virtual_address.as_ptr() }
            };
            let target_page_table_next = new_page_table.inner.level_4_table_mut();

            Self::new_from_recursion(
                frame_allocator,
                source_page_table_next,
                target_page_table_next,
                page_table_level - 1,
                physical_memory_offset,
            );
        }
    }

    pub fn new_from_current(
        frame_allocator: &mut BootInfoFrameAllocator,
        physical_memory_offset: VirtAddr,
    ) -> Self {
        let physical_address = Cr3::read().0.start_address();
        unsafe { Self::new_from(frame_allocator, physical_address, physical_memory_offset) }
    }

    pub fn ref_from_current(physical_memory_offset: VirtAddr) -> Self {
        let physical_address = Cr3::read().0.start_address();
        let virtual_address = physical_memory_offset + physical_address.as_u64();
        let page_table_ptr = virtual_address.as_mut_ptr() as *mut PageTable;

        let offset_page_table = unsafe {
            let page_table = page_table_ptr.as_mut().unwrap();
            OffsetPageTable::new(page_table, physical_memory_offset)
        };

        Self {
            inner: offset_page_table,
            physical_address,
        }
    }

    pub unsafe fn switch(&self) {
        let page_table_frame = {
            let physical_address = self.physical_address;
            PhysFrame::containing_address(physical_address)
        };
        if page_table_frame != Cr3::read().0 {
            Cr3::write(page_table_frame, Cr3::read().1);
        }
    }
}

impl Mapper<Size1GiB> for GeneralPageTable {
    #[inline]
    unsafe fn map_to_with_table_flags<A>(
        &mut self,
        page: Page<Size1GiB>,
        frame: PhysFrame<Size1GiB>,
        flags: PageTableFlags,
        parent_table_flags: PageTableFlags,
        allocator: &mut A,
    ) -> Result<MapperFlush<Size1GiB>, MapToError<Size1GiB>>
    where
        A: FrameAllocator<Size4KiB> + ?Sized,
    {
        unsafe {
            self.inner
                .map_to_with_table_flags(page, frame, flags, parent_table_flags, allocator)
        }
    }

    #[inline]
    fn unmap(
        &mut self,
        page: Page<Size1GiB>,
    ) -> Result<(PhysFrame<Size1GiB>, MapperFlush<Size1GiB>), UnmapError> {
        self.inner.unmap(page)
    }

    #[inline]
    unsafe fn update_flags(
        &mut self,
        page: Page<Size1GiB>,
        flags: PageTableFlags,
    ) -> Result<MapperFlush<Size1GiB>, FlagUpdateError> {
        self.inner.update_flags(page, flags)
    }

    #[inline]
    unsafe fn set_flags_p4_entry(
        &mut self,
        page: Page<Size1GiB>,
        flags: PageTableFlags,
    ) -> Result<MapperFlushAll, FlagUpdateError> {
        self.inner.set_flags_p4_entry(page, flags)
    }

    #[inline]
    unsafe fn set_flags_p3_entry(
        &mut self,
        page: Page<Size1GiB>,
        flags: PageTableFlags,
    ) -> Result<MapperFlushAll, FlagUpdateError> {
        self.inner.set_flags_p3_entry(page, flags)
    }

    #[inline]
    unsafe fn set_flags_p2_entry(
        &mut self,
        page: Page<Size1GiB>,
        flags: PageTableFlags,
    ) -> Result<MapperFlushAll, FlagUpdateError> {
        self.inner.set_flags_p2_entry(page, flags)
    }

    #[inline]
    fn translate_page(&self, page: Page<Size1GiB>) -> Result<PhysFrame<Size1GiB>, TranslateError> {
        self.inner.translate_page(page)
    }
}

impl Mapper<Size2MiB> for GeneralPageTable {
    #[inline]
    unsafe fn map_to_with_table_flags<A>(
        &mut self,
        page: Page<Size2MiB>,
        frame: PhysFrame<Size2MiB>,
        flags: PageTableFlags,
        parent_table_flags: PageTableFlags,
        allocator: &mut A,
    ) -> Result<MapperFlush<Size2MiB>, MapToError<Size2MiB>>
    where
        A: FrameAllocator<Size4KiB> + ?Sized,
    {
        unsafe {
            self.inner
                .map_to_with_table_flags(page, frame, flags, parent_table_flags, allocator)
        }
    }

    #[inline]
    fn unmap(
        &mut self,
        page: Page<Size2MiB>,
    ) -> Result<(PhysFrame<Size2MiB>, MapperFlush<Size2MiB>), UnmapError> {
        self.inner.unmap(page)
    }

    #[inline]
    unsafe fn update_flags(
        &mut self,
        page: Page<Size2MiB>,
        flags: PageTableFlags,
    ) -> Result<MapperFlush<Size2MiB>, FlagUpdateError> {
        self.inner.update_flags(page, flags)
    }

    #[inline]
    unsafe fn set_flags_p4_entry(
        &mut self,
        page: Page<Size2MiB>,
        flags: PageTableFlags,
    ) -> Result<MapperFlushAll, FlagUpdateError> {
        self.inner.set_flags_p4_entry(page, flags)
    }

    #[inline]
    unsafe fn set_flags_p3_entry(
        &mut self,
        page: Page<Size2MiB>,
        flags: PageTableFlags,
    ) -> Result<MapperFlushAll, FlagUpdateError> {
        self.inner.set_flags_p3_entry(page, flags)
    }

    #[inline]
    unsafe fn set_flags_p2_entry(
        &mut self,
        page: Page<Size2MiB>,
        flags: PageTableFlags,
    ) -> Result<MapperFlushAll, FlagUpdateError> {
        self.inner.set_flags_p2_entry(page, flags)
    }

    #[inline]
    fn translate_page(&self, page: Page<Size2MiB>) -> Result<PhysFrame<Size2MiB>, TranslateError> {
        self.inner.translate_page(page)
    }
}

impl Mapper<Size4KiB> for GeneralPageTable {
    #[inline]
    unsafe fn map_to_with_table_flags<A>(
        &mut self,
        page: Page<Size4KiB>,
        frame: PhysFrame<Size4KiB>,
        flags: PageTableFlags,
        parent_table_flags: PageTableFlags,
        allocator: &mut A,
    ) -> Result<MapperFlush<Size4KiB>, MapToError<Size4KiB>>
    where
        A: FrameAllocator<Size4KiB> + ?Sized,
    {
        unsafe {
            self.inner
                .map_to_with_table_flags(page, frame, flags, parent_table_flags, allocator)
        }
    }

    #[inline]
    fn unmap(
        &mut self,
        page: Page<Size4KiB>,
    ) -> Result<(PhysFrame<Size4KiB>, MapperFlush<Size4KiB>), UnmapError> {
        self.inner.unmap(page)
    }

    #[inline]
    unsafe fn update_flags(
        &mut self,
        page: Page<Size4KiB>,
        flags: PageTableFlags,
    ) -> Result<MapperFlush<Size4KiB>, FlagUpdateError> {
        self.inner.update_flags(page, flags)
    }

    #[inline]
    unsafe fn set_flags_p4_entry(
        &mut self,
        page: Page<Size4KiB>,
        flags: PageTableFlags,
    ) -> Result<MapperFlushAll, FlagUpdateError> {
        self.inner.set_flags_p4_entry(page, flags)
    }

    #[inline]
    unsafe fn set_flags_p3_entry(
        &mut self,
        page: Page<Size4KiB>,
        flags: PageTableFlags,
    ) -> Result<MapperFlushAll, FlagUpdateError> {
        self.inner.set_flags_p3_entry(page, flags)
    }

    #[inline]
    unsafe fn set_flags_p2_entry(
        &mut self,
        page: Page<Size4KiB>,
        flags: PageTableFlags,
    ) -> Result<MapperFlushAll, FlagUpdateError> {
        self.inner.set_flags_p2_entry(page, flags)
    }

    #[inline]
    fn translate_page(&self, page: Page<Size4KiB>) -> Result<PhysFrame<Size4KiB>, TranslateError> {
        self.inner.translate_page(page)
    }
}

impl Translate for GeneralPageTable {
    #[inline]
    fn translate(&self, addr: VirtAddr) -> TranslateResult {
        self.inner.translate(addr)
    }
}

impl CleanUp for GeneralPageTable {
    #[inline]
    unsafe fn clean_up<D>(&mut self, frame_deallocator: &mut D)
    where
        D: FrameDeallocator<Size4KiB>,
    {
        self.inner.clean_up(frame_deallocator)
    }

    #[inline]
    unsafe fn clean_up_addr_range<D>(
        &mut self,
        range: PageRangeInclusive,
        frame_deallocator: &mut D,
    ) where
        D: FrameDeallocator<Size4KiB>,
    {
        self.inner.clean_up_addr_range(range, frame_deallocator)
    }
}
