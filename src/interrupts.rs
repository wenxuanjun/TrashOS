use lazy_static::lazy_static;
use pic8259::ChainedPics;
use spin::Mutex;
use x86_64::instructions::port::Port;
use x86_64::structures::idt::InterruptDescriptorTable;
use x86_64::structures::idt::InterruptStackFrame;
use x86_64::structures::idt::PageFaultErrorCode;

pub const PIC0_OFFSET: u8 = 32;
pub const PIC1_OFFSET: u8 = PIC0_OFFSET + 8;

#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum InterruptIndex {
    Timer = PIC0_OFFSET,
    Keyboard,
}

pub static PICS: Mutex<ChainedPics> =
    Mutex::new(unsafe { ChainedPics::new(PIC0_OFFSET, PIC1_OFFSET) });

lazy_static! {
    pub static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        idt.breakpoint.set_handler_fn(breakpoint);
        idt.page_fault.set_handler_fn(page_fault);
        idt[InterruptIndex::Timer as usize].set_handler_fn(timer_interrupt);
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
    unsafe {
        PICS.lock().notify_end_of_interrupt(InterruptIndex::Timer as u8);
    }
}

extern "x86-interrupt" fn keyboard_interrupt(_frame: InterruptStackFrame) {
    let mut port = Port::new(0x60);
    let scancode: u8 = unsafe { port.read() };
    crate::task::keyboard::add_scancode(scancode);

    unsafe {
        PICS.lock().notify_end_of_interrupt(InterruptIndex::Keyboard as u8);
    }
}

extern "x86-interrupt" fn breakpoint(frame: InterruptStackFrame) {
    crate::println!("Exception: Breakpoint\n{:#?}", frame);
}

extern "x86-interrupt" fn double_fault(frame: InterruptStackFrame, _error: u64) -> ! {
    panic!("Exception: Double Fault\n{:#?}", frame);
}

extern "x86-interrupt" fn page_fault(frame: InterruptStackFrame, error: PageFaultErrorCode) {
    let fault_addr = x86_64::registers::control::Cr2::read();
    crate::println!("Exception: Page Failt");
    crate::println!("Accessed Address: {:?}", fault_addr);
    crate::println!("Error Code: {:?}", error);
    crate::println!("Stack Frame: {:#?}", frame);
    x86_64::instructions::hlt();
}
