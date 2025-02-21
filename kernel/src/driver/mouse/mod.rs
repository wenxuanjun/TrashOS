mod driver;
mod event;
mod packet;
mod ports;

pub use driver::{MOUSE, MOUSE_BUFFER};
pub use event::MouseEvent;

pub fn init() {
    let mut mouse = MOUSE.lock();

    if let Err(err) = mouse.init() {
        log::error!("Failed to init mouse: {}", err);
    } else {
        log::debug!("Mouse: {mouse}");
    }
}
