use core::{cell::UnsafeCell, ptr};
use x86_64::PhysAddr;

use crate::memory::convert_physical_to_virtual;

pub static HPET: Hpet = Hpet::uninit();

pub fn init() {
    let acpi = &crate::arch::acpi::ACPI;
    let physical_address = PhysAddr::new(acpi.hpet_info.base_address as u64);
    let virtual_address = convert_physical_to_virtual(physical_address);

    HPET.init(virtual_address.as_u64());
    HPET.enable_counter();

    log::debug!("HPET clock speed: {} femto seconds", HPET.clock_speed());
    log::debug!("HPET timers: {} available", HPET.timers_count());
}

pub struct Hpet {
    base_addr: UnsafeCell<u64>,
}

impl Hpet {
    #[inline]
    pub const fn uninit() -> Self {
        Hpet {
            base_addr: UnsafeCell::new(0),
        }
    }

    pub fn init(&self, base_addr: u64) {
        unsafe {
            self.base_addr.get().write(base_addr);
        }
    }

    pub fn clock_speed(&self) -> u32 {
        unsafe {
            let base_addr = *self.base_addr.get();
            let value = ptr::read_volatile(base_addr as *const u64);
            (value >> 32) as u32
        }
    }

    pub fn timers_count(&self) -> u32 {
        unsafe {
            let base_addr = *self.base_addr.get();
            let value = ptr::read_volatile(base_addr as *const u64);
            (((value >> 8) & 0b11111) + 1) as u32
        }
    }

    pub fn enable_counter(&self) {
        unsafe {
            let configuration_addr = *self.base_addr.get() + 0x10;
            let old = ptr::read_volatile(configuration_addr as *const u64);
            ptr::write_volatile(configuration_addr as *mut u64, old | 1);
        }
    }

    pub fn get_counter(&self) -> u64 {
        unsafe {
            let counter_l_addr = *self.base_addr.get() + 0xf0;
            let counter_h_addr = *self.base_addr.get() + 0xf4;
            loop {
                let high1 = ptr::read_volatile(counter_h_addr as *const u32);
                let low = ptr::read_volatile(counter_l_addr as *const u32);
                let high2 = ptr::read_volatile(counter_h_addr as *const u32);
                if high1 == high2 {
                    return (high1 as u64) << 32 | low as u64;
                }
            }
        }
    }

    #[inline]
    pub fn get_time_elapsed(&self) -> u64 {
        self.get_counter() * (self.clock_speed() as u64 / 1_000_000)
    }
}

unsafe impl Sync for Hpet {}
