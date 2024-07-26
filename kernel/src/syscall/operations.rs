use core::{slice, str};
use x86_64::{structures::paging::OffsetPageTable, VirtAddr};

use crate::memory::{ExtendedPageTable, MappingType, MemoryManager};

pub fn write(buffer: *const u8, length: usize) {
    if length == 0 {
        return;
    }

    if let Ok(string) = unsafe {
        let slice = slice::from_raw_parts(buffer, length);
        str::from_utf8(slice)
    } {
        crate::print!("{}", string);
    };
}

pub fn malloc(address: usize, length: usize) {
    if length == 0 {
        return;
    }

    MemoryManager::alloc_range(
        VirtAddr::new(address as u64),
        length as u64,
        MappingType::UserData.flags(),
        &mut unsafe { OffsetPageTable::ref_from_current() },
    )
    .expect("Failed to allocate memory for mmap");
}
