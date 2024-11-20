#![no_std]
#![no_main]

// use kernel::device::ahci::AHCI;
use kernel::device::hpet::HPET;
// use kernel::device::nvme::NVME;
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
    log::info!("Boot time: {} ns", HPET.elapsed_ns());

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

    Thread::new_kernel_thread(terminal_manual_flush);

    let ansi_red_test_string = "\x1b[31mRed\x1b[0m";
    log::info!("ANSI red test string: {}", ansi_red_test_string);

    (40..=47).for_each(|index| kernel::print!("\x1b[{}m   \x1b[0m", index));
    kernel::println!();
    (100..=107).for_each(|index| kernel::print!("\x1b[{}m   \x1b[0m", index));
    kernel::println!();

    let current_time = RtcDateTime::default().to_datetime().unwrap();
    log::info!("Current time: {}", current_time);

    kernel::println!("Hello, world!");
    kernel::println!("你好，世界！");

    let hello_raw_elf = include_bytes!("../../target/x86_64-unknown-none/release/hello");
    let counter_raw_elf = include_bytes!("../../target/x86_64-unknown-none/release/counter");
    Process::new_user_process("Hello", hello_raw_elf);
    Process::new_user_process("Counter", counter_raw_elf);

    loop {
        x86_64::instructions::hlt();
    }
}
