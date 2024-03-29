use spin::Lazy;
use x86_64::instructions::port::PortReadOnly;
use x86_64::registers::control::Cr2;
use x86_64::structures::idt::InterruptDescriptorTable;
use x86_64::structures::idt::InterruptStackFrame;
use x86_64::structures::idt::PageFaultErrorCode;
use x86_64::VirtAddr;

use super::gdt::GENERAL_INTERRUPT_IST_INDEX;
use crate::task::scheduler::SCHEDULER;

const INTERRUPT_INDEX_OFFSET: u8 = 32;

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum InterruptIndex {
    Timer = INTERRUPT_INDEX_OFFSET,
    ApicError,
    ApicSpurious,
    Keyboard,
    Mouse,
}

pub static IDT: Lazy<InterruptDescriptorTable> = Lazy::new(|| {
    let mut idt = InterruptDescriptorTable::new();

    idt.breakpoint.set_handler_fn(breakpoint);
    idt[InterruptIndex::ApicError as usize].set_handler_fn(lapic_error);
    idt[InterruptIndex::ApicSpurious as usize].set_handler_fn(spurious_interrupt);

    unsafe {
        idt[InterruptIndex::Timer as usize]
            .set_handler_fn(timer_interrupt)
            .set_stack_index(GENERAL_INTERRUPT_IST_INDEX);
        idt.page_fault
            .set_handler_fn(page_fault)
            .set_stack_index(GENERAL_INTERRUPT_IST_INDEX);
        idt.general_protection_fault
            .set_handler_fn(general_protection_fault)
            .set_stack_index(GENERAL_INTERRUPT_IST_INDEX);
        idt.double_fault
            .set_handler_fn(double_fault)
            .set_stack_index(GENERAL_INTERRUPT_IST_INDEX);
        idt.segment_not_present
            .set_handler_fn(segment_not_present)
            .set_stack_index(GENERAL_INTERRUPT_IST_INDEX);
        idt.invalid_opcode
            .set_handler_fn(invalid_opcode)
            .set_stack_index(GENERAL_INTERRUPT_IST_INDEX);
        idt[InterruptIndex::Keyboard as usize]
            .set_handler_fn(keyboard_interrupt)
            .set_stack_index(GENERAL_INTERRUPT_IST_INDEX);
        idt[InterruptIndex::Mouse as usize]
            .set_handler_fn(mouse_interrupt)
            .set_stack_index(GENERAL_INTERRUPT_IST_INDEX);
    }

    return idt;
});

#[naked]
extern "x86-interrupt" fn timer_interrupt(_frame: InterruptStackFrame) {
    fn timer_handler(context: VirtAddr) -> VirtAddr {
        super::apic::end_of_interrupt();
        SCHEDULER.try_get().unwrap().write().schedule(context)
    }

    unsafe {
        core::arch::asm!(
            "cli",
            crate::push_context!(),
            "mov rdi, rsp",
            "call {timer_handler}",
            "mov rsp, rax",
            crate::pop_context!(),
            "sti",
            "iretq",
            timer_handler = sym timer_handler,
            options(noreturn)
        );
    }
}

extern "x86-interrupt" fn lapic_error(_frame: InterruptStackFrame) {
    log::error!("Local APIC error!");
    super::apic::end_of_interrupt();
}

extern "x86-interrupt" fn spurious_interrupt(_frame: InterruptStackFrame) {
    log::debug!("Received spurious interrupt!");
    super::apic::end_of_interrupt();
}

extern "x86-interrupt" fn segment_not_present(frame: InterruptStackFrame, error_code: u64) {
    log::error!("Exception: Segment Not Present\n{:#?}", frame);
    log::error!("Error Code: {:#x}", error_code);
    panic!("Unrecoverable fault occured, halting!");
}

extern "x86-interrupt" fn general_protection_fault(frame: InterruptStackFrame, error_code: u64) {
    log::error!("Exception: General Protection Fault\n{:#?}", frame);
    log::error!("Error Code: {:#x}", error_code);
    x86_64::instructions::hlt();
}

extern "x86-interrupt" fn invalid_opcode(frame: InterruptStackFrame) {
    log::error!("Exception: Invalid Opcode\n{:#?}", frame);
    x86_64::instructions::hlt();
}

extern "x86-interrupt" fn breakpoint(frame: InterruptStackFrame) {
    log::debug!("Exception: Breakpoint\n{:#?}", frame);
}

extern "x86-interrupt" fn double_fault(frame: InterruptStackFrame, error_code: u64) -> ! {
    log::error!("Exception: Double Fault\n{:#?}", frame);
    log::error!("Error Code: {:#x}", error_code);
    panic!("Unrecoverable fault occured, halting!");
}

extern "x86-interrupt" fn keyboard_interrupt(_frame: InterruptStackFrame) {
    let scancode: u8 = unsafe { PortReadOnly::new(0x60).read() };
    crate::device::keyboard::add_scancode(scancode);
    super::apic::end_of_interrupt();
}

extern "x86-interrupt" fn mouse_interrupt(_frame: InterruptStackFrame) {
    let packet = unsafe { PortReadOnly::new(0x60).read() };
    crate::device::mouse::MOUSE.lock().process_packet(packet);
    super::apic::end_of_interrupt();
}

extern "x86-interrupt" fn page_fault(frame: InterruptStackFrame, error_code: PageFaultErrorCode) {
    let fault_addr = Cr2::read();
    log::warn!("Exception: Page Fault\n{:#?}", frame);
    log::warn!("Error Code: {:#x}", error_code);
    log::warn!("Fault Address: {:#x}", fault_addr);
    x86_64::instructions::hlt();
}
