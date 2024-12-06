use bitflags::bitflags;
use spin::Mutex;
use thiserror::Error;
use x86_64::instructions::port::Port;

pub static MOUSE: Mutex<Mouse> = Mutex::new(Mouse::default());

pub fn init() {
    let mut mouse = MOUSE.lock();

    if let Err(err) = mouse.init() {
        log::error!("Failed to init mouse: {}", err);
        return;
    }

    mouse.set_complete_handler(|state| log::info!("{:?}", state));
    log::debug!("Mouse Type: {:?}", mouse.mouse_type);
    log::info!("Mouse initialized successfully!");
}

bitflags! {
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    struct MouseFlags: u8 {
        const LEFT_BUTTON = 1 << 0;
        const RIGHT_BUTTON = 1 << 1;
        const MIDDLE_BUTTON = 1 << 2;
        const ALWAYS_ONE = 1 << 3;
        const X_SIGN = 1 << 4;
        const Y_SIGN = 1 << 5;
        const X_OVERFLOW = 1 << 6;
        const Y_OVERFLOW = 1 << 7;
    }
}

#[derive(Debug, Copy, Clone)]
enum MouseAdditionalFlags {
    FirstButton,
    SecondButton,
    ScrollUp,
    ScrollDown,
    None,
}

#[derive(Debug, Copy, Clone)]
pub struct MouseState {
    flags: MouseFlags,
    additional_flags: MouseAdditionalFlags,
    move_x: i16,
    move_y: i16,
}

impl MouseState {
    pub const fn default() -> Self {
        MouseState {
            flags: MouseFlags::empty(),
            additional_flags: MouseAdditionalFlags::None,
            move_x: 0,
            move_y: 0,
        }
    }
}

#[derive(Debug)]
enum MouseType {
    Standard,
    OnlyScroll,
    FiveButton,
}

#[derive(Debug, Error)]
pub enum MouseInitError {
    #[error("No ack response from mouse!")]
    AckNotReceived,
    #[error("Tried too many times to wait port to be ready!")]
    PortNotReady,
}

pub type MouseResult<T> = Result<T, MouseInitError>;

#[derive(Debug)]
pub struct MousePorts {
    command_port: Port<u8>,
    data_port: Port<u8>,
}

impl MousePorts {
    pub const fn default() -> Self {
        Self {
            command_port: Port::new(0x64),
            data_port: Port::new(0x60),
        }
    }
}

impl MousePorts {
    const MAX_PORT_WAIT_COUNT: usize = 20_000;

    unsafe fn send_command(&mut self, command: u8) -> MouseResult<()> {
        self.write_command_port(0xd4)?;
        self.write_data_port(command)?;
        if self.read_data_port()? == 0xfa {
            return Ok(());
        }
        Err(MouseInitError::AckNotReceived)
    }

    unsafe fn read_data_port(&mut self) -> MouseResult<u8> {
        self.wait_for_read()?;
        Ok(self.data_port.read())
    }

    unsafe fn write_command_port(&mut self, value: u8) -> MouseResult<()> {
        self.wait_for_write()?;
        self.command_port.write(value);
        Ok(())
    }

    unsafe fn write_data_port(&mut self, value: u8) -> MouseResult<()> {
        self.wait_for_write()?;
        self.data_port.write(value);
        Ok(())
    }

    unsafe fn wait_for_read(&mut self) -> MouseResult<()> {
        for _ in 0..Self::MAX_PORT_WAIT_COUNT {
            if self.command_port.read() & 0x1 == 1 {
                return Ok(());
            }
        }
        Err(MouseInitError::PortNotReady)
    }

    unsafe fn wait_for_write(&mut self) -> MouseResult<()> {
        for _ in 0..Self::MAX_PORT_WAIT_COUNT {
            if self.command_port.read() & 0x2 == 0 {
                return Ok(());
            }
        }
        Err(MouseInitError::PortNotReady)
    }
}

#[derive(Debug)]
pub struct Mouse {
    ports: MousePorts,
    current_packet_index: u16,
    current_state: MouseState,
    mouse_type: MouseType,
    complete_handler: Option<fn(MouseState)>,
}

impl Mouse {
    pub const fn default() -> Self {
        Self {
            ports: MousePorts::default(),
            current_packet_index: 0,
            current_state: MouseState::default(),
            mouse_type: MouseType::Standard,
            complete_handler: None,
        }
    }

    pub fn init(&mut self) -> MouseResult<()> {
        unsafe {
            self.enable_streaming()?;
            self.mouse_type = self.get_mouse_type()?;
        }
        Ok(())
    }

    unsafe fn enable_streaming(&mut self) -> MouseResult<()> {
        self.ports.send_command(0xf4)?;
        Ok(())
    }

    unsafe fn get_mouse_type(&mut self) -> MouseResult<MouseType> {
        for (cmd, value) in [(0xf3, 200), (0xf3, 100), (0xf3, 80)] {
            self.ports.send_command(cmd)?;
            self.ports.send_command(value)?;
        }
        self.ports.send_command(0xf2)?;
        Ok(match self.ports.read_data_port()? {
            0x3 => MouseType::OnlyScroll,
            0x4 => MouseType::FiveButton,
            _ => MouseType::Standard,
        })
    }

    pub fn set_complete_handler(&mut self, handler: fn(MouseState)) {
        self.complete_handler = Some(handler);
    }
}

impl Mouse {
    pub fn process_packet(&mut self, packet: u8) {
        let modulo = match self.mouse_type {
            MouseType::Standard => 3,
            _ => 4,
        };

        match self.current_packet_index % modulo {
            0 => {
                if !self.process_flags(packet) {
                    return;
                }
            }
            1 => self.process_move(packet, true),
            2 => self.process_move(packet, false),
            3 => self.process_additional_flags(packet),
            _ => unreachable!(),
        }

        if self.current_packet_index % modulo == modulo - 1 {
            if let Some(handler) = self.complete_handler {
                handler(self.current_state);
            }
        }

        self.current_packet_index = (self.current_packet_index + 1) % modulo;
    }

    fn process_flags(&mut self, packet: u8) -> bool {
        let flags = MouseFlags::from_bits_truncate(packet);
        if !flags.contains(MouseFlags::ALWAYS_ONE) {
            return false;
        }
        self.current_state.flags = flags;
        true
    }

    fn process_move(&mut self, packet: u8, is_x: bool) {
        let (overflow_flag, sign_flag) = if is_x {
            (MouseFlags::X_OVERFLOW, MouseFlags::X_SIGN)
        } else {
            (MouseFlags::Y_OVERFLOW, MouseFlags::Y_SIGN)
        };

        if !self.current_state.flags.contains(overflow_flag) {
            let value = if self.current_state.flags.contains(sign_flag) {
                ((packet as u16) | 0xff00) as i16
            } else {
                packet as i16
            };

            if is_x {
                self.current_state.move_x = value;
            } else {
                self.current_state.move_y = value;
            }
        }
    }

    fn process_additional_flags(&mut self, packet: u8) {
        self.current_state.additional_flags = match packet {
            0b0100_0001 => MouseAdditionalFlags::FirstButton,
            0b0111_1111 => MouseAdditionalFlags::SecondButton,
            0b0000_0001 => MouseAdditionalFlags::ScrollUp,
            // First is for OnlyScroll and second is for FiveButton
            0b1111_1111 | 0b0000_1111 => MouseAdditionalFlags::ScrollDown,
            _ => MouseAdditionalFlags::None,
        };
    }
}
