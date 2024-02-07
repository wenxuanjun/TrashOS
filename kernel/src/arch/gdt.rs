use spin::Lazy;
use x86_64::instructions::segmentation::{Segment, CS, SS};
use x86_64::instructions::tables::load_tss;
use x86_64::structures::gdt::GlobalDescriptorTable;
use x86_64::structures::gdt::{Descriptor, SegmentSelector};
use x86_64::structures::tss::TaskStateSegment;
use x86_64::VirtAddr;

pub const GENERAL_INTERRUPT_IST_INDEX: u16 = 0;

pub fn init() {
    let descriptor_table = &GDT.0;
    descriptor_table.load();

    unsafe {
        let selectors = &GDT.1;
        CS::set_reg(selectors.code_selector);
        SS::set_reg(selectors.data_selector);
        load_tss(selectors.tss_selector);
    }
}

#[derive(Debug)]
pub struct Selectors {
    code_selector: SegmentSelector,
    data_selector: SegmentSelector,
    user_code_selector: SegmentSelector,
    user_data_selector: SegmentSelector,
    tss_selector: SegmentSelector,
}

impl Selectors {
    pub fn get_kernel_segments() -> (SegmentSelector, SegmentSelector) {
        let selectors = &GDT.1;
        (selectors.code_selector, selectors.data_selector)
    }
    pub fn get_user_segments() -> (SegmentSelector, SegmentSelector) {
        let selectors = &GDT.1;
        (selectors.user_code_selector, selectors.user_data_selector)
    }
}

static GDT: Lazy<(GlobalDescriptorTable, Selectors)> = Lazy::new(|| {
    let mut gdt = GlobalDescriptorTable::new();

    let code_selector = gdt.add_entry(Descriptor::kernel_code_segment());
    let data_selector = gdt.add_entry(Descriptor::kernel_data_segment());

    let user_data_selector = gdt.add_entry(Descriptor::user_data_segment());
    let user_code_selector = gdt.add_entry(Descriptor::user_code_segment());

    let tss_selector = gdt.add_entry(Descriptor::tss_segment(&TSS));

    let selectors = Selectors {
        code_selector,
        data_selector,
        user_code_selector,
        user_data_selector,
        tss_selector,
    };

    (gdt, selectors)
});

pub static TSS: Lazy<TaskStateSegment> = Lazy::new(|| {
    let mut tss = TaskStateSegment::new();

    tss.interrupt_stack_table[GENERAL_INTERRUPT_IST_INDEX as usize] = {
        const STACK_SIZE: usize = 4096 * 5;
        static mut STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];
        let stack_start = VirtAddr::from_ptr(unsafe { core::ptr::addr_of!(STACK) });
        stack_start + STACK_SIZE
    };

    tss
});
