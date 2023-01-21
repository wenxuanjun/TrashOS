use lazy_static::lazy_static;
use x86_64::structures::idt::InterruptStackFrame;
use x86_64::structures::idt::PageFaultErrorCode;
use x86_64::structures::idt::InterruptDescriptorTable;
use x86_64::instructions::port::Port;

use crate::print;

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum InterruptIndex {
    Timer = 32,
    ApicError,
    ApicSpurious,
    Keyboard,
    Mouse,
    Syscall,
}

lazy_static! {
    pub static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        idt.breakpoint.set_handler_fn(breakpoint);
        idt.page_fault.set_handler_fn(page_fault);
        idt.general_protection_fault.set_handler_fn(general_protection_fault);

        idt[InterruptIndex::Timer as usize].set_handler_fn(timer_interrupt);
        idt[InterruptIndex::ApicError as usize].set_handler_fn(lapic_error);
        idt[InterruptIndex::ApicSpurious as usize].set_handler_fn(spurious_interrupt);
        idt[InterruptIndex::Keyboard as usize].set_handler_fn(keyboard_interrupt);
        idt[InterruptIndex::Mouse as usize].set_handler_fn(mouse_interrupt);
        idt[InterruptIndex::Syscall as usize].set_handler_fn(syscall_handler);

        unsafe {
            idt.double_fault
                .set_handler_fn(double_fault)
                .set_stack_index(crate::gdt::DOUBLE_FAULT_IST_INDEX);
        }
        return idt;
    };
}

extern "x86-interrupt" fn spurious_interrupt(_frame: InterruptStackFrame) {
    crate::debug!("Received spurious interrupt!");
    unsafe { crate::apic::LAPIC.try_get().unwrap().lock().end_of_interrupt() }
}

extern "x86-interrupt" fn lapic_error(_frame: InterruptStackFrame) {
    crate::error!("Local APIC error!");
    unsafe { crate::apic::LAPIC.try_get().unwrap().lock().end_of_interrupt() }
}

extern "x86-interrupt" fn general_protection_fault(frame: InterruptStackFrame, _error: u64) {
    crate::error!("Exception: General Protection Fault!\n{:#?}", frame);
    panic!("Unrecoverable fault occured, halting!");
}

extern "x86-interrupt" fn breakpoint(frame: InterruptStackFrame) {
    crate::debug!("Exception: Breakpoint\n{:#?}", frame);
}

extern "x86-interrupt" fn double_fault(frame: InterruptStackFrame, _error: u64) -> ! {
    crate::error!("Exception: Double Fault\n{:#?}", frame);
    panic!("Unrecoverable fault occured, halting!");
}

extern "x86-interrupt" fn timer_interrupt(_frame: InterruptStackFrame) {
    unsafe { crate::apic::LAPIC.try_get().unwrap().lock().end_of_interrupt() }
}

extern "x86-interrupt" fn keyboard_interrupt(_frame: InterruptStackFrame) {
    let mut port = Port::new(0x60);
    let scancode: u8 = unsafe { port.read() };
    crate::task::keyboard::add_scancode(scancode);
    unsafe { crate::apic::LAPIC.try_get().unwrap().lock().end_of_interrupt() }
}

extern "x86-interrupt" fn mouse_interrupt(_frame: InterruptStackFrame) {
    let mut port = Port::new(0x60);
    let scancode: u8 = unsafe { port.read() };
    print!("Mouse scancode: {:x}\n", scancode);
    unsafe { crate::apic::LAPIC.try_get().unwrap().lock().end_of_interrupt() }
}

extern "x86-interrupt" fn syscall_handler(_frame: InterruptStackFrame) {
    crate::debug!("Syscall interrupt!");
    unsafe { crate::apic::LAPIC.try_get().unwrap().lock().end_of_interrupt() }
}

extern "x86-interrupt" fn page_fault(frame: InterruptStackFrame, error: PageFaultErrorCode) {
    let fault_addr = x86_64::registers::control::Cr2::read();
    crate::warn!("Exception: Page Fault");
    crate::warn!("Accessed Address: {:?}", fault_addr);
    crate::warn!("Error Code: {:?}", error);
    crate::warn!("Stack Frame: {:#?}", frame);
    x86_64::instructions::hlt();
}
