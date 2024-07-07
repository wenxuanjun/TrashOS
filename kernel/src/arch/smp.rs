use core::sync::atomic::Ordering;

use alloc::collections::BTreeMap;
use limine::request::SmpRequest;
use limine::smp::Cpu;
use spin::{Lazy, Mutex};

use super::apic::{calibrate_timer, APIC_INIT, LAPIC};
use super::gdt::CpuInfo;
use super::interrupts::IDT;
use crate::syscall;
use crate::task::scheduler::SCHEDULER_INIT;

#[used]
#[link_section = ".requests"]
static SMP_REQUEST: SmpRequest = SmpRequest::new();

pub static CPUS: Lazy<Mutex<Cpus>> = Lazy::new(|| Mutex::new(Cpus::new()));

unsafe extern "C" fn ap_entry(smp_info: &Cpu) -> ! {
    crate::println!("Processor: {} start", smp_info.id);
    CPUS.lock().get_cpu(smp_info.id as usize).load();
    crate::println!("Processor: {} after load", smp_info.id);
    IDT.load();
    crate::println!("Processor: {} after idt", smp_info.id);

    while !APIC_INIT.load(Ordering::Relaxed) {}
    crate::println!("Processor: {} after lapic", smp_info.id);
    LAPIC.lock().enable();
    calibrate_timer();
    crate::println!("Processor: {} after calibrate_timer", smp_info.id);

    while !SCHEDULER_INIT.load(Ordering::Relaxed) {}
    crate::println!("Processor: {} after SCHEDULER_INIT", smp_info.id);

    syscall::init();
    x86_64::instructions::interrupts::enable();

    loop {
        x86_64::instructions::hlt();
    }
}

pub struct Cpus {
    bsp: CpuInfo,
    bsp_lapic_id: u32,
    ap_infos: BTreeMap<u32, CpuInfo>,
}

impl Cpus {
    pub fn new() -> Self {
        let response = SMP_REQUEST.get_response().unwrap();

        Self {
            bsp: CpuInfo::new(),
            bsp_lapic_id: response.bsp_lapic_id(),
            ap_infos: BTreeMap::new(),
        }
    }

    pub fn init_bsp(&mut self) {
        self.bsp.init();
        self.bsp.load();

        let tss_ptr = &self.bsp.tss as *const _;
        log::warn!("bsp tss_ptr: {:#x}", tss_ptr as u64);

        let stack_start = self.bsp.double_fault_stack.as_ptr();
        log::warn!("bsp stack start: {:#x}", stack_start as u64);
    }

    pub fn init_ap(&mut self) {
        let response = SMP_REQUEST.get_response().unwrap();

        for cpu in response.cpus() {
            if cpu.id != self.bsp_lapic_id {
                let info = CpuInfo::new();
                self.ap_infos.insert(cpu.lapic_id, info);

                let info = self.ap_infos.get_mut(&cpu.lapic_id).unwrap();
                info.init();

                cpu.goto_address.write(ap_entry);
                log::info!("AP CPU {} initialized!", cpu.lapic_id);

                let tss_ptr = &info.tss as *const _;
                log::warn!("ap tss_ptr: {:#x}", tss_ptr as u64);

                let stack = &self.ap_infos.get(&1).unwrap().double_fault_stack;
                let stack_end = stack.as_ptr() as u64 + stack.len() as u64;
                log::warn!("ap stack start: {:#x}", stack_end as u64);
            }
        }
    }

    pub fn get_cpu(&mut self, id: usize) -> &mut CpuInfo {
        if id == self.bsp_lapic_id as usize {
            self.bsp_cpu()
        } else {
            self.ap_infos
                .get_mut(&(id as u32))
                .unwrap_or_else(|| panic!("CPU {} not found!", id))
        }
    }

    pub fn bsp_cpu(&mut self) -> &mut CpuInfo {
        &mut self.bsp
    }

    pub fn current_cpu(&mut self) -> (u32, &mut CpuInfo) {
        let current_cpu_id = unsafe { LAPIC.lock().id() };
        (current_cpu_id, self.get_cpu(current_cpu_id as usize))
    }
}
