#![no_std]
#![no_main]

use core::panic::PanicInfo;
use kernel::device::keyboard::print_keypresses;
use kernel::device::rtc::RtcDateTime;
use kernel::task::{Process, Thread};
use limine::BaseRevision;

#[used]
#[link_section = ".requests"]
static BASE_REVISION: BaseRevision = BaseRevision::new();

#[no_mangle]
extern "C" fn _start() -> ! {
    kernel::init();
    Thread::new_kernel_thread(print_keypresses);
    let current_time = RtcDateTime::new().to_datetime().unwrap();
    log::info!("Current time: {}", current_time);

    let hello_raw_elf = include_bytes!("../../target/x86_64-unknown-none/release/hello");
    let counter_raw_elf = include_bytes!("../../target/x86_64-unknown-none/release/counter");
    Process::new_user_process("Hello", hello_raw_elf).unwrap();
    Process::new_user_process("Counter", counter_raw_elf).unwrap();

    loop {
        x86_64::instructions::hlt();
    }
}

#[panic_handler]
fn panic(panic_info: &PanicInfo<'_>) -> ! {
    log::error!("{}", panic_info);
    loop {
        x86_64::instructions::hlt();
    }
}
