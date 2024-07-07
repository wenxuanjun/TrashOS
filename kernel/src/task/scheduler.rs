use core::sync::atomic::{AtomicBool, Ordering};

use alloc::collections::VecDeque;
use spin::{Lazy, RwLock};
use x86_64::instructions::interrupts;
use x86_64::VirtAddr;

use super::context::Context;
use super::process::SharedProcess;
use super::thread::SharedThread;
use super::{Process, Thread};
use crate::arch::smp::CPUS;

pub static SCHEDULER_INIT: AtomicBool = AtomicBool::new(false);
pub static SCHEDULER: Lazy<RwLock<Scheduler>> = Lazy::new(|| RwLock::new(Scheduler::new()));
pub static KERNEL_PROCESS: Lazy<SharedProcess> = Lazy::new(|| Process::new_kernel_process());

pub fn init() {
    SCHEDULER.write().add(KERNEL_PROCESS.clone());
    x86_64::instructions::interrupts::enable();
    SCHEDULER_INIT.store(true, Ordering::Relaxed);
    log::info!("Scheduler initialized, interrupts enabled!");
}

pub struct Scheduler {
    pub current_thread: SharedThread,
    processes: VecDeque<SharedProcess>,
}

impl Scheduler {
    pub fn new() -> Self {
        Self {
            current_thread: Thread::new_init_thread(),
            processes: VecDeque::new(),
        }
    }

    #[inline]
    pub fn add(&mut self, process: SharedProcess) {
        interrupts::without_interrupts(|| {
            self.processes.push_back(process);
        });
    }

    pub fn get_next(&mut self) -> SharedThread {
        let process = {
            let process_index = self
                .processes
                .iter_mut()
                .position(|process| !process.read().threads.is_empty())
                .unwrap();
            self.processes.remove(process_index).unwrap()
        };

        let thread = {
            let mut process = process.write();
            let to_thread = process.threads.pop_front();
            process.threads.push_back(to_thread.clone().unwrap());
            to_thread
        };

        self.processes.push_back(process);

        thread.unwrap()
    }

    pub fn schedule(&mut self, context: VirtAddr) -> VirtAddr {
        {
            let mut thread = self.current_thread.write();
            thread.context = Context::from_address(context);
        }

        let _lock = crate::GLOBAL_MUTEX.lock();
        self.current_thread = self.get_next();
        let next_thread = self.current_thread.read();

        let kernel_address = next_thread.kernel_stack.end_address();
        CPUS.lock().current_cpu().1.set_ring0_rsp(kernel_address);

        next_thread.context.address()
    }
}
