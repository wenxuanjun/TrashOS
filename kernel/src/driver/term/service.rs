use alloc::boxed::Box;
use core::fmt::Write;
use core::time::Duration;
use crossbeam_queue::ArrayQueue;
use os_terminal::font::BitmapFont;
use os_terminal::{MouseInput, Terminal};
use spin::Lazy;

use super::writer::TerminalWriter;
use crate::driver::mouse::{MOUSE_BUFFER, MouseEvent};
use crate::driver::{display::Display, speaker::SPEAKER};
use crate::syscall::r#yield;

const SCANCODE_QUEUE_SIZE: usize = 256;
const TERMINAL_BUFFER_SIZE: usize = 2048;

pub static SCANCODE_QUEUE: Lazy<ArrayQueue<u8>> =
    Lazy::new(|| ArrayQueue::new(SCANCODE_QUEUE_SIZE));
pub static TERMINAL_BUFFER: Lazy<ArrayQueue<char>> =
    Lazy::new(|| ArrayQueue::new(TERMINAL_BUFFER_SIZE));

pub fn terminal_thread() {
    let mut terminal = Terminal::new(Display::default());
    terminal.set_auto_crnl(true);
    terminal.set_auto_flush(false);
    terminal.set_scroll_speed(5);

    terminal.set_font_manager(Box::new(BitmapFont));
    let bell_handler = || SPEAKER.lock().beep(750, Duration::from_millis(200));
    terminal.set_bell_handler(Some(bell_handler));

    loop {
        terminal_event(&mut terminal);
        terminal_flush(&mut terminal);
        r#yield();
    }
}

fn terminal_flush(terminal: &mut Terminal<Display>) {
    let mut need_flush = false;

    while let Some(character) = TERMINAL_BUFFER.pop() {
        let _ = terminal.write_char(character);
        need_flush = true;
    }

    if need_flush {
        terminal.flush();
    }
}

fn terminal_event(terminal: &mut Terminal<Display>) {
    while let Some(scancode) = SCANCODE_QUEUE.pop() {
        if let Some(c) = terminal.handle_keyboard(scancode) {
            let _ = TerminalWriter.write_str(&c);
        }
    }

    while let Some(MouseEvent::Scroll(delta)) = MOUSE_BUFFER.pop() {
        let input = MouseInput::Scroll(delta);
        if let Some(c) = terminal.handle_mouse(input) {
            let _ = TerminalWriter.write_str(&c);
        }
    }
}
