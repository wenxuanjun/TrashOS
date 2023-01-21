use linked_list_allocator::LockedHeap;
use x86_64::structures::paging::{Mapper, FrameAllocator};
use x86_64::structures::paging::{Page, PageTableFlags};
use crate::memory::{MAPPER, FRAME_ALLOCATOR};

pub const HEAP_START: usize = 0x114514000000;
pub const HEAP_SIZE: usize = 4 * 1024 * 1024;

#[global_allocator]
static ALLOCATOR: LockedHeap = LockedHeap::empty();

#[alloc_error_handler]
fn alloc_error_handler(layout: alloc::alloc::Layout) -> ! {
    panic!("Allocation error: {:?}", layout)
}

pub fn init_heap() {
    let page_range = {
        let heap_start = x86_64::VirtAddr::new(HEAP_START as u64);
        let heap_end = heap_start + HEAP_SIZE - 1u64;
        let heap_start_page = Page::containing_address(heap_start);
        let heap_end_page = Page::containing_address(heap_end);
        Page::range_inclusive(heap_start_page, heap_end_page)
    };

    let mut mapper = MAPPER.get().unwrap().lock();
    let mut frame_allocator = FRAME_ALLOCATOR.get().unwrap().lock();

    for page in page_range {
        let frame = frame_allocator
            .allocate_frame()
            .expect("Failed to allocate frame for heap!");
        
        let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;
        unsafe {
            mapper
                .map_to(page, frame, flags, &mut *frame_allocator)
                .expect("Failed to map heap page to frame!")
                .flush();
        }
    }

    unsafe {
        ALLOCATOR.lock().init(HEAP_START as *mut u8, HEAP_SIZE);
    }
}