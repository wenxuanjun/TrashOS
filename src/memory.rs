use bootloader_api::BootInfo;
use bootloader_api::info::{MemoryRegionKind, MemoryRegions};
use x86_64::{PhysAddr, VirtAddr};
use x86_64::structures::paging::{OffsetPageTable, PageTable};
use x86_64::structures::paging::{FrameAllocator, PhysFrame, Size4KiB};
use conquer_once::spin::OnceCell;
use spin::Mutex;

pub static MAPPER: OnceCell<Mutex<OffsetPageTable>> = OnceCell::uninit();
pub static FRAME_ALLOCATOR: OnceCell<Mutex<BootInfoFrameAllocator>> = OnceCell::uninit();
pub static PHYS_MEM_OFFSET: OnceCell<u64> = OnceCell::uninit();

pub fn init(boot_info: &'static BootInfo) {
    let offset = boot_info.physical_memory_offset.clone();
    let phys_mem_offset = VirtAddr::new(offset.into_option().unwrap());
    unsafe {
        let page_table = active_page_table(phys_mem_offset);
        let mapper = OffsetPageTable::new(page_table, phys_mem_offset);
        let frame_allocator = BootInfoFrameAllocator::init(&boot_info.memory_regions);
        MAPPER.init_once(|| Mutex::new(mapper));
        FRAME_ALLOCATOR.init_once(|| Mutex::new(frame_allocator));
        PHYS_MEM_OFFSET.init_once(|| phys_mem_offset.as_u64());
    }
}

#[macro_export]
macro_rules! map_physical_to_virtual {
    ($phys_addr:expr, $virt_addr:expr) => {
        use x86_64::{PhysAddr, VirtAddr};
        use x86_64::structures::paging::{Mapper, mapper::MapToError};
        use x86_64::structures::paging::{Page, Size4KiB, PageTableFlags, PhysFrame};
        let result = unsafe {
            $crate::memory::MAPPER.try_get().unwrap().lock().map_to(
                Page::<Size4KiB>::containing_address(VirtAddr::new($virt_addr)),
                PhysFrame::containing_address(PhysAddr::new($phys_addr)),
                PageTableFlags::PRESENT | PageTableFlags::WRITABLE,
                &mut *$crate::memory::FRAME_ALLOCATOR.try_get().unwrap().lock(),
            )
        };
        match result {
            Ok(flush) => flush.flush(),
            Err(err) => match err {
                MapToError::FrameAllocationFailed => {
                    panic!("Failed to allocate frame!");
                }
                MapToError::PageAlreadyMapped(frame) => {
                    crate::debug!("Already mapped to frame: {:?}", frame);
                }
                MapToError::ParentEntryHugePage => {
                    crate::debug!("Already mapped to huge page!");
                }
            },
        };
    };
}

unsafe fn active_page_table(phys_mem_offset: VirtAddr) -> &'static mut PageTable {
    use x86_64::registers::control::Cr3;
    let (page_table_frame, _) = Cr3::read();
    let physical_address = page_table_frame.start_address();
    let virtual_address = phys_mem_offset + physical_address.as_u64();
    let page_table_ptr: *mut PageTable = virtual_address.as_mut_ptr();
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

unsafe impl Send for BootInfoFrameAllocator {}
