use alloc::vec::Vec;
use x86_64::VirtAddr;

const KERNEL_STACK_SIZE: usize = 4 * 1024;
const USER_STACK_INIT_SIZE: usize = 16 * 1024;
const INIT_THREAD_STACK_SIZE: usize = 0;

pub enum StackType {
    Kernel,
    User,
    Empty,
}

#[derive(Debug)]
pub struct ThreadStack {
    _inner: Vec<u8>,
    start_address: VirtAddr,
    end_address: VirtAddr,
}

impl ThreadStack {
    pub fn new(stack_type: StackType) -> Self {
        let size = match stack_type {
            StackType::Kernel => KERNEL_STACK_SIZE,
            StackType::User => USER_STACK_INIT_SIZE,
            StackType::Empty => INIT_THREAD_STACK_SIZE,
        };

        let inner = Vec::with_capacity(size);
        let start_address = VirtAddr::from_ptr(inner.as_ptr());
        let end_address = start_address + size as u64 - 1u64;

        ThreadStack {
            _inner: inner,
            start_address,
            end_address,
        }
    }

    pub fn start_address(&self) -> VirtAddr {
        self.start_address
    }

    pub fn end_address(&self) -> VirtAddr {
        self.end_address
    }
}
