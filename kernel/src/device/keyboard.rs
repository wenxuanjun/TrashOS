use crossbeam_queue::ArrayQueue;
use pc_keyboard::{layouts, DecodedKey, HandleControl, Keyboard, ScancodeSet1};
use spin::Lazy;

const SCANCODE_QUEUE_SIZE: usize = 128;

static SCANCODE_QUEUE: Lazy<ArrayQueue<u8>> = Lazy::new(|| ArrayQueue::new(SCANCODE_QUEUE_SIZE));

pub fn add_scancode(scancode: u8) {
    if let Err(_) = SCANCODE_QUEUE.push(scancode) {
        log::warn!("Scancode queue full, dropping keyboard input!");
    }
}

pub fn print_keypresses() {
    let mut keyboard = Keyboard::new(
        ScancodeSet1::new(),
        layouts::Us104Key,
        HandleControl::Ignore,
    );

    loop {
        if let Some(scancode) = SCANCODE_QUEUE.pop() {
            if let Ok(Some(key_event)) = keyboard.add_byte(scancode) {
                if let Some(key) = keyboard.process_keyevent(key_event) {
                    match key {
                        DecodedKey::Unicode(character) => crate::print!("{}", character),
                        DecodedKey::RawKey(key) => crate::print!("{:?}", key),
                    }
                }
            }
        }
    }
}
