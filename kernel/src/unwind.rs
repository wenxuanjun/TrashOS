use alloc::boxed::Box;
use core::{panic::PanicInfo, ptr::addr_of_mut};
use unwinding::abi::{_Unwind_Backtrace, _Unwind_GetIP};
use unwinding::abi::{UnwindContext, UnwindReasonCode};
use unwinding::panic::begin_panic;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    log::error!("{}", info);
    log::error!("Backtrace:");

    struct Data {
        counter: usize,
    }

    extern "C" fn callback(
        unwind_ctx: &UnwindContext<'_>,
        arg: *mut core::ffi::c_void,
    ) -> UnwindReasonCode {
        let data = unsafe { &mut *(arg as *mut Data) };
        log::error!(
            "{:4}:{:#19x} - <unknown>",
            data.counter,
            _Unwind_GetIP(unwind_ctx)
        );
        UnwindReasonCode::NO_REASON
    }

    let mut data = Data { counter: 0 };
    _Unwind_Backtrace(callback, addr_of_mut!(data) as _);

    if info.can_unwind() {
        struct NoPayload;
        let code = begin_panic(Box::new(NoPayload));
        log::error!("Unwind reason code: {}", code.0);
    }

    loop {
        x86_64::instructions::hlt();
    }
}
