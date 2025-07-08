use alloc::boxed::Box;
use alloc::string::String;
use core::fmt::Write;
use core::sync::atomic::{AtomicBool, Ordering};
use core::time::Duration;
use crossbeam_queue::ArrayQueue;
use os_terminal::font::BitmapFont;
use os_terminal::{MouseInput, Terminal};
use spin::Lazy;

use super::writer::TerminalWriter;
use crate::drivers::mouse::{MOUSE_BUFFER, MouseEvent};
use crate::drivers::{display::Display, speaker::SPEAKER};
use crate::syscall::r#yield;

const SCANCODE_QUEUE_SIZE: usize = 128;
const TERMINAL_BUFFER_SIZE: usize = 4096;

pub static SCANCODE_QUEUE: Lazy<ArrayQueue<u8>> =
    Lazy::new(|| ArrayQueue::new(SCANCODE_QUEUE_SIZE));
pub static TERMINAL_BUFFER: Lazy<ArrayQueue<String>> =
    Lazy::new(|| ArrayQueue::new(TERMINAL_BUFFER_SIZE));

static NEED_FLUSH: AtomicBool = AtomicBool::new(false);

fn terminal_flush(terminal: &mut Terminal<Display>) {
    while let Some(s) = TERMINAL_BUFFER.pop() {
        let _ = terminal.write_str(&s);
        NEED_FLUSH.store(true, Ordering::Relaxed);
    }

    if NEED_FLUSH.swap(false, Ordering::Relaxed) {
        terminal.flush();
    }
}

fn terminal_event(terminal: &mut Terminal<Display>) {
    while let Some(scancode) = SCANCODE_QUEUE.pop() {
        terminal.handle_keyboard(scancode);
        NEED_FLUSH.store(true, Ordering::Relaxed);
    }

    while let Some(MouseEvent::Scroll(delta)) = MOUSE_BUFFER.pop() {
        terminal.handle_mouse(MouseInput::Scroll(delta));
        NEED_FLUSH.store(true, Ordering::Relaxed);
    }
}

pub fn terminal_thread() {
    let mut terminal = Terminal::new(Display::default());
    terminal.set_auto_flush(false);
    terminal.set_crnl_mapping(true);
    terminal.set_scroll_speed(5);
    terminal.set_font_manager(Box::new(BitmapFont));

    terminal.set_bell_handler(|| SPEAKER.lock().beep(750, Duration::from_millis(100)));
    terminal.set_pty_writer(Box::new(|s: String| TerminalWriter.write_str(&s).unwrap()));

    loop {
        terminal_event(&mut terminal);
        terminal_flush(&mut terminal);
        r#yield();
    }
}
