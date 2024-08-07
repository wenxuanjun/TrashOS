use alloc::boxed::Box;
use alloc::sync::{Arc, Weak};
use core::fmt::Debug;
use core::sync::atomic::{AtomicU64, Ordering};
use spin::RwLock;
use x86_64::instructions::interrupts;

use super::context::Context;
use super::process::{WeakSharedProcess, KERNEL_PROCESS};
use super::scheduler::SCHEDULER;
use super::stack::{KernelStack, UserStack};
use crate::arch::gdt::Selectors;
use crate::memory::{ExtendedPageTable, KERNEL_PAGE_TABLE};

pub(super) type SharedThread = Arc<RwLock<Box<Thread>>>;
pub(super) type WeakSharedThread = Weak<RwLock<Box<Thread>>>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct ThreadId(pub u64);

impl ThreadId {
    fn new() -> Self {
        static NEXT_ID: AtomicU64 = AtomicU64::new(0);
        ThreadId(NEXT_ID.fetch_add(1, Ordering::Relaxed))
    }
}

pub struct Thread {
    pub id: ThreadId,
    pub kernel_stack: KernelStack,
    pub context: Context,
    pub process: WeakSharedProcess,
}

impl Thread {
    pub fn new(process: WeakSharedProcess) -> Self {
        let thread = Thread {
            id: ThreadId::new(),
            context: Context::default(),
            kernel_stack: KernelStack::new(),
            process,
        };

        thread
    }

    pub fn get_init_thread() -> WeakSharedThread {
        let thread = Self::new(Arc::downgrade(&KERNEL_PROCESS));
        let thread = Arc::new(RwLock::new(Box::new(thread)));
        KERNEL_PROCESS.write().threads.push(thread.clone());
        Arc::downgrade(&thread)
    }

    pub fn new_kernel_thread(function: fn()) {
        let mut thread = Self::new(Arc::downgrade(&KERNEL_PROCESS));

        thread.context.init(
            function as usize,
            thread.kernel_stack.end_address(),
            KERNEL_PAGE_TABLE.lock().physical_address(),
            Selectors::get_kernel_segments(),
        );

        let thread = Arc::new(RwLock::new(Box::new(thread)));
        KERNEL_PROCESS.write().threads.push(thread.clone());

        interrupts::without_interrupts(|| {
            SCHEDULER.lock().add(Arc::downgrade(&thread));
        });
    }

    pub fn new_user_thread(process: WeakSharedProcess, entry_point: usize) {
        let mut thread = Self::new(process.clone());
        let process = process.upgrade().unwrap();
        let mut process = process.write();
        let user_stack = UserStack::new(&mut process.page_table);

        thread.context.init(
            entry_point,
            user_stack.end_address,
            process.page_table.physical_address(),
            Selectors::get_user_segments(),
        );

        let thread = Arc::new(RwLock::new(Box::new(thread)));
        process.threads.push(thread.clone());

        SCHEDULER.lock().add(Arc::downgrade(&thread));
    }
}
