use bootloader_api::info::{MemoryRegionKind, MemoryRegions};
use x86_64::structures::paging::Size4KiB;
use x86_64::structures::paging::{FrameAllocator, PageSize, PhysFrame};
use x86_64::PhysAddr;

pub struct BootInfoFrameAllocator {
    memory_map: &'static MemoryRegions,
    next: usize,
}

impl BootInfoFrameAllocator {
    pub fn init(memory_map: &'static MemoryRegions) -> Self {
        BootInfoFrameAllocator {
            memory_map,
            next: 0,
        }
    }

    fn usable_frames<T: PageSize>(&self) -> impl Iterator<Item = PhysFrame<T>> {
        let regions = self.memory_map.iter();
        let usable = regions.filter(|r| r.kind == MemoryRegionKind::Usable);
        let ranges = usable.map(|r| r.start..r.end);
        let frame_addresses = ranges.flat_map(|r| r.step_by(4096));
        frame_addresses.map(|a| PhysFrame::containing_address(PhysAddr::new(a)))
    }
}

unsafe impl FrameAllocator<Size4KiB> for BootInfoFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame<Size4KiB>> {
        let frame = self.usable_frames::<Size4KiB>().nth(self.next);
        self.next += 1;
        frame
    }
}

unsafe impl Send for BootInfoFrameAllocator {}
