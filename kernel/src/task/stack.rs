use alloc::boxed::Box;
use x86_64::{structures::paging::PageTableFlags, VirtAddr};

use crate::memory::{GeneralPageTable, MemoryManager};

const KERNEL_STACK_SIZE: usize = 16 * 1024;
const USER_STACK_SIZE: usize = 64 * 1024;
const USER_STACK_ADDRESS: usize = 0x0000_7fff_feff_f000;

pub struct KernelStack(Box<[u8]>);

impl KernelStack {
    pub fn new() -> Self {
        Self(Box::from(alloc::vec![0; KERNEL_STACK_SIZE]))
    }

    pub fn end_address(&self) -> VirtAddr {
        VirtAddr::new(self.0.as_ptr_range().end as u64)
    }
}

pub struct UserStack {
    pub start_address: VirtAddr,
    pub end_address: VirtAddr,
}

impl UserStack {
    pub fn new(page_table: &mut GeneralPageTable) -> Self {
        let user_stack_end = VirtAddr::new(USER_STACK_ADDRESS as u64);
        let user_stack_start = user_stack_end - USER_STACK_SIZE as u64;

        let flags =
            PageTableFlags::PRESENT | PageTableFlags::WRITABLE | PageTableFlags::USER_ACCESSIBLE;

        <MemoryManager>::alloc_range(user_stack_start, USER_STACK_SIZE as u64, flags, page_table)
            .unwrap();

        Self {
            start_address: user_stack_start,
            end_address: user_stack_end,
        }
    }
}
