use time::error::ComponentRange;
use time::{Date, Month, Time};
use time::{OffsetDateTime, PrimitiveDateTime};
use x86_64::instructions::port::Port;

#[derive(Debug)]
pub struct RtcDateTime {
    second: u8,
    minute: u8,
    hour: u8,
    day: u8,
    month: u8,
    year: u8,
}

impl Default for RtcDateTime {
    fn default() -> Self {
        let mut time = Self {
            second: Self::read_rtc(0x00),
            minute: Self::read_rtc(0x02),
            hour: Self::read_rtc(0x04),
            day: Self::read_rtc(0x07),
            month: Self::read_rtc(0x08),
            year: Self::read_rtc(0x09),
        };

        let format = Self::read_rtc(0x0b);
        let is_24_hour = format & (1 << 1) != 0;
        let is_binary = format & (1 << 2) != 0;

        if !is_binary {
            time.second = Self::bcd_to_bin(time.second);
            time.minute = Self::bcd_to_bin(time.minute);
            time.hour = Self::bcd_to_bin(time.hour);
            time.day = Self::bcd_to_bin(time.day);
            time.month = Self::bcd_to_bin(time.month);
            time.year = Self::bcd_to_bin(time.year);
        }

        if !is_24_hour {
            let is_pm = time.hour & 0x80 != 0;
            time.hour = (time.hour & 0x7F) % 12 + 12 * is_pm as u8;
        }

        time
    }
}

impl RtcDateTime {
    fn read_rtc(idx: u8) -> u8 {
        unsafe {
            Port::new(0x70).write(idx);
            Port::new(0x71).read()
        }
    }

    fn bcd_to_bin(bcd: u8) -> u8 {
        let msb = bcd & 0x80;
        let value = bcd & 0x7F;
        ((value / 16 * 10) + (value % 16)) | msb
    }

    pub fn to_datetime(&self) -> Result<OffsetDateTime, ComponentRange> {
        let month = Month::try_from(self.month)?;
        let date = Date::from_calendar_date(2000 + self.year as i32, month, self.day)?;
        let time = Time::from_hms(self.hour, self.minute, self.second)?;
        Ok(PrimitiveDateTime::new(date, time).assume_utc())
    }
}
