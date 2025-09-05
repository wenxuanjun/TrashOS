#![no_std]
#![no_main]

use kernel::drivers::hpet::HPET;
use kernel::drivers::rtc::RtcDateTime;
use kernel::drivers::term::terminal_thread;
use kernel::tasks::process::Process;
use kernel::tasks::thread::Thread;
use limine::BaseRevision;
use unwinding::panic::catch_unwind;

#[used]
#[unsafe(link_section = ".requests")]
static BASE_REVISION: BaseRevision = BaseRevision::new();

#[unsafe(no_mangle)]
extern "C" fn kmain() -> ! {
    catch_unwind(kernel::init).unwrap();
    Thread::new_kernel_thread(terminal_thread);
    log::info!("Boot time: {:?}", HPET.elapsed());

    (40..=47).for_each(|index| kernel::print!("\x1b[{}m   \x1b[0m", index));
    kernel::println!();
    (100..=107).for_each(|index| kernel::print!("\x1b[{}m   \x1b[0m", index));
    kernel::println!();

    let current_time = RtcDateTime::default().to_datetime().unwrap();
    log::info!("Current time: {current_time}");

    let hello_raw_elf = include_bytes!("../../target/x86_64-unknown-none/release/hello");
    let counter_raw_elf = include_bytes!("../../target/x86_64-unknown-none/release/counter");
    Process::create("Hello", hello_raw_elf);
    Process::create("Counter", counter_raw_elf);

    kernel::io::init_manager().unwrap();
    kernel::drivers::xhci::test_xhci();

    loop {
        x86_64::instructions::hlt();
    }
}
