use bitflags::bitflags;
use spin::Mutex;
use x86_64::instructions::port::Port;

pub static MOUSE: Mutex<Mouse> = Mutex::new(Mouse::new());

pub fn init() {
    let mut mouse = MOUSE.lock();
    mouse.init().unwrap_or_else(|err| {
        log::error!("Mouse initialization failed: {}", err);
    });
    mouse.set_complete_handler(|mouse_state: MouseState| {
        crate::println!("{:?}", mouse_state);
    });
    log::debug!("Mouse Type: {:?}", mouse.mouse_type);
    log::info!("Mouse initialized successfully!");
}

bitflags! {
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    struct MouseFlags: u8 {
        const LEFT_BUTTON = 0b0000_0001;
        const RIGHT_BUTTON = 0b0000_0010;
        const MIDDLE_BUTTON = 0b0000_0100;
        const ALWAYS_ONE = 0b0000_1000;
        const X_SIGN = 0b0001_0000;
        const Y_SIGN = 0b0010_0000;
        const X_OVERFLOW = 0b0100_0000;
        const Y_OVERFLOW = 0b1000_0000;
    }
}

#[derive(Debug, Copy, Clone)]
enum MouseAdditionalFlags {
    FirstButton = 0b0100_0001,
    SecondButton = 0b0111_1111,
    ScrollUp = 0b0000_0001,
    ScrollDown = 0b0000_1111,
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
    pub const fn new() -> MouseState {
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

#[derive(Debug)]
pub struct Mouse {
    command_port: Port<u8>,
    data_port: Port<u8>,
    current_packet_index: u16,
    current_state: MouseState,
    mouse_type: MouseType,
    complete_handler: Option<fn(MouseState)>,
}

impl Mouse {
    pub const fn new() -> Mouse {
        Mouse {
            command_port: Port::new(0x64),
            data_port: Port::new(0x60),
            current_packet_index: 0,
            current_state: MouseState::new(),
            mouse_type: MouseType::Standard,
            complete_handler: None,
        }
    }

    pub fn init(&mut self) -> Result<(), &'static str> {
        unsafe {
            self.enable_packet_streaming()?
                .enable_scroll_wheel()?
                .enable_additional_button()?;
        }
        Ok(())
    }

    pub fn process_packet(&mut self, packet: u8) {
        let modulo = match self.mouse_type {
            MouseType::Standard => 3,
            _ => 4,
        };
        match self.current_packet_index % modulo {
            0 => {
                let flags = MouseFlags::from_bits_truncate(packet);
                if !flags.contains(MouseFlags::ALWAYS_ONE) {
                    return;
                }
                self.current_state.flags = flags;
            }
            1 => {
                if !self.current_state.flags.contains(MouseFlags::X_OVERFLOW) {
                    self.current_state.move_x = packet as i16;
                    if self.current_state.flags.contains(MouseFlags::X_SIGN) {
                        self.current_state.move_x = ((packet as u16) | 0xff00) as i16;
                    }
                }
            }
            2 => {
                if !self.current_state.flags.contains(MouseFlags::Y_OVERFLOW) {
                    self.current_state.move_y = packet as i16;
                    if self.current_state.flags.contains(MouseFlags::Y_SIGN) {
                        self.current_state.move_y = ((packet as u16) | 0xff00) as i16;
                    }
                }
            }
            3 => {
                self.current_state.additional_flags = match packet {
                    0b0100_0001 => MouseAdditionalFlags::FirstButton,
                    0b0111_1111 => MouseAdditionalFlags::SecondButton,
                    0b0000_0001 => MouseAdditionalFlags::ScrollUp,
                    // First is for OnlyScroll and second is for FiveButton
                    0b1111_1111 => MouseAdditionalFlags::ScrollDown,
                    0b0000_1111 => MouseAdditionalFlags::ScrollDown,
                    _ => MouseAdditionalFlags::None,
                };
            }
            _ => unreachable!(),
        }
        if self.current_packet_index % modulo == modulo - 1 {
            if self.complete_handler.is_some() {
                (self.complete_handler.unwrap())(self.current_state);
            }
        }
        self.current_packet_index += 1;
        self.current_packet_index %= modulo;
    }

    pub fn set_complete_handler(&mut self, handler: fn(MouseState)) {
        self.complete_handler = Some(handler);
    }

    unsafe fn enable_packet_streaming(&mut self) -> Result<&mut Self, &'static str> {
        Ok(self.send_command(0xf4 as u8)?)
    }

    unsafe fn enable_scroll_wheel(&mut self) -> Result<&mut Self, &'static str> {
        self.send_command(0xf3)?.send_command(200)?;
        self.send_command(0xf3)?.send_command(100)?;
        self.send_command(0xf3)?.send_command(80)?;
        self.send_command(0xf2 as u8)?;
        if self.read_data_port()? == 0x3 {
            self.mouse_type = MouseType::OnlyScroll;
        }
        Ok(self)
    }

    unsafe fn enable_additional_button(&mut self) -> Result<&mut Self, &'static str> {
        self.send_command(0xf3)?.send_command(200)?;
        self.send_command(0xf3)?.send_command(200)?;
        self.send_command(0xf3)?.send_command(80)?;
        self.send_command(0xf2 as u8)?;
        if self.read_data_port()? == 0x4 {
            self.mouse_type = MouseType::FiveButton;
        }
        Ok(self)
    }

    unsafe fn send_command(&mut self, command: u8) -> Result<&mut Self, &'static str> {
        self.write_command_port(0xd4)?;
        self.write_data_port(command)?;
        if self.read_data_port()? == 0xfa {
            return Ok(self);
        }
        Err("Did not receive ack response from mouse!")
    }

    unsafe fn read_data_port(&mut self) -> Result<u8, &'static str> {
        self.wait_for_read()?;
        Ok(self.data_port.read())
    }

    unsafe fn write_command_port(&mut self, value: u8) -> Result<(), &'static str> {
        self.wait_for_write()?;
        Ok(self.command_port.write(value))
    }

    unsafe fn write_data_port(&mut self, value: u8) -> Result<(), &'static str> {
        self.wait_for_write()?;
        Ok(self.data_port.write(value))
    }

    unsafe fn wait_for_read(&mut self) -> Result<(), &'static str> {
        for _ in 0..10_000 {
            if self.command_port.read() & 0x1 == 1 {
                return Ok(());
            }
        }
        Err("Tried too many times to wait command port to be ready to read!")
    }

    unsafe fn wait_for_write(&mut self) -> Result<(), &'static str> {
        for _ in 0..10_000 {
            if self.command_port.read() & 0x2 == 0 {
                return Ok(());
            }
        }
        Err("Tried too many times to wait command port to be ready to write!")
    }
}
