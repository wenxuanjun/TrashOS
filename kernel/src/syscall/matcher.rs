use core::arch::asm;
use core::mem::{transmute, variant_count};

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

impl TryFrom<usize> for SyscallIndex {
    type Error = ();

    fn try_from(number: usize) -> Result<Self, Self::Error> {
        (number < variant_count::<Self>())
            .then(|| unsafe { transmute(number as u8) })
            .ok_or(())
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

    syscall_index.try_into().map_or(-1, |index| match index {
        SyscallIndex::Read => unimplemented!(),
        SyscallIndex::Write => write(arg1 as *const u8, arg2),
        SyscallIndex::Mmap => mmap(arg1, arg2),
        SyscallIndex::Yield => r#yield(),
        SyscallIndex::Sleep => sleep(arg1 as u64),
        SyscallIndex::Exit => exit(),
    })
}
