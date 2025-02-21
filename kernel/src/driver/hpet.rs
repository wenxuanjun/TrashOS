use bit_field::BitField;
use core::fmt::Display;
use core::time::Duration;
use core::{fmt, ptr};
use spin::Lazy;
use x86_64::PhysAddr;

use crate::arch::acpi::ACPI;
use crate::arch::apic::IrqVector;
use crate::mem::convert_physical_to_virtual;

pub static HPET: Lazy<Hpet> = Lazy::new(|| {
    let physical_address = PhysAddr::new(ACPI.hpet_info.base_address as u64);
    let virtual_address = convert_physical_to_virtual(physical_address);

    let hpet = Hpet::new(virtual_address.as_u64());
    log::info!("HPET: {}", hpet);
    hpet.enable_all().enable_timer()
});

pub struct Hpet {
    base_address: u64,
    fms_per_tick: u64,
    num_timers: u8,
}

impl Display for Hpet {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "addr={:#x}, timers={}, tick={}fms",
            self.base_address,
            self.num_timers,
            self.fms_per_tick,
        )
    }
}

impl Hpet {
    pub fn new(base_address: u64) -> Self {
        let general_ptr = base_address as *const u64;
        let general_info = unsafe { ptr::read_volatile(general_ptr) };

        let num_timers = general_info.get_bits(8..13) as u8 - 1;
        let fms_per_tick = general_info.get_bits(32..64);

        let counter_addr = (base_address + 0xf0) as *const u64;
        unsafe { ptr::write_volatile(counter_addr as *mut u64, 0) };

        Self {
            base_address,
            fms_per_tick,
            num_timers,
        }
    }
}

impl Hpet {
    pub fn ticks(&self) -> u64 {
        let counter_addr = (self.base_address + 0xf0) as *const u64;
        unsafe { ptr::read_volatile(counter_addr) }
    }

    pub fn elapsed(&self) -> Duration {
        let ticks = self.ticks();
        Duration::from_nanos(ticks * self.fms_per_tick / 1_000_000)
    }

    pub fn estimate(&self, duration: Duration) -> u64 {
        let ticks = self.ticks();
        ticks + (duration.as_nanos() as u64 * 1_000_000 / self.fms_per_tick)
    }

    pub fn set_timer(&self, value: u64) {
        let comparator_addr = (self.base_address + 0x108) as *mut u64;
        unsafe { ptr::write_volatile(comparator_addr, value) };
    }
}

impl Hpet {
    fn enable_all(self) -> Self {
        unsafe {
            let enable_cnf_addr = (self.base_address + 0x10) as *mut u64;
            let old_cnf = ptr::read_volatile(enable_cnf_addr);
            ptr::write_volatile(enable_cnf_addr, old_cnf | 1);
        }

        self
    }

    fn enable_timer(self) -> Self {
        let timer_config_addr = (self.base_address + 0x100) as *mut u64;

        unsafe {
            let old_config = ptr::read_volatile(timer_config_addr);
            let route_cap = old_config.get_bits(32..63);

            if !route_cap.get_bit(IrqVector::HpetTimer as usize) {
                log::warn!("HPET timer does not support our IRQ vector!");
                log::info!("Timer route capabilities: {:#032b}", route_cap);
            }

            let timer_config = ((IrqVector::HpetTimer as u64) << 9) | (1 << 2);
            ptr::write_volatile(timer_config_addr, timer_config);
        }

        self
    }
}
