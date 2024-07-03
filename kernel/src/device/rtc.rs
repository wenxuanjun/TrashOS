use time::{error::ComponentRange, Time};
use time::{Date, Month, OffsetDateTime, PrimitiveDateTime};
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

impl RtcDateTime {
    pub fn new() -> Self {
        let mut rtc_time = Self {
            second: Self::get_rtc_register(0x00),
            minute: Self::get_rtc_register(0x02),
            hour: Self::get_rtc_register(0x04),
            day: Self::get_rtc_register(0x07),
            month: Self::get_rtc_register(0x08),
            year: Self::get_rtc_register(0x09),
        };

        let format = Self::get_rtc_register(0x0b);
        let is_24_hour = format & (1 << 1) != 0;
        let is_binary = format & (1 << 2) != 0;

        if !is_binary {
            rtc_time.second = Self::rtc_bcd_to_bin(rtc_time.second);
            rtc_time.minute = Self::rtc_bcd_to_bin(rtc_time.minute);
            rtc_time.hour = Self::rtc_bcd_to_bin(rtc_time.hour);
            rtc_time.day = Self::rtc_bcd_to_bin(rtc_time.day);
            rtc_time.month = Self::rtc_bcd_to_bin(rtc_time.month);
            rtc_time.year = Self::rtc_bcd_to_bin(rtc_time.year);
        }

        if !is_24_hour {
            let is_pm = (rtc_time.hour & 0x80) != 0;
            rtc_time.hour = ((rtc_time.hour & !0x80) % 12) + 12 * is_pm as u8;
        }

        rtc_time
    }

    fn get_rtc_register(idx: u8) -> u8 {
        unsafe {
            Port::new(0x70).write(idx);
            Port::new(0x71).read()
        }
    }

    fn rtc_bcd_to_bin(bcd: u8) -> u8 {
        let msb = bcd & 0x80;
        let msb_masked = bcd & !0x80;
        (((msb_masked / 16) * 10) + (msb_masked % 16)) | msb
    }

    pub fn to_datetime(&self) -> Result<OffsetDateTime, ComponentRange> {
        let month = Month::try_from(self.month)?;
        let date = Date::from_calendar_date(self.year as i32 + 2000, month, self.day)?;
        let time = Time::from_hms(self.hour, self.minute, self.second)?;

        Ok(PrimitiveDateTime::new(date, time).assume_utc())
    }
}
