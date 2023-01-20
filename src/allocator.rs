use x86_64::VirtAddr;
use x86_64::structures::paging::{Mapper, FrameAllocator};
use x86_64::structures::paging::{Page, PageTableFlags, Size4KiB};
use linked_list_allocator::LockedHeap;

pub const HEAP_START: usize = 0x114514000000;
pub const HEAP_SIZE: usize = 1024 * 1024;

#[global_allocator]
static ALLOCATOR: LockedHeap = LockedHeap::empty();

#[alloc_error_handler]
fn alloc_error_handler(layout: alloc::alloc::Layout) -> ! {
    panic!("Allocation error: {:?}", layout)
}

pub fn init_heap(
    (mapper, frame_allocator): &mut (
        impl Mapper<Size4KiB>,
        impl FrameAllocator<Size4KiB>,
    ),
) {
    let page_range = {
        let heap_start = VirtAddr::new(HEAP_START as u64);
        let heap_end = heap_start + HEAP_SIZE - 1u64;
        let heap_start_page = Page::containing_address(heap_start);
        let heap_end_page = Page::containing_address(heap_end);
        Page::range_inclusive(heap_start_page, heap_end_page)
    };

    for page in page_range {
        let frame = frame_allocator
            .allocate_frame()
            .expect("Failed to allocate frame for heap!");
        
        let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;
        unsafe {
            mapper
                .map_to(page, frame, flags, frame_allocator)
                .expect("Failed to map heap page to frame!")
                .flush();
        }
    }

    unsafe {
        ALLOCATOR.lock().init(HEAP_START, HEAP_SIZE);
    }
}