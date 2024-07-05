use super::naked::{syscall0, syscall2};

pub fn write(buffer: *const u8, length: usize) -> usize {
    const WRITE_SYSCALL_NUMBER: u64 = 1;
    syscall2(WRITE_SYSCALL_NUMBER, buffer as usize, length)
}

pub fn mmap(address: usize, length: usize) -> usize {
    const MMAP_SYSCALL_NUMBER: u64 = 2;
    syscall2(MMAP_SYSCALL_NUMBER, address, length)
}

pub fn halt() {
    const HALT_SYSCALL_NUMBER: u64 = 3;
    syscall0(HALT_SYSCALL_NUMBER);
}