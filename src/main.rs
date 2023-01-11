#![no_std]
#![no_main]
#![allow(non_snake_case)]
#![feature(abi_x86_interrupt)]

extern crate alloc;
use x86_64::VirtAddr;
use core::panic::PanicInfo;
use alloc::{boxed::Box, vec::Vec};
use bootloader::{BootInfo, entry_point};
use TrashOS::{println, allocator, task::keyboard};
use TrashOS::memory::{self, BootInfoFrameAllocator};
use TrashOS::task::{Task, executor::Executor};

entry_point!(main);

fn main(boot_info: &'static BootInfo) -> ! {
    TrashOS::init();

    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let mut mapper = unsafe { memory::init(phys_mem_offset) };
    let mut frame_allocator = unsafe {
        BootInfoFrameAllocator::init(&boot_info.memory_map)
    };

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