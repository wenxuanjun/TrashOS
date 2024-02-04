use conquer_once::spin::OnceCell;
use crossbeam_queue::ArrayQueue;
use pc_keyboard::{layouts, DecodedKey, HandleControl, Keyboard, ScancodeSet1};

const SCANCODE_QUEUE_SIZE: usize = 128;

static SCANCODE_QUEUE: OnceCell<ArrayQueue<u8>> = OnceCell::uninit();

pub struct ScancodeStream;

impl ScancodeStream {
    pub fn new() -> Self {
        SCANCODE_QUEUE
            .try_init_once(|| ArrayQueue::new(SCANCODE_QUEUE_SIZE))
            .expect("ScancodeStream::new should only be called once!");
        ScancodeStream {}
    }

    pub fn next(&self) -> Option<u8> {
        let queue = SCANCODE_QUEUE
            .try_get()
            .expect("Scancode queue not initialized!");
        queue.pop()
    }
}

pub fn add_scancode(scancode: u8) {
    if let Ok(queue) = SCANCODE_QUEUE.try_get() {
        if let Err(_) = queue.push(scancode) {
            crate::println!("Scancode queue full, dropping keyboard input!");
        }
    } else {
        crate::println!("Scancode queue not initialized!");
    }
}

pub fn print_keypresses() {
    let scancodes = ScancodeStream::new();
    let mut keyboard = Keyboard::new(
        ScancodeSet1::new(),
        layouts::Us104Key,
        HandleControl::Ignore,
    );
    loop {
        if let Some(scancode) = scancodes.next() {
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
