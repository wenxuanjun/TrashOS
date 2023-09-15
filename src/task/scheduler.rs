use alloc::boxed::Box;
use alloc::collections::VecDeque;
use alloc::sync::Arc;
use conquer_once::spin::OnceCell;
use spin::{Lazy, RwLock};
use x86_64::VirtAddr;
use x86_64::instructions::interrupts;

use super::context::Context;
use super::{Process, Thread};

pub static SCHEDULER: Lazy<RwLock<Scheduler>> = Lazy::new(|| RwLock::new(Scheduler::new()));
pub static KERNEL_PROCESS: OnceCell<Arc<RwLock<Box<Process>>>> = OnceCell::uninit();

pub fn init() {
    let kernel_process = Process::new_kernel_process();
    KERNEL_PROCESS.init_once(|| kernel_process.clone());
    /*let idle_thread = || loop { x86_64::instructions::hlt(); };
    Thread::new_kernel_thread(idle_thread);
    x86_64::instructions::interrupts::enable();*/
}

pub struct Scheduler {
    pub current_thread: Option<Arc<RwLock<Box<Thread>>>>,
    processes: VecDeque<Arc<RwLock<Box<Process>>>>,
}

impl Scheduler {
    pub fn new() -> Self {
        Scheduler {
            current_thread: None,
            processes: VecDeque::new(),
        }
    }

    pub fn add(&mut self, process: Arc<RwLock<Box<Process>>>) {
        self.processes.push_back(process);
    }

    pub fn get_next(&mut self) -> Option<Arc<RwLock<Box<Thread>>>> {
        if self.processes.is_empty() {
            return None;
        }

        let process = {
            let filter = |process: &mut Arc<RwLock<Box<Process>>>| {
                let process = process.read();
                !process.threads.is_empty()
            };
            let process_index = self.processes.iter_mut().position(filter).unwrap();
            self.processes.remove(process_index).unwrap()
        };

        let thread = {
            let mut process = process.write();

            if process.threads.is_empty() {
                return None;
            }

            let to_thread = process.threads.pop_front();
            process.threads.push_back(to_thread.clone().unwrap());

            to_thread
        };

        self.processes.push_back(process);

        Some(thread.unwrap())
    }

    pub fn schedule(&mut self, context_address: VirtAddr) -> Option<VirtAddr> {
        if let Some(thread) = self.current_thread.take() {
            let mut thread = thread.write();
            thread.context = Context::from_address(context_address);
        }

        self.current_thread = self.get_next();

        match self.current_thread.as_ref() {
            Some(thread) => {
                let thread = thread.read();

                let page_table = &thread.process.read().page_table;

                interrupts::without_interrupts(|| unsafe {
                    if !page_table.is_current() {
                        page_table.switch();
                    }
                });

                Some(thread.context.address())
            }
            None => None,
        }
    }
}
