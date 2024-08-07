use alloc::boxed::Box;
use x86_64::structures::paging::OffsetPageTable;
use x86_64::VirtAddr;

use crate::memory::{MappingType, MemoryManager};

const KERNEL_STACK_SIZE: usize = 64 * 1024;
const USER_STACK_END: usize = 0x7ffffefff000;
const USER_STACK_SIZE: usize = 256 * 1024;

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
    pub end_address: VirtAddr,
}

impl UserStack {
    pub fn new(page_table: &mut OffsetPageTable<'static>) -> Self {
        let end_address = VirtAddr::new(USER_STACK_END as u64);

        MemoryManager::alloc_range(
            end_address - USER_STACK_SIZE as u64,
            USER_STACK_SIZE as u64,
            MappingType::UserData.flags(),
            page_table,
        )
        .unwrap();

        Self { end_address }
    }
}
