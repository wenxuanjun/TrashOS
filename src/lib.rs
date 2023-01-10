#![no_std]
#![allow(non_snake_case)]
#![feature(abi_x86_interrupt)]

pub mod serial;
pub mod vga_buffer;
pub mod gdt;
pub mod interrupts;

pub fn init() {
    gdt::init();
    interrupts::init_idt();
    unsafe { interrupts::PICS.lock().initialize() };
    x86_64::instructions::interrupts::enable();
}