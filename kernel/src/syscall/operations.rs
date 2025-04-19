use alloc::sync::Arc;
use core::arch::asm;
use core::time::Duration;
use core::{slice, str};
use x86_64::VirtAddr;

use crate::arch::interrupts::InterruptIndex;
use crate::mem::ref_current_page_table;
use crate::mem::{MappingType, MemoryManager};
use crate::task::scheduler::SCHEDULER;
use crate::task::timer::TIMER;

pub fn write(buffer: *const u8, length: usize) -> isize {
    if length == 0 {
        return 0;
    }

    if let Ok(string) = unsafe {
        let slice = slice::from_raw_parts(buffer, length);
        str::from_utf8(slice)
    } {
        crate::print!("{}", string);
    };

    length as isize
}

pub fn mmap(address: usize, length: usize) -> isize {
    if length == 0 {
        return 0;
    }

    match MemoryManager::alloc_range(
        VirtAddr::new(address as u64),
        length as u64,
        MappingType::UserData.flags(),
        &mut ref_current_page_table(),
    ) {
        Ok(_) => length as isize,
        Err(_) => -1,
    }
}

pub fn r#yield() -> isize {
    unsafe {
        asm!("int {}", const InterruptIndex::Timer as u8);
    }

    0
}

pub fn sleep(duration: u64) -> isize {
    let thread = SCHEDULER.lock().current();
    let Some(thread) = thread.upgrade() else {
        return -1;
    };

    TIMER.lock().add(Duration::from_millis(duration));
    thread.write().sleeping = true;

    r#yield()
}

pub fn exit() -> isize {
    let thread = SCHEDULER.lock().current();
    if let Some(process) = thread
        .upgrade()
        .and_then(|thread| thread.read().process.upgrade())
    {
        let mut scheduler = SCHEDULER.lock();
        for thread in process.read().threads.iter() {
            scheduler.remove(Arc::downgrade(thread));
        }
        process.read().exit();
    }

    r#yield()
}
