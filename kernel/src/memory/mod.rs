use frame::BitmapFrameAllocator;
use limine::request::{HhdmRequest, MemoryMapRequest};
use spin::{Lazy, Mutex};
use x86_64::{structures::paging::OffsetPageTable, PhysAddr, VirtAddr};

mod frame;
mod kernel_heap;
mod manager;
mod page_table;

pub use kernel_heap::init_heap;
pub use manager::{MappingType, MemoryManager};
pub use page_table::ExtendedPageTable;

#[used]
#[link_section = ".requests"]
static HHDM_REQUEST: HhdmRequest = HhdmRequest::new();

#[used]
#[link_section = ".requests"]
static MEMORY_MAP_REQUEST: MemoryMapRequest = MemoryMapRequest::new();

pub static PHYSICAL_MEMORY_OFFSET: Lazy<u64> =
    Lazy::new(|| HHDM_REQUEST.get_response().unwrap().offset());

pub static FRAME_ALLOCATOR: Lazy<Mutex<BitmapFrameAllocator>> = Lazy::new(|| {
    let memory_map = MEMORY_MAP_REQUEST.get_response().unwrap();
    Mutex::new(BitmapFrameAllocator::init(memory_map))
});

pub static KERNEL_PAGE_TABLE: Lazy<Mutex<OffsetPageTable>> = Lazy::new(|| {
    let page_table = unsafe { OffsetPageTable::ref_from_current() };
    Mutex::new(page_table)
});

#[inline]
pub fn convert_physical_to_virtual(physical_address: PhysAddr) -> VirtAddr {
    VirtAddr::new(physical_address.as_u64() + PHYSICAL_MEMORY_OFFSET.clone())
}

#[inline]
pub fn convert_virtual_to_physical(virtual_address: VirtAddr) -> PhysAddr {
    PhysAddr::new(virtual_address.as_u64() - PHYSICAL_MEMORY_OFFSET.clone())
}

pub fn create_page_table_from_kernel() -> OffsetPageTable<'static> {
    let mut frame_allocator = FRAME_ALLOCATOR.lock();
    let page_table_address = KERNEL_PAGE_TABLE.lock().physical_address();
    unsafe { OffsetPageTable::new_from_address(&mut frame_allocator, page_table_address) }
}
