#![no_std]
#![no_main]
#![feature(naked_functions)]
#![feature(alloc_error_handler)]
#![feature(stmt_expr_attributes)]

// use kernel::driver::ahci::AHCI;
use kernel::driver::hpet::HPET;
// use kernel::driver::nvme::NVME;
use kernel::driver::rtc::RtcDateTime;
use kernel::driver::terminal::terminal_thread;
use kernel::task::process::Process;
use kernel::task::thread::Thread;
use limine::BaseRevision;

#[used]
#[unsafe(link_section = ".requests")]
static BASE_REVISION: BaseRevision = BaseRevision::new();

#[unsafe(no_mangle)]
extern "C" fn kmain() -> ! {
    kernel::init();
    log::info!("Boot time: {:?}", HPET.elapsed());

    // let mut ahci_manager = AHCI.lock();
    // log::info!("AHCI disk count: {}", ahci_manager.len());

    // if let Some(disk) = ahci_manager.get_disk(0) {
    //     log::info!("AHCI identity: {:#?}", disk.get_identity());

    //     let mut read_buffer = [0u8; 512];
    //     disk.read_block(1, &mut read_buffer);
    //     log::info!("AHCI first sector: {:?}", read_buffer);
    // }

    // let mut nvme_manager = NVME.lock();
    // log::info!("NVMe disk count: {}", nvme_manager.len());

    // if let Some(disk) = nvme_manager.get_disk(0) {
    //     log::info!("NVMe identity: {:#?}", disk.get_identity());

    //     let mut read_buffer = [0u8; 512];
    //     disk.read_block(1, &mut read_buffer);
    //     log::info!("NVMe first sector: {:?}", read_buffer);
    // }

    // let mut write_buffer = [0u8; 512];
    // write_buffer[0] = 11;
    // write_buffer[1] = 45;
    // write_buffer[2] = 14;
    // nvme_manager.write_block(0, 1, &write_buffer);

    // let mut read_buffer = [0u8; 512];
    // nvme_manager.read_block(0, 1, &mut read_buffer);
    // log::info!("NVMe first sector: {:?}", read_buffer);

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
