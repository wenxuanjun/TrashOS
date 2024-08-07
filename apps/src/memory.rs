use core::alloc::Layout;
use good_memory_allocator::SpinLockedAllocator;

use crate::syscall::malloc;

pub const HEAP_START: usize = 0x19198100000;
pub const HEAP_SIZE: usize = 1 * 1024 * 1024;

#[global_allocator]
static ALLOCATOR: SpinLockedAllocator = SpinLockedAllocator::empty();

#[alloc_error_handler]
fn alloc_error_handler(layout: Layout) -> ! {
    panic!("User heap allocation error: {:?}", layout)
}

pub fn init_heap() {
    malloc(HEAP_START, HEAP_SIZE);
    unsafe {
        ALLOCATOR.init(HEAP_START, HEAP_SIZE);
    }
}
