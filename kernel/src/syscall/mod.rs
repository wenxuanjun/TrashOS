use core::arch::asm;
use x86_64::registers::model_specific::{Efer, EferFlags};
use x86_64::registers::model_specific::{LStar, SFMask, Star};
use x86_64::registers::rflags::RFlags;
use x86_64::VirtAddr;

use crate::arch::gdt::Selectors;
use matcher::syscall_matcher;

mod matcher;
mod operations;

pub fn init() {
    let handler_addr = syscall_handler as *const () as u64;

    SFMask::write(RFlags::INTERRUPT_FLAG);
    LStar::write(VirtAddr::new(handler_addr as u64));

    let (code_selector, data_selector) = Selectors::get_kernel_segments();
    let (user_code_selector, user_data_selector) = Selectors::get_user_segments();

    Star::write(
        user_code_selector,
        user_data_selector,
        code_selector,
        data_selector,
    )
    .unwrap();

    unsafe {
        Efer::write(Efer::read() | EferFlags::SYSTEM_CALL_EXTENSIONS);
    }
}

#[naked]
extern "C" fn syscall_handler() {
    unsafe {
        asm!(
            "push rcx",
            "push r11",
            "push rbp",
            "push rbx",
            "push r12",
            "push r13",
            "push r14",
            "push r15",

            // Move the 4th argument in r10 to rcx to fit the C ABI
            "mov rcx, r10",
            "call {syscall_matcher}",

            "pop r15",
            "pop r14",
            "pop r13",
            "pop r12",
            "pop rbx",
            "pop rbp",
            "pop r11",
            "pop rcx",
            "sysretq",
            syscall_matcher = sym syscall_matcher,
            options(noreturn)
        );
    }
}
