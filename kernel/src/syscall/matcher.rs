use core::{
    arch::asm,
    mem::{transmute, variant_count},
};

use super::operations::*;

#[derive(Debug)]
#[allow(dead_code)]
enum SyscallIndex {
    Read,
    Write,
    Mmap,
    Yield,
    Sleep,
    Exit,
}

impl From<usize> for SyscallIndex {
    fn from(number: usize) -> Self {
        let syscall_length = variant_count::<Self>();
        if number >= syscall_length {
            panic!("Invalid syscall index: {}", number);
        }
        unsafe { transmute(number as u8) }
    }
}

#[allow(unused_variables)]
pub extern "C" fn syscall_matcher(
    arg1: usize,
    arg2: usize,
    arg3: usize,
    arg4: usize,
    arg5: usize,
    arg6: usize,
) -> isize {
    let syscall_index: usize;
    unsafe { asm!("mov {0}, rax", out(reg) syscall_index) };

    match SyscallIndex::from(syscall_index) {
        SyscallIndex::Read => unimplemented!(),
        SyscallIndex::Write => write(arg1 as *const u8, arg2),
        SyscallIndex::Mmap => mmap(arg1, arg2),
        SyscallIndex::Yield => r#yield(),
        SyscallIndex::Sleep => sleep(arg1 as u64),
        SyscallIndex::Exit => exit(),
    }
}
