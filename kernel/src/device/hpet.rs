use core::ptr;
use spin::Lazy;
use x86_64::{PhysAddr, VirtAddr};

use crate::arch::acpi::ACPI;
use crate::memory::convert_physical_to_virtual;

pub static HPET: Lazy<Hpet> = Lazy::new(|| {
    let physical_address = PhysAddr::new(ACPI.hpet_info.base_address as u64);
    Hpet::new(convert_physical_to_virtual(physical_address))
});

pub struct Hpet {
    address: VirtAddr,
    fms_per_tick: u32,
}

impl Hpet {
    pub fn new(address: VirtAddr) -> Self {
        let fms_per_tick = unsafe {
            let value: u64 = ptr::read_volatile(address.as_ptr());
            (value >> 32) as u32
        };

        let hpet = Self {
            address,
            fms_per_tick,
        };

        hpet.enable_counter();

        hpet
    }

    pub fn elapsed_ns(&self) -> u64 {
        let elapsed_fms = self.elapsed_ticks() * self.fms_per_tick as u64;
        elapsed_fms / 1_000_000
    }
}

impl Hpet {
    fn enable_counter(&self) {
        unsafe {
            let configuration_addr = self.address + 0x10;
            let old: u64 = ptr::read_volatile(configuration_addr.as_ptr());
            ptr::write_volatile(configuration_addr.as_mut_ptr(), old | 1);
        }
    }

    fn elapsed_ticks(&self) -> u64 {
        unsafe {
            let counter_l_addr = self.address + 0xf0;
            let counter_h_addr = self.address + 0xf4;
            loop {
                let low: u32 = ptr::read_volatile(counter_l_addr.as_ptr());
                let high1: u32 = ptr::read_volatile(counter_h_addr.as_ptr());
                let high2 = ptr::read_volatile(counter_h_addr.as_ptr());

                if high1 == high2 {
                    return (high1 as u64) << 32 | low as u64;
                }
            }
        }
    }
}

unsafe impl Sync for Hpet {}
