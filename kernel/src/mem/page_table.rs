use alloc::vec;
use alloc::vec::Vec;
use x86_64::structures::paging::FrameAllocator;
use x86_64::structures::paging::FrameDeallocator;
use x86_64::structures::paging::PageSize;
use x86_64::structures::paging::PhysFrame;
use x86_64::structures::paging::Size4KiB;
use x86_64::structures::paging::mapper::*;
use x86_64::structures::paging::{PageTable, PageTableFlags};
use x86_64::{PhysAddr, VirtAddr};

use super::MappingType;
use super::{FRAME_ALLOCATOR, PHYSICAL_MEMORY_OFFSET};
use super::{convert_physical_to_virtual, convert_virtual_to_physical};

pub trait ExtendedPageTable {
    fn physical_address(&self) -> PhysAddr;
    unsafe fn write_to_mapped(&self, buffer: &[u8], address: VirtAddr);
    unsafe fn deep_copy(&self) -> OffsetPageTable<'static>;
    unsafe fn free_user_pages(&mut self);
}

impl ExtendedPageTable for OffsetPageTable<'_> {
    fn physical_address(&self) -> PhysAddr {
        let virtual_address = self.level_4_table() as *const _ as u64;
        convert_virtual_to_physical(VirtAddr::new(virtual_address))
    }

    unsafe fn write_to_mapped(&self, buffer: &[u8], address: VirtAddr) {
        let mut written: usize = 0;

        while written < buffer.len() {
            let current_address = address + written as u64;
            let page_offset = current_address.as_u64() % Size4KiB::SIZE;
            let remaining = (buffer.len() - written) as u64;
            let chunk_size = (Size4KiB::SIZE - page_offset).min(remaining) as usize;

            let physical_address = self
                .translate_addr(current_address)
                .expect("Failed to translate address!");
            let virtual_address = convert_physical_to_virtual(physical_address);

            core::ptr::copy_nonoverlapping(
                buffer[written..written + chunk_size].as_ptr(),
                virtual_address.as_mut_ptr::<u8>(),
                chunk_size,
            );
            written += chunk_size;
        }
    }

    unsafe fn free_user_pages(&mut self) {
        let frame_allocator = &mut FRAME_ALLOCATOR.lock();
        let mut table_frames_to_free: Vec<PhysFrame> = Vec::new();
        let mut stack = vec![(self.level_4_table_mut() as *mut PageTable, 4)];

        while let Some((table_ptr, current_level)) = stack.pop() {
            let table = &mut *table_ptr;

            let table_vaddr = VirtAddr::new(table_ptr as u64);
            let table_paddr = convert_virtual_to_physical(table_vaddr);
            let table_frame = PhysFrame::containing_address(table_paddr);
            table_frames_to_free.push(table_frame);

            for entry in table.iter_mut().filter(|entry| {
                !entry.is_unused() && !entry.flags().contains(PageTableFlags::HUGE_PAGE)
            }) {
                if current_level == 1 {
                    if entry.flags().contains(MappingType::UserCode.flags()) {
                        if let Ok(frame) = entry.frame() {
                            frame_allocator.deallocate_frame(frame);
                        }
                    }
                } else {
                    let child_address = convert_physical_to_virtual(entry.addr());
                    stack.push((child_address.as_mut_ptr(), current_level - 1));
                }
            }
        }

        for frame in table_frames_to_free.into_iter().rev() {
            frame_allocator.deallocate_frame(frame);
        }
    }

    unsafe fn deep_copy(&self) -> OffsetPageTable<'static> {
        let frame_allocator = &mut FRAME_ALLOCATOR.lock();

        let root_table_frame = frame_allocator
            .allocate_frame()
            .expect("Failed to allocate frame for root page table")
            .start_address();

        let target_root_vaddr = convert_physical_to_virtual(root_table_frame);
        let root_table: &mut PageTable = &mut *target_root_vaddr.as_mut_ptr();
        root_table.zero();

        let mut stack: Vec<(*const PageTable, *mut PageTable, u8)> =
            vec![(self.level_4_table() as *const _, root_table as *mut _, 4)];

        while let Some((source_table, target_table, level)) = stack.pop() {
            for (index, entry) in (*source_table)
                .iter()
                .enumerate()
                .filter(|(_, entry)| !entry.is_unused())
            {
                if level == 1 || entry.flags().contains(PageTableFlags::HUGE_PAGE) {
                    (&mut *target_table)[index].set_addr(entry.addr(), entry.flags());
                } else {
                    let target_child_frame = frame_allocator
                        .allocate_frame()
                        .expect("Failed to allocate frame for child page table")
                        .start_address();

                    let target_child_vaddr = convert_physical_to_virtual(target_child_frame);
                    let target_child_table = &mut *target_child_vaddr.as_mut_ptr::<PageTable>();
                    target_child_table.zero();
                    (&mut *target_table)[index].set_addr(target_child_frame, entry.flags());

                    let source_child_vaddr = convert_physical_to_virtual(entry.addr());
                    stack.push((source_child_vaddr.as_ptr(), target_child_table, level - 1));
                }
            }
        }

        OffsetPageTable::new(root_table, VirtAddr::new(*PHYSICAL_MEMORY_OFFSET))
    }
}
