use core::time::Duration;
use spin::Mutex;
use x86_64::instructions::port::Port;

use super::hpet::HPET;

pub static SPEAKER: Mutex<Speaker> = Mutex::new(Speaker::default());

pub struct Speaker {
    channel_2: Port<u8>,
    command_register: Port<u8>,
    speaker_port: Port<u8>,
}

impl Speaker {
    pub const fn default() -> Self {
        Self {
            channel_2: Port::new(0x42),
            command_register: Port::new(0x43),
            speaker_port: Port::new(0x61),
        }
    }

    pub fn beep(&mut self, frequency: u32, duration: Duration) {
        self.play_sound(frequency);
        HPET.busy_wait(duration);
        self.nosound();
    }
}

impl Speaker {
    fn nosound(&mut self) {
        unsafe {
            let tmp = self.speaker_port.read();
            self.speaker_port.write(tmp & 0xFC)
        };
    }

    fn play_sound(&mut self, n_frequency: u32) {
        let div = 1193180 / n_frequency;

        unsafe {
            self.command_register.write(0xb6);
            self.channel_2.write(div as u8);
            self.channel_2.write((div >> 8) as u8);

            let tmp = self.speaker_port.read();

            if tmp != (tmp | 3) {
                self.speaker_port.write(tmp | 3);
            }
        }
    }
}
