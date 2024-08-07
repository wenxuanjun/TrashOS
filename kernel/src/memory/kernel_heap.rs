use alloc::alloc::Layout;
use good_memory_allocator::SpinLockedAllocator;
use x86_64::VirtAddr;

use super::MappingType;
use super::KERNEL_PAGE_TABLE;
use crate::memory::MemoryManager;

pub const HEAP_START: usize = 0x114514000000;
pub const HEAP_SIZE: usize = 32 * 1024 * 1024;

#[global_allocator]
static ALLOCATOR: SpinLockedAllocator = SpinLockedAllocator::empty();

#[alloc_error_handler]
fn alloc_error_handler(layout: Layout) -> ! {
    panic!("Kernel heap allocation error: {:?}", layout)
}

pub fn init_heap() {
    let heap_start = VirtAddr::new(HEAP_START as u64);

    MemoryManager::alloc_range(
        heap_start,
        HEAP_SIZE as u64,
        MappingType::KernelData.flags(),
        &mut KERNEL_PAGE_TABLE.lock(),
    )
    .unwrap();

    unsafe {
        ALLOCATOR.init(HEAP_START, HEAP_SIZE);
    }
}
