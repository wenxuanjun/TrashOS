use spin::Lazy;
use x86_64::VirtAddr;
use x86_64::instructions::port::PortReadOnly;
use x86_64::registers::control::Cr2;
use x86_64::structures::idt::InterruptDescriptorTable;
use x86_64::structures::idt::InterruptStackFrame;
use x86_64::structures::idt::PageFaultErrorCode;

const INTERRUPT_INDEX_OFFSET: u8 = 32;
pub const IOAPIC_INTERRUPT_INDEX_OFFSET: u8 = 32;

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum InterruptIndex {
    Timer = INTERRUPT_INDEX_OFFSET,
    ApicError,
    ApicSpurious,
    Keyboard,
    Mouse,
    Syscall,
}

pub static IDT: Lazy<InterruptDescriptorTable> = Lazy::new(|| {
    let mut idt = InterruptDescriptorTable::new();

    idt.breakpoint.set_handler_fn(breakpoint);
    idt[InterruptIndex::ApicError as usize].set_handler_fn(lapic_error);
    idt[InterruptIndex::ApicSpurious as usize].set_handler_fn(spurious_interrupt);

    unsafe {
        idt[InterruptIndex::Timer as usize]
            .set_handler_fn(timer_interrupt)
            .set_stack_index(crate::gdt::TIMER_INTERRUPT_IST_INDEX);

        idt.page_fault
            .set_handler_fn(page_fault)
            .set_stack_index(crate::gdt::GENERAL_INTERRUPT_IST_INDEX);
        idt.general_protection_fault
            .set_handler_fn(general_protection_fault)
            .set_stack_index(crate::gdt::GENERAL_INTERRUPT_IST_INDEX);
        idt.double_fault
            .set_handler_fn(double_fault)
            .set_stack_index(crate::gdt::GENERAL_INTERRUPT_IST_INDEX);
        idt[InterruptIndex::Keyboard as usize]
            .set_handler_fn(keyboard_interrupt)
            .set_stack_index(crate::gdt::GENERAL_INTERRUPT_IST_INDEX);
        idt[InterruptIndex::Mouse as usize]
            .set_handler_fn(mouse_interrupt)
            .set_stack_index(crate::gdt::GENERAL_INTERRUPT_IST_INDEX);
        idt.segment_not_present
            .set_handler_fn(segment_not_present)
            .set_stack_index(crate::gdt::GENERAL_INTERRUPT_IST_INDEX);
        idt.invalid_opcode
            .set_handler_fn(invalid_opcode)
            .set_stack_index(crate::gdt::GENERAL_INTERRUPT_IST_INDEX);
    }
    return idt;
});

#[naked]
extern "x86-interrupt" fn timer_interrupt(_frame: InterruptStackFrame) {
    unsafe {
        core::arch::asm!(
            "cli",
            crate::push_context!(),
            "mov rdi, rsp",
            "call {timer_handler}",
            "cmp rax, 0",
            "je 1f",
            "mov rsp, rax",
            "1:",
            crate::pop_context!(),
            "sti",
            "iretq",
            timer_handler = sym timer_interrupt_handler,
            options(noreturn)
        );
    }
}

extern "C" fn timer_interrupt_handler(context_address: VirtAddr) -> VirtAddr {
    let lapic = crate::apic::LAPIC.try_get().unwrap();
    unsafe { lapic.lock().end_of_interrupt() }

    let mut scheduer = crate::task::scheduler::SCHEDULER.write();
    scheduer.schedule(context_address).unwrap_or(VirtAddr::zero())
}

extern "x86-interrupt" fn spurious_interrupt(_frame: InterruptStackFrame) {
    crate::debug!("Received spurious interrupt!");
    let lapic = crate::apic::LAPIC.try_get().unwrap();
    unsafe { lapic.lock().end_of_interrupt() }
}

extern "x86-interrupt" fn lapic_error(_frame: InterruptStackFrame) {
    crate::error!("Local APIC error!");
    let lapic = crate::apic::LAPIC.try_get().unwrap();
    unsafe { lapic.lock().end_of_interrupt() }
}

extern "x86-interrupt" fn segment_not_present(frame: InterruptStackFrame, error: u64) {
    crate::error!("Exception: Segment Not Present\n{:#?}", frame);
    crate::error!("Error Code: {:#x}", error);
    panic!("Unrecoverable fault occured, halting!");
}

extern "x86-interrupt" fn general_protection_fault(frame: InterruptStackFrame, error: u64) {
    crate::error!("Exception: General Protection Fault\n{:#?}", frame);
    crate::error!("Error Code: {:#x}", error);
    x86_64::instructions::hlt();
}

extern "x86-interrupt" fn invalid_opcode(frame: InterruptStackFrame) {
    crate::error!("Exception: Invalid Opcode\n{:#?}", frame);
    x86_64::instructions::hlt();
}

extern "x86-interrupt" fn breakpoint(frame: InterruptStackFrame) {
    crate::debug!("Exception: Breakpoint\n{:#?}", frame);
}

extern "x86-interrupt" fn double_fault(frame: InterruptStackFrame, error: u64) -> ! {
    crate::error!("Exception: Double Fault\n{:#?}", frame);
    crate::error!("Error Code: {:#x}", error);
    panic!("Unrecoverable fault occured, halting!");
}

extern "x86-interrupt" fn keyboard_interrupt(_frame: InterruptStackFrame) {
    let scancode: u8 = unsafe { PortReadOnly::new(0x60).read() };
    crate::device::keyboard::add_scancode(scancode);
    let lapic = crate::apic::LAPIC.try_get().unwrap();
    unsafe { lapic.lock().end_of_interrupt() }
}

extern "x86-interrupt" fn mouse_interrupt(_frame: InterruptStackFrame) {
    let packet = unsafe { PortReadOnly::new(0x60).read() };
    let mouse = crate::device::mouse::MOUSE.try_get().unwrap();
    mouse.lock().process_packet(packet);
    let lapic = crate::apic::LAPIC.try_get().unwrap();
    unsafe { lapic.lock().end_of_interrupt() }
}

extern "x86-interrupt" fn page_fault(frame: InterruptStackFrame, error: PageFaultErrorCode) {
    let fault_addr = Cr2::read();
    crate::warn!("Exception: Page Fault\n{:#?}", frame);
    crate::warn!("Error Code: {:#x}", error);
    crate::warn!("Fault Address: {:#x}", fault_addr);
    x86_64::instructions::hlt();
}
