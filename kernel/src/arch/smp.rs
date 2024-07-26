use alloc::boxed::Box;
use alloc::collections::BTreeMap;
use limine::request::SmpRequest;
use limine::response::SmpResponse;
use spin::{Lazy, RwLock};

use super::ap_entry;
use super::gdt::CpuInfo;

#[used]
#[link_section = ".requests"]
static SMP_REQUEST: SmpRequest = SmpRequest::new();

pub static CPUS: Lazy<RwLock<Cpus>> = Lazy::new(|| RwLock::new(Cpus::new()));
pub static BSP_LAPIC_ID: Lazy<u32> = Lazy::new(|| SMP_RESPONSE.bsp_lapic_id());
static SMP_RESPONSE: Lazy<&SmpResponse> = Lazy::new(|| SMP_REQUEST.get_response().unwrap());

pub struct Cpus(BTreeMap<u32, &'static mut CpuInfo>);

impl Cpus {
    pub fn get(&self, lapic_id: u32) -> &CpuInfo {
        self.0.get(&lapic_id).unwrap()
    }

    pub fn get_mut(&mut self, lapic_id: u32) -> &mut CpuInfo {
        self.0.get_mut(&lapic_id).unwrap()
    }

    pub fn iter_id(&self) -> impl Iterator<Item = &u32> {
        self.0.keys()
    }
}

impl Cpus {
    pub fn new() -> Self {
        let mut cpus = BTreeMap::new();
        cpus.insert(*BSP_LAPIC_ID, Box::leak(Box::new(CpuInfo::new())));
        Cpus(cpus)
    }

    pub fn init_bsp(&mut self) {
        let bsp_info = self.get_mut(*BSP_LAPIC_ID);
        bsp_info.init();
        bsp_info.load();
    }

    pub fn init_ap(&mut self) {
        for cpu in SMP_RESPONSE.cpus() {
            if cpu.id == *BSP_LAPIC_ID {
                continue;
            }
            let info = Box::leak(Box::new(CpuInfo::new()));
            info.init();
            self.0.insert(cpu.lapic_id, info);
            cpu.goto_address.write(ap_entry);
        }
    }
}
