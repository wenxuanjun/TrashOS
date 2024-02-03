use alloc::boxed::Box;
use alloc::collections::VecDeque;
use alloc::sync::Arc;
use conquer_once::spin::OnceCell;
use spin::RwLock;
use x86_64::instructions::interrupts;
use x86_64::VirtAddr;

use super::context::Context;
use super::{Process, Thread};

const KERNEL_PROCESS_NAME: &str = "kernel";

pub static SCHEDULER: OnceCell<RwLock<Scheduler>> = OnceCell::uninit();
pub static KERNEL_PROCESS: OnceCell<Arc<RwLock<Box<Process>>>> = OnceCell::uninit();

pub fn init() {
    let kernel_process = Arc::new(RwLock::new(Process::new(KERNEL_PROCESS_NAME)));
    KERNEL_PROCESS.init_once(|| kernel_process.clone());
    SCHEDULER.init_once(|| RwLock::new(Scheduler::new()));
    SCHEDULER.try_get().unwrap().write().add(kernel_process);
    x86_64::instructions::interrupts::enable();
}

pub struct Scheduler {
    pub current_thread: Arc<RwLock<Box<Thread>>>,
    processes: VecDeque<Arc<RwLock<Box<Process>>>>,
}

impl Scheduler {
    pub fn new() -> Self {
        Scheduler {
            current_thread: Thread::new_init_thread(),
            processes: VecDeque::new(),
        }
    }

    #[inline]
    pub fn add(&mut self, process: Arc<RwLock<Box<Process>>>) {
        self.processes.push_back(process);
    }

    pub fn get_next(&mut self) -> Arc<RwLock<Box<Thread>>> {
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
            thread.context = Context::copy_from_address(context);
        }

        self.current_thread = self.get_next();
        let thread = self.current_thread.read();
        let page_table = &thread.process.read().page_table;

        interrupts::without_interrupts(|| unsafe {
            page_table.switch();
        });

        thread.context.address()
    }
}
