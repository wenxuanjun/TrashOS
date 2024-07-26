use spin::Lazy;
use x86_64::instructions::segmentation::{Segment, CS, SS};
use x86_64::instructions::tables::load_tss;
use x86_64::structures::gdt::GlobalDescriptorTable;
use x86_64::structures::gdt::{Descriptor, SegmentSelector};
use x86_64::structures::tss::TaskStateSegment;
use x86_64::VirtAddr;

pub const DOUBLE_FAULT_IST_INDEX: usize = 0;
const FAULT_STACK_SIZE: usize = 256;

pub struct CpuInfo {
    gdt: GlobalDescriptorTable,
    tss: TaskStateSegment,
    selectors: Option<Selectors>,
    fault_stack: [u8; FAULT_STACK_SIZE],
}

impl CpuInfo {
    pub fn new() -> Self {
        Self {
            gdt: GlobalDescriptorTable::new(),
            tss: TaskStateSegment::new(),
            selectors: None,
            fault_stack: [0; FAULT_STACK_SIZE],
        }
    }

    pub fn init(&mut self) {
        let (mut gdt, mut selectors) = COMMON_GDT.clone();

        self.tss.interrupt_stack_table[DOUBLE_FAULT_IST_INDEX] = {
            let stack_start = self.fault_stack.as_ptr() as u64;
            VirtAddr::new(stack_start + self.fault_stack.len() as u64)
        };

        let tss_ptr: *const _ = &self.tss;
        let tss_selector = Some(gdt.append(Descriptor::tss_segment(unsafe { &*tss_ptr })));
        selectors.tss_selector = tss_selector;

        self.gdt = gdt;
        self.selectors = Some(selectors);
    }

    pub fn load(&self) {
        let gdt_ptr: *const _ = &self.gdt;
        unsafe { (&*gdt_ptr).load() }

        let selectors = &self.selectors.as_ref().unwrap();
        unsafe {
            CS::set_reg(selectors.code_selector);
            SS::set_reg(selectors.data_selector);
            load_tss(selectors.tss_selector.unwrap());
        }
    }

    pub fn set_ring0_rsp(&mut self, rsp: VirtAddr) {
        self.tss.privilege_stack_table[0] = rsp;
    }
}

static COMMON_GDT: Lazy<(GlobalDescriptorTable, Selectors)> = Lazy::new(|| {
    let mut gdt = GlobalDescriptorTable::new();

    let code_selector = gdt.append(Descriptor::kernel_code_segment());
    let data_selector = gdt.append(Descriptor::kernel_data_segment());
    let user_data_selector = gdt.append(Descriptor::user_data_segment());
    let user_code_selector = gdt.append(Descriptor::user_code_segment());

    let selectors = Selectors {
        code_selector,
        data_selector,
        user_data_selector,
        user_code_selector,
        tss_selector: None,
    };

    (gdt, selectors)
});

#[derive(Clone)]
pub struct Selectors {
    code_selector: SegmentSelector,
    data_selector: SegmentSelector,
    user_code_selector: SegmentSelector,
    user_data_selector: SegmentSelector,
    tss_selector: Option<SegmentSelector>,
}

impl Selectors {
    pub fn get_kernel_segments() -> (SegmentSelector, SegmentSelector) {
        let selectors = &COMMON_GDT.1;
        (selectors.code_selector, selectors.data_selector)
    }
    pub fn get_user_segments() -> (SegmentSelector, SegmentSelector) {
        let selectors = &COMMON_GDT.1;
        (selectors.user_code_selector, selectors.user_data_selector)
    }
}
