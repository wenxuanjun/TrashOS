use super::driver::{MouseError, MouseResult};
use x86_64::instructions::port::Port;

const MAX_PORT_WAIT_COUNT: usize = 20_000;

pub struct MousePorts {
    command: Port<u8>,
    data: Port<u8>,
}

impl MousePorts {
    pub const fn default() -> Self {
        Self {
            command: Port::new(0x64),
            data: Port::new(0x60),
        }
    }

    pub unsafe fn send(&mut self, cmd: u8) -> MouseResult<()> {
        self.wait_write()?;
        self.command.write(0xd4);
        self.wait_write()?;
        self.data.write(cmd);
        if self.data.read() != 0xfa {
            return Err(MouseError::AckNotReceived);
        }
        Ok(())
    }

    pub unsafe fn read(&mut self) -> MouseResult<u8> {
        self.wait_read()?;
        Ok(self.data.read())
    }

    unsafe fn wait_read(&mut self) -> MouseResult<()> {
        (0..MAX_PORT_WAIT_COUNT)
            .find(|_| (self.command.read() & 0x01) != 0)
            .map(|_| ())
            .ok_or(MouseError::PortNotReady)
    }

    unsafe fn wait_write(&mut self) -> MouseResult<()> {
        (0..MAX_PORT_WAIT_COUNT)
            .find(|_| (self.command.read() & 0x02) == 0)
            .map(|_| ())
            .ok_or(MouseError::PortNotReady)
    }
}
