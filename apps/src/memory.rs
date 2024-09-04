use core::alloc::Layout;
use talc::{ClaimOnOom, Span, Talc, Talck};

use crate::syscall::malloc;

pub const HEAP_START: usize = 0x19198100000;
pub const HEAP_SIZE: usize = 1 * 1024 * 1024;

#[global_allocator]
static ALLOCATOR: Talck<spin::Mutex<()>, ClaimOnOom> =
    Talc::new(unsafe { ClaimOnOom::new(Span::empty()) }).lock();

#[alloc_error_handler]
fn alloc_error_handler(layout: Layout) -> ! {
    panic!("User heap allocation error: {:?}", layout)
}

pub fn init_heap() {
    malloc(HEAP_START, HEAP_SIZE);
    unsafe {
        let arena = Span::from_base_size(HEAP_START as *mut u8, HEAP_SIZE);
        ALLOCATOR.lock().claim(arena).unwrap();
    }
}
