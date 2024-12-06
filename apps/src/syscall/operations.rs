use super::naked::*;

#[allow(dead_code)]
const READ_SYSCALL_NUMBER: u64 = 0;
const WRITE_SYSCALL_NUMBER: u64 = 1;
const MMAP_SYSCALL_NUMBER: u64 = 2;
const YIELD_SYSCALL_NUMBER: u64 = 3;
const SLEEP_SYSCALL_NUMBER: u64 = 4;
const EXIT_SYSCALL_NUMBER: u64 = 5;

pub fn write(buffer: *const u8, length: usize) -> usize {
    syscall2(WRITE_SYSCALL_NUMBER, buffer as usize, length)
}

pub fn mmap(address: usize, length: usize) -> usize {
    syscall2(MMAP_SYSCALL_NUMBER, address, length)
}

pub fn r#yield() {
    syscall0(YIELD_SYSCALL_NUMBER);
}

pub fn sleep(duration: u64) -> usize {
    syscall1(SLEEP_SYSCALL_NUMBER, duration as usize)
}

pub fn exit() -> ! {
    syscall0_noret(EXIT_SYSCALL_NUMBER);
}
