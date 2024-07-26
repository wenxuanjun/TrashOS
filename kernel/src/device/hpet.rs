use core::ptr::{read_volatile, write_volatile};
use spin::Lazy;
use x86_64::PhysAddr;

use crate::{arch::acpi::ACPI, memory::convert_physical_to_virtual};

pub static HPET: Lazy<Hpet> = Lazy::new(|| {
    let physical_address = PhysAddr::new(ACPI.hpet_info.base_address as u64);
    let virtual_address = convert_physical_to_virtual(physical_address);

    let hpet = Hpet::new(virtual_address.as_u64());
    hpet.enable_counter();
    hpet
});

pub struct Hpet {
    base_address: u64,
    fms_per_tick: u32,
}

impl Hpet {
    #[inline]
    pub fn new(base_address: u64) -> Self {
        let fms_per_tick = unsafe {
            let value = read_volatile(base_address as *const u64);
            (value >> 32) as u32
        };

        Self {
            base_address,
            fms_per_tick,
        }
    }

    #[inline]
    pub fn elapsed_ns(&self) -> u64 {
        let elapsed_fms = self.elapsed_ticks() * self.fms_per_tick as u64;
        elapsed_fms / 1_000_000
    }

    fn enable_counter(&self) {
        unsafe {
            let configuration_addr = self.base_address + 0x10;
            let old = read_volatile(configuration_addr as *const u64);
            write_volatile(configuration_addr as *mut u64, old | 1);
        }
    }

    fn elapsed_ticks(&self) -> u64 {
        unsafe {
            let counter_l_addr = self.base_address + 0xf0;
            let counter_h_addr = self.base_address + 0xf4;
            loop {
                let high1 = read_volatile(counter_h_addr as *const u32);
                let low = read_volatile(counter_l_addr as *const u32);
                let high2 = read_volatile(counter_h_addr as *const u32);
                if high1 == high2 {
                    return (high1 as u64) << 32 | low as u64;
                }
            }
        }
    }
}

unsafe impl Sync for Hpet {}
