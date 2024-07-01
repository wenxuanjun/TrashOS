use bit_field::BitField;
use bootloader_api::info::{MemoryRegionKind, MemoryRegions};
use x86_64::structures::paging::{FrameAllocator, PhysFrame};
use x86_64::structures::paging::{FrameDeallocator, Size4KiB};
use x86_64::PhysAddr;

use crate::memory::convert_physical_to_virtual;

pub struct Bitmap {
    inner: &'static mut [u8],
}

impl Bitmap {
    pub fn new(inner: &'static mut [u8]) -> Self {
        inner.fill(0);
        Self { inner }
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }

    pub fn get(&self, index: usize) -> bool {
        let byte = self.inner[index / 8];
        byte.get_bit(index % 8)
    }

    pub fn set(&mut self, index: usize, value: bool) {
        let byte = &mut self.inner[index / 8];
        byte.set_bit(index % 8, value);
    }
}

pub struct BitmapFrameAllocator {
    bitmap: Bitmap,
    usable_frames: usize,
    next_frame: usize,
}

impl BitmapFrameAllocator {
    pub fn init(memory_map: &'static mut MemoryRegions) -> Self {
        let memory_size = memory_map
            .iter()
            .map(|region| region.end)
            .max()
            .expect("No memory regions found!");

        let bitmap_size = (memory_size / 4096).div_ceil(8) as usize;

        let bitmap_address = memory_map
            .iter_mut()
            .find(|region| {
                region.kind == MemoryRegionKind::Usable
                    && (region.end - region.start) >= bitmap_size as u64
            })
            .map(|region| region.start)
            .expect("No suitable memory region for bitmap!");

        let bitmap_buffer = unsafe {
            let physical_address = PhysAddr::new(bitmap_address);
            let virtual_address = convert_physical_to_virtual(physical_address).as_u64();
            core::slice::from_raw_parts_mut(virtual_address as *mut u8, bitmap_size as usize)
        };

        let mut bitmap = Bitmap::new(bitmap_buffer);
        let mut usable_frames = 0;
        let mut next_frame = usize::MAX;

        for region in memory_map
            .iter()
            .filter(|region| region.kind == MemoryRegionKind::Usable)
        {
            let start_page_index = region.start.div_ceil(4096) as usize;
            let frame_count = ((region.end - region.start) / 4096) as usize;

            usable_frames += frame_count;
            next_frame = next_frame.min(start_page_index);

            for index in start_page_index..start_page_index + frame_count {
                bitmap.set(index, true);
            }
        }

        let bitmap_frame_start = (bitmap_address / 4096) as usize;
        let bitmap_frame_count = bitmap_size.div_ceil(4096);
        let bitmap_frame_end = bitmap_frame_start + bitmap_frame_count;

        assert!(next_frame <= bitmap_frame_start);
        if next_frame == bitmap_frame_start {
            next_frame = bitmap_frame_end + 1;
        }
        usable_frames -= bitmap_frame_count;
        (bitmap_frame_start..bitmap_frame_end).for_each(|index| bitmap.set(index, false));

        BitmapFrameAllocator {
            bitmap,
            usable_frames,
            next_frame,
        }
    }
}

unsafe impl FrameAllocator<Size4KiB> for BitmapFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame<Size4KiB>> {
        if self.usable_frames == 0 {
            log::error!("No more usable frames!");
            return None;
        }

        self.usable_frames -= 1;
        self.bitmap.set(self.next_frame, false);

        let address = self.next_frame * 4096;

        self.next_frame = (self.next_frame + 1..self.bitmap.len())
            .find(|&index| self.bitmap.get(index))
            .unwrap_or(self.bitmap.len());

        Some(PhysFrame::containing_address(PhysAddr::new(address as u64)))
    }
}

impl FrameDeallocator<Size4KiB> for BitmapFrameAllocator {
    unsafe fn deallocate_frame(&mut self, frame: PhysFrame<Size4KiB>) {
        let index = frame.start_address().as_u64() / 4096;
        self.bitmap.set(index as usize, true);
    }
}
