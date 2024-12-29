macro_rules! syscall {
    (@mov 0) => {"rdi, {0}"};
    (@mov 1) => {"rsi, {1}"};
    (@mov 2) => {"rdx, {2}"};
    (@mov 3) => {"r10, {3}"};
    (@mov 4) => {"r8, {4}"};
    (@mov 5) => {"r9, {5}"};
    (@mov $index:expr) => {compile_error!("Allows up to 6 arguments")};

    (@noret $index:expr $(,$arg:expr)*) => {
        unsafe {
            core::arch::asm!(
                $(
                    ${ignore($arg)}
                    concat!("mov ", syscall!(@mov ${index()})),
                )*
                "syscall",
                $(in(reg) $arg,)*
                in("rax") $index,
                clobber_abi("C"),
                options(noreturn),
            );
        }
    };

    ($index:expr $(,$arg:expr)*) => {
        unsafe {
            let ret;
            core::arch::asm!(
                $(
                    ${ignore($arg)}
                    concat!("mov ", syscall!(@mov ${index()})),
                )*
                "syscall",
                $(in(reg) $arg,)*
                in("rax") $index,
                lateout("rax") ret,
                clobber_abi("C"),
            );
            ret
        }
    };
}
