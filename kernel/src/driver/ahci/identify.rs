use alloc::string::{String, ToString};
use alloc::vec::Vec;

#[repr(C)]
pub struct Identify {
    _1: [u16; 10],
    pub serial_number: [u8; 20],
    _2: [u16; 3],
    pub firmware_revision: [u8; 8],
    pub model: [u8; 40],
    _3: [u16; 53],
    pub lba48_sectors: u64,
}

#[derive(Debug)]
pub struct StorageInfo {
    pub serial_number: String,
    pub firmware_revision: String,
    pub model: String,
    pub lba48_sectors: u64,
}

impl From<&Identify> for StorageInfo {
    fn from(info: &Identify) -> Self {
        let parse = |input: &[u8]| -> String {
            let str = input
                .chunks(2)
                .flat_map(|c| c.iter().rev().copied())
                .collect::<Vec<_>>();
            String::from_utf8_lossy(&str).trim().to_string()
        };

        Self {
            serial_number: parse(&info.serial_number),
            firmware_revision: parse(&info.firmware_revision),
            model: parse(&info.model),
            lba48_sectors: info.lba48_sectors,
        }
    }
}
