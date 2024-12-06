use alloc::boxed::Box;
use alloc::collections::BTreeMap;
use limine::request::SmpRequest;
use spin::{Lazy, RwLock};

use super::ap_entry;
use super::gdt::CpuInfo;

#[used]
#[unsafe(link_section = ".requests")]
static SMP_REQUEST: SmpRequest = SmpRequest::new();

pub static CPUS: Lazy<RwLock<Cpus>> = Lazy::new(|| RwLock::new(Cpus::default()));

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

impl Default for Cpus {
    fn default() -> Self {
        let mut cpus = BTreeMap::new();
        let response = SMP_REQUEST.get_response().unwrap();
        let bsp_lapic_id = response.bsp_lapic_id();
        cpus.insert(bsp_lapic_id, Box::leak(Box::new(CpuInfo::default())));
        Cpus(cpus)
    }
}

impl Cpus {
    pub fn init_bsp(&mut self) {
        let response = SMP_REQUEST.get_response().unwrap();
        let bsp_info = self.get_mut(response.bsp_lapic_id());

        bsp_info.init();
        bsp_info.load();
    }

    pub fn init_ap(&mut self) {
        let response = SMP_REQUEST.get_response().unwrap();
        let bsp_lapic_id = response.bsp_lapic_id();

        for cpu in response.cpus() {
            if cpu.id != bsp_lapic_id {
                let info = Box::leak(Box::new(CpuInfo::default()));
                info.init();
                self.0.insert(cpu.lapic_id, info);
                cpu.goto_address.write(ap_entry);
            }
        }
    }
}
