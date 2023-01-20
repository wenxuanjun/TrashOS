use bootloader_api::BootInfo;
use bootloader_api::info::{MemoryRegions, MemoryRegionKind};
use x86_64::{PhysAddr, VirtAddr};
use x86_64::structures::paging::{OffsetPageTable, PageTable};
use x86_64::structures::paging::{FrameAllocator, PhysFrame, Size4KiB};

pub fn init(boot_info: &'static BootInfo) -> (OffsetPageTable<'static>, BootInfoFrameAllocator) {
    let offset = boot_info.physical_memory_offset.clone();
    let phys_mem_offset = VirtAddr::new(offset.into_option().unwrap());
    unsafe {
        let page_table = active_page_table(phys_mem_offset);
        let mapper = OffsetPageTable::new(page_table, phys_mem_offset);
        let frame_allocator = BootInfoFrameAllocator::init(&boot_info.memory_regions);
        return (mapper, frame_allocator);
    }
}

pub unsafe fn active_page_table(phys_mem_offset: VirtAddr) -> &'static mut PageTable {
    use x86_64::registers::control::Cr3;
    let (page_table_frame, _) = Cr3::read();
    let phys = page_table_frame.start_address();
    let virt = phys_mem_offset + phys.as_u64();
    let page_table_ptr: *mut PageTable = virt.as_mut_ptr();
    return &mut *page_table_ptr;
}

pub struct BootInfoFrameAllocator {
    memory_map: &'static MemoryRegions,
    next: usize,
}

impl BootInfoFrameAllocator {
    pub unsafe fn init(memory_map: &'static MemoryRegions) -> Self {
        BootInfoFrameAllocator {
            memory_map: memory_map,
            next: 0,
        }
    }

    fn usable_frames(&self) -> impl Iterator<Item = PhysFrame> {
        let regions = self.memory_map.iter();
        let usable = regions.filter(|r| r.kind == MemoryRegionKind::Usable);
        let ranges = usable.map(|r| r.start..r.end);
        let frame_addresses = ranges.flat_map(|r| r.step_by(4096));
        frame_addresses.map(|a| PhysFrame::containing_address(PhysAddr::new(a)))
    }
}

unsafe impl FrameAllocator<Size4KiB> for BootInfoFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame> {
        let frame = self.usable_frames().nth(self.next);
        self.next += 1;
        return frame;
    }
}