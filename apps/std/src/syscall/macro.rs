macro_rules! syscall {
    (@noret $index:expr $(,$arg:expr)*) => {
        syscall!(@impl $index, (), (in), (noreturn), $($arg),*)
    };

    ($index:expr $(,$arg:expr)*) => {{
        let ret;
        syscall!(@impl $index, (ret), (inlateout), (), $($arg),*);
        ret
    }};

    (@impl $index:expr, $ret:tt, $rax_mod:tt, $options:tt, $($arg:expr),*) => {
        _syscall_impl! {
            [$index, $ret, $rax_mod, $options]
            [] ["rdi" "rsi" "rdx" "r10" "r8" "r9"] ($($arg),*)
        }
    };
}

#[doc(hidden)]
macro_rules! _syscall_impl {
    ([$index:expr, ($($ret:tt)?), ($rax_mod:tt), ($($options:tt)?)]
        [$($asm_args:tt)*] [$($_regs:tt)*] ()
    ) => {
        unsafe {
            core::arch::asm!(
                "syscall",
                $($asm_args)*
                $rax_mod("rax") $index $(=> $ret)?,
                clobber_abi("system"),
                options(nostack $(, $options)?)
            )
        }
    };

    ([$index:expr, $ret:tt, $rax_mod:tt, $options:tt]
        [$($asm_args:tt)*] [$reg:tt $($rest_regs:tt)*]
        ($arg:expr $(, $rest_args:expr)*)
    ) => {
        _syscall_impl! {
            [$index, $ret, $rax_mod, $options]
            [$($asm_args)* in($reg) $arg,] [$($rest_regs)*] ($($rest_args),*)
        }
    };

    ([$_index:expr, $_ret:tt, $_rax_mod:tt, $_noret:tt]
        [$($_asm_args:tt)*] [] ($arg:expr $(, $rest_args:expr)*)
    ) => {
        compile_error!(concat!("Syscall allows up to 6 arguments: ", stringify!($arg $(, $rest_args)*)));
    };
}
