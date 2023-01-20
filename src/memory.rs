use conquer_once::spin::OnceCell;
use bootloader_api::BootInfo;
use bootloader_api::info::{MemoryRegions, MemoryRegionKind};
use x86_64::{PhysAddr, VirtAddr};
use x86_64::structures::paging::{OffsetPageTable, PageTable};
use x86_64::structures::paging::{FrameAllocator, PhysFrame, Size4KiB};
use spin::Mutex;
use core::ops::DerefMut;
use x86_64::structures::paging::{Mapper};
use x86_64::structures::paging::{Page, PageTableFlags};

pub static MAPPER: OnceCell<Mutex<OffsetPageTable>> = OnceCell::uninit();
pub static FRAME_ALLOCATOR: OnceCell<Mutex<BootInfoFrameAllocator>> = OnceCell::uninit();

pub fn init(boot_info: &'static BootInfo) {
    let offset = boot_info.physical_memory_offset.clone();
    let phys_mem_offset = VirtAddr::new(offset.into_option().unwrap());
    unsafe {
        let page_table = active_page_table(phys_mem_offset);
        let mapper = OffsetPageTable::new(page_table, phys_mem_offset);
        let frame_allocator = BootInfoFrameAllocator::init(&boot_info.memory_regions);
        MAPPER.try_init_once(|| Mutex::new(mapper)).unwrap();
        FRAME_ALLOCATOR.try_init_once(|| Mutex::new(frame_allocator)).unwrap(); 
    }
}

pub fn map_physical_to_virtual(phys_addr: u64, virt_addr: u64) {
    let phys_addr = PhysAddr::new(phys_addr);
    let virt_addr = VirtAddr::new(virt_addr);
    let mut mapper = MAPPER.get().unwrap().lock();
    let mut frame_allocator = FRAME_ALLOCATOR.get().unwrap().lock();
    unsafe {
        mapper.map_to(
            Page::<Size4KiB>::containing_address(virt_addr),
            PhysFrame::containing_address(phys_addr),
            PageTableFlags::PRESENT | PageTableFlags::WRITABLE,
            frame_allocator.deref_mut(),
        ).unwrap().flush();
    }
}

unsafe fn active_page_table(phys_mem_offset: VirtAddr) -> &'static mut PageTable {
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

unsafe impl Send for BootInfoFrameAllocator {}