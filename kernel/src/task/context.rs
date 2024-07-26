use x86_64::structures::gdt::SegmentSelector;
use x86_64::{PhysAddr, VirtAddr};

#[derive(Debug, Clone, Copy, Default)]
#[repr(packed)]
#[allow(dead_code)]
pub struct Context {
    cr3: usize,
    r15: usize,
    r14: usize,
    r13: usize,

    r12: usize,
    r11: usize,
    r10: usize,
    r9: usize,

    r8: usize,
    rbp: usize,
    rsi: usize,
    rdi: usize,

    rdx: usize,
    rcx: usize,
    rbx: usize,
    rax: usize,

    rip: usize,
    cs: usize,
    rflags: usize,
    rsp: usize,
    ss: usize,
}

impl Context {
    pub fn init(
        &mut self,
        entry_point: usize,
        stack_end_address: VirtAddr,
        page_table_address: PhysAddr,
        segment_selectors: (SegmentSelector, SegmentSelector),
    ) {
        self.rflags = 0x200;
        self.rip = entry_point;
        self.rsp = stack_end_address.as_u64() as usize;
        self.cr3 = page_table_address.as_u64() as usize;

        let (code_selector, data_selector) = segment_selectors;
        self.cs = code_selector.0 as usize;
        self.ss = data_selector.0 as usize;
    }

    #[inline]
    pub fn address(&self) -> VirtAddr {
        VirtAddr::new(self as *const Context as u64)
    }

    #[inline]
    pub fn from_address(address: VirtAddr) -> Context {
        unsafe { *&mut *(address.as_u64() as *mut Context) }
    }
}

#[macro_export]
macro_rules! push_context {
    () => {
        concat!(
            r#"
            push rax
            push rbx
            push rcx
            push rdx
            push rdi
            push rsi
            push rbp
            push r8
            push r9
            push r10
            push r11
            push r12
            push r13
            push r14
            push r15
            mov r15, cr3
            push r15
            "#,
        )
    };
}

#[macro_export]
macro_rules! pop_context {
    () => {
        concat!(
            r#"
            pop r15
            mov cr3, r15
            pop r15
            pop r14
            pop r13
            pop r12
            pop r11
            pop r10
            pop r9
            pop r8
            pop rbp
            pop rsi
            pop rdi
            pop rdx
            pop rcx
            pop rbx
            pop rax
			"#
        )
    };
}
