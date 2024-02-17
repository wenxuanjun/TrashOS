#[naked]
pub extern "C" fn syscall0(_rax: u64) -> usize {
    unsafe {
        core::arch::asm!(
            "mov rax, rdi",
            "syscall",
            "ret",
            options(noreturn)
        )
    }
}

#[naked]
pub extern "C" fn syscall1(_rax: u64, _rdi: *const u8) -> usize {
    unsafe {
        core::arch::asm!(
            "mov rax, rdi",
            "mov rdi, rsi",
            "syscall",
            "ret",
            options(noreturn)
        )
    }
}

#[naked]
pub extern "C" fn syscall2(_rax: u64, _rdi: *const u8, _rsi: usize) -> usize {
    unsafe {
        core::arch::asm!(
            "mov rax, rdi",
            "mov rdi, rsi",
            "mov rsi, rdx",
            "syscall",
            "ret",
            options(noreturn)
        )
    }
}
