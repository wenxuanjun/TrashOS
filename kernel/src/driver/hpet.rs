use core::ptr;
use core::time::Duration;
use spin::Lazy;
use x86_64::{PhysAddr, VirtAddr};

use crate::arch::acpi::ACPI;
use crate::arch::apic::IrqVector;
use crate::mem::convert_physical_to_virtual;

pub static HPET: Lazy<Hpet> = Lazy::new(|| {
    let physical_address = PhysAddr::new(ACPI.hpet_info.base_address as u64);
    Hpet::new(convert_physical_to_virtual(physical_address)).enable()
});

pub struct Hpet {
    address: VirtAddr,
    fms_per_tick: u32,
}

impl Hpet {
    pub fn new(address: VirtAddr) -> Self {
        let period_addr = (address + 0x4).as_ptr();

        Self {
            address,
            fms_per_tick: unsafe { ptr::read_volatile(period_addr) },
        }
    }

    pub fn elapsed(&self) -> Duration {
        let ticks = self.elapsed_ticks();
        Duration::from_nanos(ticks * self.fms_per_tick as u64 / 1_000_000)
    }

    pub fn estimate(&self, duration: Duration) -> u64 {
        let ticks = self.elapsed_ticks();
        ticks + (duration.as_nanos() * 1_000_000 / self.fms_per_tick as u128) as u64
    }

    pub fn set_timer(&self, value: u64) {
        let comparator_addr = self.address + 0x108;
        unsafe {
            ptr::write_volatile(comparator_addr.as_mut_ptr(), value);
        }
    }

    pub fn busy_wait(&self, duration: Duration) {
        let start = self.elapsed_ticks();
        let ticks = duration.as_nanos() * 1_000_000 / self.fms_per_tick as u128;
        while self.elapsed_ticks() < start + ticks as u64 {
            core::hint::spin_loop()
        }
    }
}

impl Hpet {
    fn enable(self) -> Self {
        let enable_cnf_addr = self.address + 0x10;
        let timer_config_addr = self.address + 0x100;
        unsafe {
            let old_cnf: u64 = ptr::read_volatile(enable_cnf_addr.as_ptr());
            ptr::write_volatile(enable_cnf_addr.as_mut_ptr(), old_cnf | 1);
            let timer_config = ((IrqVector::HpetTimer as u32) << 9) | (1 << 2);
            ptr::write_volatile(timer_config_addr.as_mut_ptr(), timer_config);
        }
        self
    }

    fn elapsed_ticks(&self) -> u64 {
        let counter_addr = self.address + 0xf0;
        unsafe { ptr::read_volatile(counter_addr.as_ptr()) }
    }
}
