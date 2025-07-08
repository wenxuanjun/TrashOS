use spin::Lazy;
use x86_64::VirtAddr;
use x86_64::instructions::port::Port;
use x86_64::registers::control::Cr2;
use x86_64::structures::idt::InterruptDescriptorTable;
use x86_64::structures::idt::InterruptStackFrame;
use x86_64::structures::idt::PageFaultErrorCode;

// use super::gdt::DOUBLE_FAULT_IST_INDEX;
use crate::drivers::term::SCANCODE_QUEUE;
use crate::tasks::scheduler::SCHEDULER;
use crate::tasks::timer::TIMER;

const INTERRUPT_INDEX_OFFSET: u8 = 32;

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum InterruptIndex {
    Timer = INTERRUPT_INDEX_OFFSET,
    ApicError,
    ApicSpurious,
    Keyboard,
    Mouse,
    HpetTimer,
}

pub static IDT: Lazy<InterruptDescriptorTable> = Lazy::new(|| {
    let mut idt = InterruptDescriptorTable::new();

    idt.breakpoint.set_handler_fn(breakpoint);
    idt.segment_not_present.set_handler_fn(segment_not_present);
    idt.invalid_opcode.set_handler_fn(invalid_opcode);
    idt.page_fault.set_handler_fn(page_fault);
    idt.general_protection_fault
        .set_handler_fn(general_protection_fault);

    idt[InterruptIndex::Timer as u8].set_handler_fn(timer_interrupt);
    idt[InterruptIndex::ApicError as u8].set_handler_fn(lapic_error);
    idt[InterruptIndex::ApicSpurious as u8].set_handler_fn(spurious_interrupt);
    idt[InterruptIndex::Keyboard as u8].set_handler_fn(keyboard_interrupt);
    idt[InterruptIndex::Mouse as u8].set_handler_fn(mouse_interrupt);
    idt[InterruptIndex::HpetTimer as u8].set_handler_fn(hpet_timer_interrupt);

    // unsafe {
    //     idt.double_fault
    //         .set_handler_fn(double_fault)
    //         .set_stack_index(DOUBLE_FAULT_IST_INDEX as u16);
    // }

    idt
});

#[unsafe(naked)]
pub extern "x86-interrupt" fn timer_interrupt(_frame: InterruptStackFrame) {
    fn timer_handler(context: VirtAddr) -> VirtAddr {
        super::apic::end_of_interrupt();
        SCHEDULER.lock().schedule(context)
    }

    core::arch::naked_asm!(
        crate::push_context!(),
        "mov rdi, rsp",
        "call {timer_handler}",
        "mov rsp, rax",
        crate::pop_context!(),
        "iretq",
        timer_handler = sym timer_handler,
    );
}

extern "x86-interrupt" fn lapic_error(_frame: InterruptStackFrame) {
    super::apic::end_of_interrupt();
    log::error!("Local APIC error!");
}

extern "x86-interrupt" fn spurious_interrupt(_frame: InterruptStackFrame) {
    super::apic::end_of_interrupt();
    log::debug!("Received spurious interrupt!");
}

extern "x86-interrupt" fn hpet_timer_interrupt(_frame: InterruptStackFrame) {
    TIMER.lock().wakeup();
    super::apic::end_of_interrupt();
}

extern "x86-interrupt" fn segment_not_present(frame: InterruptStackFrame, code: u64) {
    log::error!("Exception: Segment Not Present\n{frame:#?}");
    log::error!("Error Code: {code:#x}");
    panic!("Unrecoverable fault occured, halting!");
}

extern "x86-interrupt" fn general_protection_fault(frame: InterruptStackFrame, code: u64) {
    log::error!("Exception: General Protection Fault\n{frame:#?}");
    log::error!("Error Code: {code:#x}");
    x86_64::instructions::hlt();
}

extern "x86-interrupt" fn invalid_opcode(frame: InterruptStackFrame) {
    log::error!("Exception: Invalid Opcode\n{frame:#?}");
    x86_64::instructions::hlt();
}

extern "x86-interrupt" fn breakpoint(frame: InterruptStackFrame) {
    log::debug!("Exception: Breakpoint\n{frame:#?}");
}

// extern "x86-interrupt" fn double_fault(frame: InterruptStackFrame, code: u64) {
//     log::error!("Exception: Double Fault\n{frame:#?}");
//     log::error!("Error Code: {code:#x}");
//     panic!("Unrecoverable fault occured, halting!");
// }

extern "x86-interrupt" fn keyboard_interrupt(_frame: InterruptStackFrame) {
    super::apic::end_of_interrupt();
    let scancode = unsafe { Port::new(0x60).read() };
    SCANCODE_QUEUE.force_push(scancode);
}

extern "x86-interrupt" fn mouse_interrupt(_frame: InterruptStackFrame) {
    let packet = unsafe { Port::new(0x60).read() };
    crate::drivers::mouse::MOUSE.lock().process_packet(packet);
    super::apic::end_of_interrupt();
}

extern "x86-interrupt" fn page_fault(frame: InterruptStackFrame, code: PageFaultErrorCode) {
    log::warn!("Exception: Page Fault\n{frame:#?}");
    log::warn!("Error Code: {code:#x}");
    match Cr2::read() {
        Ok(address) => {
            log::warn!("Fault Address: {address:#x}");
        }
        Err(error) => {
            log::warn!("Invalid virtual address: {error:?}");
        }
    }
    panic!("Cannot recover from page fault, halting!");
}
