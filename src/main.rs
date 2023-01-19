#![no_std]
#![no_main]
#![allow(non_snake_case)]
#![feature(abi_x86_interrupt)]

extern crate alloc;
use x86_64::VirtAddr;
use core::panic::PanicInfo;
use alloc::{boxed::Box, vec::Vec};
use TrashOS::printk::PrintLevel;
use TrashOS::{println, allocator, task::keyboard};
use TrashOS::memory::{self, BootInfoFrameAllocator};
use TrashOS::task::{Task, executor::Executor};
use bootloader_api::{BootInfo, entry_point};
use bootloader_api::config::{BootloaderConfig, Mapping};

pub static BOOTLOADER_CONFIG: BootloaderConfig = {
    let mut config = BootloaderConfig::new_default();
    config.mappings.physical_memory = Some(Mapping::Dynamic);
    config
};

entry_point!(main, config = &BOOTLOADER_CONFIG);

fn main(boot_info: &'static mut BootInfo) -> ! {
    TrashOS::init(unsafe { &mut *(boot_info as *mut BootInfo) });

    TrashOS::log!(PrintLevel::Error, "This is an error message!");
    TrashOS::log!(PrintLevel::Warn, "This is a warning message!");
    TrashOS::log!(PrintLevel::Info, "This is an info message!");
    TrashOS::log!(PrintLevel::Debug, "This is a debug message!");

    let offset = boot_info.physical_memory_offset.clone();
    let phys_mem_offset = VirtAddr::new(offset.into_option().unwrap());
    let mut mapper = unsafe { memory::init(phys_mem_offset) };
    let mut frame_allocator = unsafe { BootInfoFrameAllocator::init(&boot_info.memory_regions) };

    allocator::init_heap(&mut mapper, &mut frame_allocator)
        .expect("Heap initialization failed!");

    let heap_value = Box::new(41);
    println!("The heap start at {:p}", heap_value);

    let mut vec = Vec::new();
    for i in 0..500 { vec.push(i); }
    println!("Now test the vec is at {:p}", vec.as_slice());

    println!("min0911_ TQL%%%!");

    let mut executor = Executor::new();
    executor.spawn(Task::new(keyboard::print_keypresses()));
    executor.run();
}

#[panic_handler]
fn panic(_panic_info: &PanicInfo<'_>) -> ! {
    println!("{}", _panic_info);
    loop { x86_64::instructions::hlt(); }
}