macro_rules! syscall {
    (@mov 0) => {"rdi, {1}"};
    (@mov 1) => {"rsi, {2}"};
    (@mov 2) => {"rdx, {3}"};
    (@mov 3) => {"r10, {4}"};
    (@mov 4) => {"r8, {5}"};
    (@mov 5) => {"r9, {6}"};
    (@mov $index:expr) => {compile_error!("Allows up to 6 arguments")};

    (@noret $index:expr $(,$arg:expr)*) => {
        unsafe {
            core::arch::asm!(
                "mov rax, {0:r}",
                $(
                    ${ignore($arg)}
                    concat!("mov ", syscall!(@mov ${index()})),
                )*
                "syscall",
                in(reg) $index,
                $(in(reg) $arg,)*
                options(noreturn),
            );
        }
    };

    ($index:expr $(,$arg:expr)*) => {
        unsafe {
            let ret;
            core::arch::asm!(
                "mov rax, {0:r}",
                $(
                    ${ignore($arg)}
                    concat!("mov ", syscall!(@mov ${index()})),
                )*
                "syscall",
                in(reg) $index,
                $(in(reg) $arg,)*
                out("rcx") _,
                out("r11") _,
                lateout("rax") ret,
            );
            ret
        }
    };
}
