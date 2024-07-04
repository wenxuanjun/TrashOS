use core::{slice, str};
use x86_64::{structures::paging::PageTableFlags, VirtAddr};

use crate::memory::{GeneralPageTable, MemoryManager};

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

pub fn mmap(address: usize, length: usize) {
    if length == 0 {
        return;
    }

    let flags = PageTableFlags::PRESENT
        | PageTableFlags::WRITABLE
        | PageTableFlags::USER_ACCESSIBLE
        | PageTableFlags::NO_EXECUTE;

    let address = VirtAddr::new(address as u64);
    let mut page_table = unsafe { GeneralPageTable::ref_from_current() };

    MemoryManager::alloc_range(address, length as u64, flags, &mut page_table)
        .expect("Failed to allocate memory for mmap!");
}
