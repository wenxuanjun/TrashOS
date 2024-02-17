use crate::syscall::naked::syscall2;

pub fn write(buffer: *const u8, length: usize) -> usize {
    const WRITE_SYSCALL_NUMBER: u64 = 1;
    syscall2(WRITE_SYSCALL_NUMBER, buffer, length)
}
