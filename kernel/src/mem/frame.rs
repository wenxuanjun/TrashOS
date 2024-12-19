use core::fmt::{self, Display};
use humansize::{BINARY, format_size};
use limine::memory_map::EntryType;
use limine::response::MemoryMapResponse;
use x86_64::PhysAddr;
use x86_64::structures::paging::{FrameAllocator, PhysFrame};
use x86_64::structures::paging::{FrameDeallocator, Size4KiB};

use super::bitmap::Bitmap;
use super::convert_physical_to_virtual;

pub struct BitmapFrameAllocator {
    bitmap: Bitmap,
    origin_frames: usize,
    usable_frames: usize,
}

impl BitmapFrameAllocator {
    #[inline]
    fn used_bytes(&self) -> usize {
        (self.origin_frames - self.usable_frames) * 4096
    }

    #[inline]
    fn total_bytes(&self) -> usize {
        self.origin_frames * 4096
    }
}

impl Display for BitmapFrameAllocator {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{} used, {} total",
            format_size(self.used_bytes(), BINARY),
            format_size(self.total_bytes(), BINARY)
        )
    }
}

impl BitmapFrameAllocator {
    pub fn init(memory_map: &MemoryMapResponse) -> Self {
        let memory_size = memory_map
            .entries()
            .last()
            .map(|region| region.base + region.length)
            .expect("No memory regions found");

        let bitmap_size = (memory_size / 4096).div_ceil(8) as usize;

        let usable_regions = memory_map
            .entries()
            .iter()
            .filter(|region| region.entry_type == EntryType::USABLE);

        let bitmap_address = usable_regions
            .clone()
            .find(|region| region.length >= bitmap_size as u64)
            .map(|region| region.base)
            .expect("No suitable memory region for bitmap");

        let bitmap_buffer = unsafe {
            let physical_address = PhysAddr::new(bitmap_address);
            let virtual_address = convert_physical_to_virtual(physical_address).as_u64();
            let bitmap_inner_size = bitmap_size / size_of::<usize>();
            core::slice::from_raw_parts_mut(virtual_address as *mut usize, bitmap_inner_size)
        };

        let mut bitmap = Bitmap::new(bitmap_buffer);
        let mut origin_frames = 0;

        for region in usable_regions {
            let start_page_index = (region.base / 4096) as usize;
            let frame_count = (region.length / 4096) as usize;

            origin_frames += frame_count;
            bitmap.set_range(start_page_index, start_page_index + frame_count, true);
        }

        let bitmap_frame_start = (bitmap_address / 4096) as usize;
        let bitmap_frame_count = bitmap_size.div_ceil(4096);
        let bitmap_frame_end = bitmap_frame_start + bitmap_frame_count;

        let usable_frames = origin_frames - bitmap_frame_count;
        bitmap.set_range(bitmap_frame_start, bitmap_frame_end, false);

        BitmapFrameAllocator {
            bitmap,
            origin_frames,
            usable_frames,
        }
    }

    pub fn allocate_frames(&mut self, count: usize) -> Option<PhysFrame> {
        let index = self
            .bitmap
            .find_range(count, true)
            .expect("No more usable frames!");

        self.bitmap.set_range(index, index + count, false);
        self.usable_frames -= count;

        let address = PhysAddr::new(index as u64 * 4096);
        Some(PhysFrame::containing_address(address))
    }
}

unsafe impl FrameAllocator<Size4KiB> for BitmapFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame<Size4KiB>> {
        self.allocate_frames(1)
    }
}

impl FrameDeallocator<Size4KiB> for BitmapFrameAllocator {
    unsafe fn deallocate_frame(&mut self, frame: PhysFrame<Size4KiB>) {
        let index = frame.start_address().as_u64() / 4096;
        self.bitmap.set(index as usize, true);
        self.usable_frames += 1;
    }
}
