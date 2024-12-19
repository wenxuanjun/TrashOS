use crate::{println, syscall};
use core::panic::PanicInfo;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("panicked: {}", info.message());
    syscall::exit();
}
