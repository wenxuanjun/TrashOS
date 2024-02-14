use bootloader_api::info::{MemoryRegions, Optional};
use conquer_once::spin::OnceCell;
use frame::BootInfoFrameAllocator;
use spin::Mutex;
use x86_64::{PhysAddr, VirtAddr};

mod frame;
mod kernel_heap;
mod manager;
mod page_table;

pub use manager::MemoryManager;
pub use page_table::GeneralPageTable;

pub static KERNEL_PAGE_TABLE: OnceCell<Mutex<GeneralPageTable>> = OnceCell::uninit();
pub static FRAME_ALLOCATOR: OnceCell<Mutex<BootInfoFrameAllocator>> = OnceCell::uninit();
static PHYSICAL_MEMORY_OFFSET: OnceCell<u64> = OnceCell::uninit();

pub fn init(offset: &Optional<u64>, regions: &'static MemoryRegions) {
    let offset = offset.into_option().unwrap();
    PHYSICAL_MEMORY_OFFSET.init_once(|| offset);

    let frame_allocator = BootInfoFrameAllocator::init(regions);
    FRAME_ALLOCATOR.init_once(|| Mutex::new(frame_allocator));

    let page_table = GeneralPageTable::ref_from_current(VirtAddr::new(offset));
    KERNEL_PAGE_TABLE.init_once(|| Mutex::new(page_table));

    kernel_heap::init_heap();
}

#[inline]
pub fn convert_physical_to_virtual(physical_address: PhysAddr) -> VirtAddr {
    let physical_memory_offset = PHYSICAL_MEMORY_OFFSET.get().unwrap();
    VirtAddr::new(physical_address.as_u64() + physical_memory_offset)
}

pub fn create_page_table_from_kernel() -> GeneralPageTable {
    let physical_memory_offset = PHYSICAL_MEMORY_OFFSET.get().unwrap();
    let mut frame_allocator = FRAME_ALLOCATOR.get().unwrap().lock();
    let page_table_address = KERNEL_PAGE_TABLE.get().unwrap().lock().physical_address;
    unsafe {
        GeneralPageTable::new_from(
            &mut frame_allocator,
            page_table_address,
            VirtAddr::new(*physical_memory_offset),
        )
    }
}
