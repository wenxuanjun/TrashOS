use crossbeam_queue::ArrayQueue;
use derive_more::Display;
use spin::{Lazy, Mutex};
use thiserror::Error;

use super::event::{MouseButton, MouseEvent};
use super::packet::{MouseAdditionalFlags, MouseFlags, MousePacket};
use super::ports::MousePorts;

const MOUSE_BUFFER_SIZE: usize = 2048;

pub static MOUSE_BUFFER: Lazy<ArrayQueue<MouseEvent>> =
    Lazy::new(|| ArrayQueue::new(MOUSE_BUFFER_SIZE));

pub static MOUSE: Mutex<Mouse> = Mutex::new(Mouse::default());

#[derive(Debug)]
pub enum MouseType {
    Standard,
    OnlyScroll,
    FiveButton,
}

#[derive(Debug, Error)]
pub enum MouseError {
    #[error("No ACK received from mouse")]
    AckNotReceived,
    #[error("Port not ready after multiple retries")]
    PortNotReady,
}

pub type MouseResult<T> = Result<T, MouseError>;

#[derive(Display)]
#[display("{:?}", mouse_type)]
pub struct Mouse {
    ports: MousePorts,
    mouse_type: MouseType,
    packet_index: u8,
    current_packet: MousePacket,
    button_states: [bool; 3],
}

impl Mouse {
    pub const fn default() -> Self {
        Self {
            ports: MousePorts::default(),
            mouse_type: MouseType::Standard,
            packet_index: 0,
            current_packet: MousePacket::default(),
            button_states: [false; 3],
        }
    }

    pub fn init(&mut self) -> MouseResult<()> {
        unsafe {
            self.ports.send(0xf4)?;
            self.mouse_type = self.detect_type()?;
        }
        Ok(())
    }

    unsafe fn detect_type(&mut self) -> MouseResult<MouseType> {
        for rate in [200, 100, 80] {
            self.ports.send(0xf3)?;
            self.ports.send(rate)?;
        }
        self.ports.send(0xf2)?;
        Ok(match self.ports.read()? {
            0x03 => MouseType::OnlyScroll,
            0x04 => MouseType::FiveButton,
            _ => MouseType::Standard,
        })
    }

    pub fn process_packet(&mut self, packet: u8) {
        let modulo = match self.mouse_type {
            MouseType::Standard => 3,
            _ => 4,
        };

        match self.packet_index % modulo {
            0 => {
                if !self.handle_flags(packet) {
                    return;
                }
            }
            1 => self.handle_movement(packet, true),
            2 => self.handle_movement(packet, false),
            3 => self.handle_additional_flags(packet),
            _ => unreachable!(),
        }

        if self.packet_index % modulo == modulo - 1 {
            self.process_state();
        }

        self.packet_index = (self.packet_index + 1) % modulo;
    }

    fn handle_flags(&mut self, packet: u8) -> bool {
        let flags = MouseFlags::from_bits_truncate(packet);
        (flags.contains(MouseFlags::ALWAYS_ONE))
            .then(|| self.current_packet.flags = flags)
            .is_some()
    }

    fn handle_movement(&mut self, packet: u8, is_x: bool) {
        let (overflow, sign) = if is_x {
            (MouseFlags::X_OVERFLOW, MouseFlags::X_SIGN)
        } else {
            (MouseFlags::Y_OVERFLOW, MouseFlags::Y_SIGN)
        };

        if !self.current_packet.flags.contains(overflow) {
            let delta = if self.current_packet.flags.contains(sign) {
                ((packet as u16) | 0xff00) as i16
            } else {
                packet as i16
            };

            if is_x {
                self.current_packet.move_x = delta;
            } else {
                self.current_packet.move_y = delta;
            }
        }
    }

    fn handle_additional_flags(&mut self, packet: u8) {
        self.current_packet.additional_flags = match packet {
            0b0100_0001 => MouseAdditionalFlags::FirstButton,
            0b0111_1111 => MouseAdditionalFlags::SecondButton,
            0b0000_0001 => MouseAdditionalFlags::ScrollUp,
            0b1111_1111 | 0b0000_1111 => MouseAdditionalFlags::ScrollDown,
            _ => MouseAdditionalFlags::None,
        };
    }
}

impl Mouse {
    fn process_state(&mut self) {
        const BUTTON_MAP: [(MouseButton, MouseFlags); 3] = [
            (MouseButton::Left, MouseFlags::LEFT_BUTTON),
            (MouseButton::Right, MouseFlags::RIGHT_BUTTON),
            (MouseButton::Middle, MouseFlags::MIDDLE_BUTTON),
        ];

        for (index, (button, flag)) in BUTTON_MAP.iter().enumerate() {
            let is_pressed = self.current_packet.flags.contains(*flag);
            let state = &mut self.button_states[index];

            if *state != is_pressed {
                let event = if is_pressed {
                    MouseEvent::Pressed(*button)
                } else {
                    MouseEvent::Released(*button)
                };
                MOUSE_BUFFER.force_push(event);
                *state = is_pressed;
            }
        }

        if let Some(delta) = match self.current_packet.additional_flags {
            MouseAdditionalFlags::ScrollUp => Some(-1),
            MouseAdditionalFlags::ScrollDown => Some(1),
            _ => None,
        } {
            MOUSE_BUFFER.force_push(MouseEvent::Scroll(delta));
        }

        if self.current_packet.move_x != 0 || self.current_packet.move_y != 0 {
            MOUSE_BUFFER.force_push(MouseEvent::Moved(
                self.current_packet.move_x,
                self.current_packet.move_y,
            ));
        }
    }
}
