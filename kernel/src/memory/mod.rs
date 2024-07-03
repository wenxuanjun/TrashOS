use conquer_once::spin::OnceCell;
use frame::BitmapFrameAllocator;
use limine::request::{HhdmRequest, MemoryMapRequest};
use spin::Mutex;
use x86_64::{PhysAddr, VirtAddr};

mod frame;
mod kernel_heap;
mod manager;
mod page_table;

pub use manager::MemoryManager;
pub use page_table::GeneralPageTable;

pub static KERNEL_PAGE_TABLE: OnceCell<Mutex<GeneralPageTable>> = OnceCell::uninit();
pub static FRAME_ALLOCATOR: OnceCell<Mutex<BitmapFrameAllocator>> = OnceCell::uninit();
pub static PHYSICAL_MEMORY_OFFSET: OnceCell<u64> = OnceCell::uninit();

#[used]
#[link_section = ".requests"]
static HHDM_REQUEST: HhdmRequest = HhdmRequest::new();

#[used]
#[link_section = ".requests"]
static MEMORY_MAP_REQUEST: MemoryMapRequest = MemoryMapRequest::new();

pub fn init() {
    let hhdm = HHDM_REQUEST.get_response().unwrap();
    let memory_map = MEMORY_MAP_REQUEST.get_response().unwrap();

    let physical_memory_offset = hhdm.offset();
    PHYSICAL_MEMORY_OFFSET.init_once(|| physical_memory_offset);

    let frame_allocator = BitmapFrameAllocator::init(memory_map);
    FRAME_ALLOCATOR.init_once(|| Mutex::new(frame_allocator));

    let page_table = GeneralPageTable::ref_from_current(VirtAddr::new(physical_memory_offset));
    KERNEL_PAGE_TABLE.init_once(|| Mutex::new(page_table));

    kernel_heap::init_heap();
}

#[inline]
pub fn convert_physical_to_virtual(physical_address: PhysAddr) -> VirtAddr {
    let physical_memory_offset = PHYSICAL_MEMORY_OFFSET.try_get().unwrap();
    VirtAddr::new(physical_address.as_u64() + physical_memory_offset)
}

#[inline]
pub fn convert_virtual_to_physical(virtual_address: VirtAddr) -> PhysAddr {
    let physical_memory_offset = PHYSICAL_MEMORY_OFFSET.try_get().unwrap();
    PhysAddr::new(virtual_address.as_u64() - physical_memory_offset)
}

pub fn create_page_table_from_kernel() -> GeneralPageTable {
    let physical_memory_offset = PHYSICAL_MEMORY_OFFSET.try_get().unwrap();
    let mut frame_allocator = FRAME_ALLOCATOR.try_get().unwrap().lock();
    let page_table_address = KERNEL_PAGE_TABLE.try_get().unwrap().lock().physical_address;
    unsafe {
        GeneralPageTable::new_from(
            &mut frame_allocator,
            page_table_address,
            VirtAddr::new(*physical_memory_offset),
        )
    }
}
