use alloc::boxed::Box;
use x86_64::VirtAddr;
use x86_64::structures::paging::OffsetPageTable;

use crate::mem::{MappingType, MemoryManager};

const KERNEL_STACK_SIZE: usize = 64 * 1024;
const USER_STACK_END: usize = 0x7fffffff0000;
const USER_STACK_SIZE: usize = 256 * 1024;

pub struct KernelStack(Box<[u8]>);

impl Default for KernelStack {
    fn default() -> Self {
        Self(Box::from(alloc::vec![0; KERNEL_STACK_SIZE]))
    }
}

impl KernelStack {
    pub fn end_address(&self) -> VirtAddr {
        VirtAddr::new(self.0.as_ptr_range().end as u64)
    }
}

pub struct UserStack;

impl UserStack {
    pub fn end_address() -> VirtAddr {
        VirtAddr::new(USER_STACK_END as u64)
    }
}

impl UserStack {
    pub fn map(page_table: &mut OffsetPageTable<'static>) {
        let end_address = VirtAddr::new(USER_STACK_END as u64);

        MemoryManager::alloc_range(
            end_address - USER_STACK_SIZE as u64,
            USER_STACK_SIZE as u64,
            MappingType::UserData.flags(),
            page_table,
        )
        .unwrap();
    }
}
