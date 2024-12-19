use alloc::collections::BTreeMap;
use limine::request::SmpRequest;
use spin::{Lazy, RwLock};

use super::ap_entry;
use super::gdt::CpuInfo;

#[used]
#[unsafe(link_section = ".requests")]
static SMP_REQUEST: SmpRequest = SmpRequest::new();

pub static BSP_LAPIC_ID: Lazy<u32> =
    Lazy::new(|| SMP_REQUEST.get_response().unwrap().bsp_lapic_id());

pub static CPUS: Lazy<RwLock<Cpus>> = Lazy::new(|| RwLock::new(Cpus::default()));

pub struct Cpus(BTreeMap<u32, CpuInfo>);

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
        cpus.insert(*BSP_LAPIC_ID, CpuInfo::default());
        Cpus(cpus)
    }
}

impl Cpus {
    pub fn load(&mut self, lapic_id: u32) {
        let cpu_info = self.get_mut(lapic_id);
        cpu_info.init();
    }

    pub fn init_ap(&mut self) {
        let response = SMP_REQUEST.get_response().unwrap();

        for cpu in response.cpus() {
            if cpu.id != *BSP_LAPIC_ID {
                self.0.insert(cpu.lapic_id, CpuInfo::default());
                cpu.goto_address.write(ap_entry);
            }
        }
    }
}
