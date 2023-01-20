use lazy_static::lazy_static;
use x86_64::structures::idt::InterruptStackFrame;
use x86_64::structures::idt::PageFaultErrorCode;
use x86_64::structures::idt::InterruptDescriptorTable;
use x86_64::instructions::port::Port;

pub const PIC0_OFFSET: u8 = 32;
pub const PIC1_OFFSET: u8 = PIC0_OFFSET + 8;

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum InterruptIndex {
    Timer = 32,
    ApicError,
    ApicSpurious,
    Keyboard,
    Syscall,
}

lazy_static! {
    pub static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        idt.breakpoint.set_handler_fn(breakpoint);
        idt.page_fault.set_handler_fn(page_fault);

        idt[InterruptIndex::Timer as usize].set_handler_fn(timer_interrupt);
        idt[InterruptIndex::ApicSpurious as usize].set_handler_fn(spurious_interrupt);
        idt[InterruptIndex::Keyboard as usize].set_handler_fn(keyboard_interrupt);

        unsafe {
            idt.double_fault
                .set_handler_fn(double_fault)
                .set_stack_index(crate::gdt::DOUBLE_FAULT_IST_INDEX);
        }
        return idt;
    };
}

extern "x86-interrupt" fn timer_interrupt(_frame: InterruptStackFrame) {
    crate::print!(".");
    unsafe { crate::apic::LAPIC.lock().as_mut().unwrap().end_of_interrupt() };
}

extern "x86-interrupt" fn spurious_interrupt(_frame: InterruptStackFrame) {
    crate::debug!("Received spurious interrupt");
    unsafe { crate::apic::LAPIC.lock().as_mut().unwrap().end_of_interrupt() };
}

extern "x86-interrupt" fn keyboard_interrupt(_frame: InterruptStackFrame) {
    let mut port = Port::new(0x60);
    let scancode: u8 = unsafe { port.read() };
    crate::task::keyboard::add_scancode(scancode);
    unsafe { crate::apic::LAPIC.lock().as_mut().unwrap().end_of_interrupt() };
}

extern "x86-interrupt" fn breakpoint(frame: InterruptStackFrame) {
    crate::debug!("Exception: Breakpoint\n{:#?}", frame);
}

extern "x86-interrupt" fn double_fault(frame: InterruptStackFrame, _error: u64) -> ! {
    panic!("Exception: Double Fault\n{:#?}", frame);
}

extern "x86-interrupt" fn page_fault(frame: InterruptStackFrame, error: PageFaultErrorCode) {
    let fault_addr = x86_64::registers::control::Cr2::read();
    crate::warn!("Exception: Page Fault");
    crate::warn!("Accessed Address: {:?}", fault_addr);
    crate::warn!("Error Code: {:?}", error);
    crate::warn!("Stack Frame: {:#?}", frame);
    x86_64::instructions::hlt();
}
