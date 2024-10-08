#![no_std]
#![no_main]

use core::panic::PanicInfo;
use kernel::device::ahci::AHCI;
use kernel::device::hpet::HPET;
use kernel::device::keyboard::print_keypresses;
use kernel::device::rtc::RtcDateTime;
use kernel::device::terminal::terminal_manual_flush;
use kernel::task::process::Process;
use kernel::task::thread::Thread;
use limine::BaseRevision;

#[used]
#[link_section = ".requests"]
static BASE_REVISION: BaseRevision = BaseRevision::new();

#[no_mangle]
extern "C" fn kmain() -> ! {
    kernel::init();
    log::info!("HPET elapsed: {} ns", HPET.elapsed_ns());

    let mut ahci_manager = AHCI.lock();
    log::info!("AHCI disk count: {}", ahci_manager.len());

    if let Some(disk) = ahci_manager.get_disk(0) {
        log::info!("AHCI identity: {:#?}", disk.get_identity());

        let mut read_buffer = [0u8; 512];
        disk.read_block(1, &mut read_buffer);
        log::info!("AHCI first sector: {:?}", read_buffer);
    }

    Thread::new_kernel_thread(print_keypresses);
    Thread::new_kernel_thread(terminal_manual_flush);

    let ansi_red_test_string = "\x1b[31mRed\x1b[0m";
    log::info!("ANSI red test string: {}", ansi_red_test_string);

    (40..=47).for_each(|index| kernel::print!("\x1b[{}m   \x1b[0m", index));
    kernel::println!();
    (100..=107).for_each(|index| kernel::print!("\x1b[{}m   \x1b[0m", index));
    kernel::println!();

    let current_time = RtcDateTime::new().to_datetime().unwrap();
    log::info!("Current time: {}", current_time);

    let hello_raw_elf = include_bytes!("../../target/x86_64-unknown-none/release/hello");
    let counter_raw_elf = include_bytes!("../../target/x86_64-unknown-none/release/counter");
    Process::new_user_process("Hello", hello_raw_elf);
    Process::new_user_process("Counter", counter_raw_elf);

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
