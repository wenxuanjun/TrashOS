#![no_std]
#![no_main]

use kernel::driver::ahci::AHCI;
use kernel::driver::hpet::HPET;
use kernel::driver::rtc::RtcDateTime;
use kernel::driver::term::terminal_thread;
use kernel::task::process::Process;
use kernel::task::thread::Thread;
use limine::BaseRevision;
use unwinding::panic::catch_unwind;

#[used]
#[unsafe(link_section = ".requests")]
static BASE_REVISION: BaseRevision = BaseRevision::new();

#[unsafe(no_mangle)]
extern "C" fn kmain() -> ! {
    catch_unwind(|| kernel::init()).unwrap();
    log::info!("Boot time: {:?}", HPET.elapsed());

    let ahci_manager = AHCI.lock();
    log::info!("AHCI disk count: {}", ahci_manager.len());

    Thread::new_kernel_thread(terminal_thread);

    (40..=47).for_each(|index| kernel::print!("\x1b[{}m   \x1b[0m", index));
    kernel::println!();
    (100..=107).for_each(|index| kernel::print!("\x1b[{}m   \x1b[0m", index));
    kernel::println!();

    let current_time = RtcDateTime::default().to_datetime().unwrap();
    log::info!("Current time: {}", current_time);

    let hello_raw_elf = include_bytes!("../../target/x86_64-unknown-none/release/hello");
    let counter_raw_elf = include_bytes!("../../target/x86_64-unknown-none/release/counter");
    Process::create("Hello", hello_raw_elf);
    Process::create("Number", counter_raw_elf);

    loop {
        x86_64::instructions::hlt();
    }
}
