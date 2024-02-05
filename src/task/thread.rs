use alloc::boxed::Box;
use alloc::sync::Arc;
use core::fmt::Debug;
use core::sync::atomic::{AtomicU64, Ordering};
use spin::RwLock;
use x86_64::structures::paging::PageTableFlags;
use x86_64::VirtAddr;

use super::context::Context;
use super::process::SharedProcess;
use super::scheduler;
use super::stack::{StackType, ThreadStack};
use crate::gdt::Selectors;
use crate::memory::MemoryManager;

pub(super) type SharedThread = Arc<RwLock<Box<Thread>>>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct ThreadId(u64);

impl ThreadId {
    fn new() -> Self {
        static NEXT_ID: AtomicU64 = AtomicU64::new(0);
        ThreadId(NEXT_ID.fetch_add(1, Ordering::Relaxed))
    }
}

#[derive(Debug, Copy, Clone)]
pub enum ThreadState {
    Running,
    Ready,
    Blocked,
    Waiting,
    Terminated,
}

#[allow(dead_code)]
pub struct Thread {
    id: ThreadId,
    state: ThreadState,
    pub user_stack: ThreadStack,
    pub context: Context,
    pub process: SharedProcess,
}

impl Thread {
    fn new(process: SharedProcess, stack_type: StackType) -> Box<Self> {
        let thread = Thread {
            id: ThreadId::new(),
            state: ThreadState::Ready,
            user_stack: ThreadStack::new(stack_type),
            context: Context::default(),
            process,
        };

        Box::new(thread)
    }

    pub fn new_init_thread() -> SharedThread {
        let process = scheduler::KERNEL_PROCESS.try_get().unwrap();
        let thread = Self::new(process.clone(), StackType::Empty);
        let thread = Arc::new(RwLock::new(thread));
        process.write().threads.push_back(thread.clone());

        thread
    }

    pub fn new_kernel_thread(function: fn()) {
        let process = scheduler::KERNEL_PROCESS.try_get().unwrap();
        let mut thread = Self::new(process.clone(), StackType::Kernel);
        thread.context.init(
            function as usize,
            thread.user_stack.end_address(),
            Selectors::get_kernel_segments(),
        );
        let thread = Arc::new(RwLock::new(thread));
        process.write().threads.push_back(thread);
    }

    pub fn new_user_thread(process: SharedProcess, entry_point: usize) {
        const USER_STACK_SIZE: usize = 64 * 1024;
        let user_stack_end = VirtAddr::new(0x00007fffffffffff);
        let user_stack_start = user_stack_end - USER_STACK_SIZE as u64 + 1u64;

        let mut thread = Self::new(process.clone(), StackType::User);
        let mut process = process.write();

        <MemoryManager>::alloc_range(
            user_stack_start,
            user_stack_end,
            PageTableFlags::PRESENT | PageTableFlags::WRITABLE | PageTableFlags::USER_ACCESSIBLE,
            &mut process.page_table,
        )
        .unwrap();

        thread
            .context
            .init(entry_point, user_stack_end, Selectors::get_user_segments());

        let thread = Arc::new(RwLock::new(thread));
        process.threads.push_back(thread.clone());
    }
}
