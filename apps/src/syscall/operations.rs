use super::naked::{syscall0_noret, syscall2};

pub fn write(buffer: *const u8, length: usize) -> usize {
    const WRITE_SYSCALL_NUMBER: u64 = 1;
    syscall2(WRITE_SYSCALL_NUMBER, buffer as usize, length)
}

pub fn malloc(address: usize, length: usize) -> usize {
    const MALLOC_SYSCALL_NUMBER: u64 = 2;
    syscall2(MALLOC_SYSCALL_NUMBER, address, length)
}

pub fn exit() -> ! {
    const EXIT_SYSCALL_NUMBER: u64 = 3;
    syscall0_noret(EXIT_SYSCALL_NUMBER);
}
