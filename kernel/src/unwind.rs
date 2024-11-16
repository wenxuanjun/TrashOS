use core::panic::PanicInfo;

#[panic_handler]
unsafe fn panic(panic_info: &PanicInfo<'_>) -> ! {
    log::error!("{}", panic_info);

    loop {
        x86_64::instructions::hlt();
    }
}
