use alloc::sync::Arc;
use core::{slice, str};
use x86_64::VirtAddr;

use crate::memory::ref_current_page_table;
use crate::memory::{MappingType, MemoryManager};
use crate::task::scheduler::SCHEDULER;

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

    MemoryManager::alloc_range(
        VirtAddr::new(address as u64),
        length as u64,
        MappingType::UserData.flags(),
        unsafe { &mut ref_current_page_table() },
    )
    .expect("Failed to allocate memory for mmap");
}

pub fn exit() {
    let current_thread = SCHEDULER.lock().current_thread();

    if let Some(current_thread) = current_thread.upgrade() {
        let current_thread = current_thread.read();
        if let Some(process) = current_thread.process.upgrade() {
            let mut scheduler = SCHEDULER.lock();
            for thread in process.read().threads.iter() {
                scheduler.remove(Arc::downgrade(&thread));
            }
            process.read().exit_process();
        }
    }

    unsafe {
        loop {
            core::arch::asm!("sti", "2:", "hlt", "jmp 2b");
        }
    }
}
