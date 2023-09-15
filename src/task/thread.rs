use alloc::boxed::Box;
use alloc::sync::Arc;
use core::fmt::Debug;
use core::sync::atomic::{AtomicU64, Ordering};
use spin::RwLock;
use x86_64::structures::paging::PageTableFlags;
use x86_64::VirtAddr;

use super::context::Context;
use super::process::Process;
use super::scheduler;
use super::stack::{StackType, ThreadStack};
use crate::gdt::Selectors;
use crate::memory::MemoryManager;

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

pub struct Thread {
    pub id: ThreadId,
    state: ThreadState,
    pub user_stack: ThreadStack,
    pub context: Context,
    pub process: Arc<RwLock<Box<Process>>>,
}

impl Thread {
    pub fn new_kernel_thread(function: fn()) {
        let process = scheduler::KERNEL_PROCESS.try_get().unwrap();

        let mut thread = {
            let thread = Thread {
                id: ThreadId::new(),
                state: ThreadState::Ready,
                user_stack: ThreadStack::new(StackType::User),
                context: Context::default(),
                process: process.clone(),
            };

            Box::new(thread)
        };

        thread.context.init(
            function as usize,
            thread.user_stack.end_address(),
            Selectors::get_kernel_segments(),
        );

        let thread = Arc::new(RwLock::new(thread));
        process.write().threads.push_back(thread);
    }

    pub fn new_user_thread(process: Arc<RwLock<Box<Process>>>, entry_point: usize) {
        let mut thread = Box::new(Thread {
            id: ThreadId::new(),
            state: ThreadState::Ready,
            user_stack: ThreadStack::new(StackType::User),
            context: Context::default(),
            process: process.clone(),
        });

        const USER_STACK_SIZE: usize = 64 * 1024;
        let user_stack_end = VirtAddr::new(0x00007fffffffffff);
        let user_stack_start = user_stack_end - USER_STACK_SIZE as u64 + 1u64;

        let flags =
            PageTableFlags::PRESENT | PageTableFlags::WRITABLE | PageTableFlags::USER_ACCESSIBLE;

        let mut process = process.write();

        <MemoryManager>::alloc_range(
            user_stack_start,
            user_stack_end,
            flags,
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
