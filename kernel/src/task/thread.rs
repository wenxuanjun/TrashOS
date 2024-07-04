use alloc::boxed::Box;
use alloc::sync::Arc;
use core::fmt::Debug;
use core::sync::atomic::{AtomicU64, Ordering};
use spin::RwLock;

use super::context::Context;
use super::process::SharedProcess;
use super::scheduler::KERNEL_PROCESS;
use super::stack::{KernelStack, UserStack};
use crate::arch::gdt::Selectors;

pub(super) type SharedThread = Arc<RwLock<Box<Thread>>>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct ThreadId(pub u64);

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
    pub id: ThreadId,
    pub state: ThreadState,
    pub kernel_stack: KernelStack,
    pub context: Context,
    pub process: SharedProcess,
}

impl Thread {
    pub fn new(process: SharedProcess) -> Box<Self> {
        let thread = Thread {
            id: ThreadId::new(),
            state: ThreadState::Ready,
            context: Context::default(),
            kernel_stack: KernelStack::new(),
            process,
        };

        Box::new(thread)
    }

    pub fn new_init_thread() -> SharedThread {
        let thread = Self::new(KERNEL_PROCESS.clone());
        let thread = Arc::new(RwLock::new(thread));
        KERNEL_PROCESS.write().threads.push_back(thread.clone());

        thread
    }

    pub fn new_kernel_thread(function: fn()) {
        let mut thread = Self::new(KERNEL_PROCESS.clone());

        thread.context.init(
            function as usize,
            thread.kernel_stack.end_address(),
            Selectors::get_kernel_segments(),
        );

        let thread = Arc::new(RwLock::new(thread));
        KERNEL_PROCESS.write().threads.push_back(thread);
    }

    pub fn new_user_thread(process: SharedProcess, entry_point: usize) {
        let mut thread = Self::new(process.clone());
        let mut process = process.write();

        let user_stack = UserStack::new(&mut process.page_table);

        thread.context.init(
            entry_point,
            user_stack.end_address,
            Selectors::get_user_segments(),
        );

        let thread = Arc::new(RwLock::new(thread));
        process.threads.push_back(thread.clone());
    }
}
