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
        &mut ref_current_page_table(),
    )
    .expect("Failed to allocate memory for mmap");
}

pub fn r#yield() {
    unsafe {
        asm!(
            "int {interrupt_number}",
            interrupt_number =
            const InterruptIndex::Timer as u8
        );
    }
}

pub fn sleep(duration: u64) {
    TIMER.lock().add(Duration::from_millis(duration));
    if let Some(thread) = SCHEDULER.lock().current_thread().upgrade() {
        thread.write().sleeping = true;
    }
    r#yield();
}

pub fn exit() {
    let current_thread = SCHEDULER.lock().current_thread();

    if let Some(current_thread) = current_thread.upgrade() {
        let current_thread = current_thread.read();
        if let Some(process) = current_thread.process.upgrade() {
            let mut scheduler = SCHEDULER.lock();
            for thread in process.read().threads.iter() {
                scheduler.remove(Arc::downgrade(thread));
            }
            process.read().exit_process();
        }
    }

    r#yield();
}
