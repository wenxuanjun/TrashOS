use x86_64::instructions::segmentation::{Segment, CS, SS};
use x86_64::instructions::tables::load_tss;
use x86_64::structures::gdt::GlobalDescriptorTable;
use x86_64::structures::gdt::{Descriptor, SegmentSelector};
use x86_64::structures::tss::TaskStateSegment;
use x86_64::VirtAddr;

use super::smp::CPUS;

pub const DOUBLE_FAULT_IST_INDEX: usize = 0;

pub struct CpuInfo {
    pub gdt: GlobalDescriptorTable,
    pub tss: TaskStateSegment,
    pub selectors: Option<Selectors>,
    pub double_fault_stack: [u8; 4096],
}

impl CpuInfo {
    pub fn new() -> Self {
        Self {
            gdt: GlobalDescriptorTable::new(),
            tss: TaskStateSegment::new(),
            selectors: None,
            double_fault_stack: [0; 4096],
        }
    }

    pub fn init(&mut self) {
        let stack_start = self.double_fault_stack.as_ptr() as u64;
        let stack_end = VirtAddr::new(stack_start + self.double_fault_stack.len() as u64);
        log::warn!("stack_end: {:#x}", stack_end.as_u64());

        self.tss.interrupt_stack_table[DOUBLE_FAULT_IST_INDEX] = stack_end;
        self.selectors = Some(Selectors::new(&mut self.gdt, &self.tss));
    }

    pub fn load(&self) {
        let gdt_ptr: *const _ = &self.gdt;
        unsafe { (&*gdt_ptr).load() }

        let selectors = &self.selectors.as_ref().unwrap();
        unsafe {
            CS::set_reg(selectors.code_selector);
            SS::set_reg(selectors.data_selector);
            load_tss(selectors.tss_selector);
        }
    }

    pub fn set_ring0_rsp(&mut self, rsp: VirtAddr) {
        self.tss.privilege_stack_table[0] = rsp;
    }
}

const CODE_SELECTOR: Descriptor = Descriptor::kernel_code_segment();
const DATA_SELECTOR: Descriptor = Descriptor::kernel_data_segment();
const USER_DATA_SELECTOR: Descriptor = Descriptor::user_data_segment();
const USER_CODE_SELECTOR: Descriptor = Descriptor::user_code_segment();

pub struct Selectors {
    code_selector: SegmentSelector,
    data_selector: SegmentSelector,
    user_code_selector: SegmentSelector,
    user_data_selector: SegmentSelector,
    tss_selector: SegmentSelector,
}

impl Selectors {
    pub fn new(gdt: &mut GlobalDescriptorTable, tss: &TaskStateSegment) -> Self {
        let code_selector = gdt.append(CODE_SELECTOR);
        let data_selector = gdt.append(DATA_SELECTOR);
        let user_data_selector = gdt.append(USER_DATA_SELECTOR);
        let user_code_selector = gdt.append(USER_CODE_SELECTOR);

        let tss_ptr: *const _ = tss;
        log::warn!("tss_ptr: {:#x}", tss_ptr as u64);
        let tss_selector = gdt.append(Descriptor::tss_segment(unsafe { &*tss_ptr }));

        let selectors = Self {
            code_selector,
            data_selector,
            user_code_selector,
            user_data_selector,
            tss_selector,
        };

        selectors
    }

    pub fn get_kernel_segments() -> (SegmentSelector, SegmentSelector) {
        let mut bsp_cpu = CPUS.lock();
        let selectors = &bsp_cpu.bsp_cpu().selectors.as_ref().unwrap();
        (selectors.code_selector, selectors.data_selector)
    }
    pub fn get_user_segments() -> (SegmentSelector, SegmentSelector) {
        let mut bsp_cpu = CPUS.lock();
        let selectors = &bsp_cpu.bsp_cpu().selectors.as_ref().unwrap();
        (selectors.user_code_selector, selectors.user_data_selector)
    }
}
