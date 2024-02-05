use alloc::alloc::Layout;
use good_memory_allocator::SpinLockedAllocator;
use x86_64::structures::paging::PageTableFlags;
use x86_64::VirtAddr;

use crate::memory::MemoryManager;

pub const HEAP_START: usize = 0x114514000000;
pub const HEAP_SIZE: usize = 1 * 1024 * 1024;

#[global_allocator]
static ALLOCATOR: SpinLockedAllocator = SpinLockedAllocator::empty();

#[alloc_error_handler]
fn alloc_error_handler(layout: Layout) -> ! {
    panic!("Global Allocation Error: {:?}", layout)
}

pub fn init_heap() {
    let heap_start = VirtAddr::new(HEAP_START as u64);
    let heap_end = heap_start + HEAP_SIZE as u64 - 1u64;

    let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;
    let mut page_table = crate::memory::KERNEL_PAGE_TABLE.try_get().unwrap().lock();
    <MemoryManager>::alloc_range(heap_start, heap_end, flags, &mut page_table).unwrap();

    unsafe {
        ALLOCATOR.init(HEAP_START, HEAP_SIZE);
    }
}
