use vcell::VolatileCell as Volatile;

#[repr(C)]
pub struct CommandHeader {
    pub flags: u16,
    pub prdt_length: u16,
    pub prd_byte_count: Volatile<u32>,
    pub command_table_base_address: u64,
    pub reserved: [u32; 4],
}

#[repr(C)]
pub struct CommandTable {
    pub cfis: [u8; 64],
    pub acmd: [u8; 16],
    pub reserved: [u8; 48],
    pub prdt: [PrdtEntry; 1],
}

#[repr(C)]
pub struct PrdtEntry {
    pub data_base_address: u64,
    pub reserved: u32,
    pub byte_count_i: u32,
}

#[repr(C)]
pub struct FisRegH2D {
    pub fis_type: u8,
    pub cflags: u8,
    pub command: u8,
    pub feature_lo: u8,
    pub lba_0: u8,
    pub lba_1: u8,
    pub lba_2: u8,
    pub device: u8,
    pub lba_3: u8,
    pub lba_4: u8,
    pub lba_5: u8,
    pub feature_hi: u8,
    pub sector_count: u16,
    pub icc: u8,
    pub control: u8,
    pub _padding: [u8; 4],
}

impl FisRegH2D {
    pub fn set_lba(&mut self, lba: u64) {
        self.lba_0 = lba as u8;
        self.lba_1 = (lba >> 8) as u8;
        self.lba_2 = (lba >> 16) as u8;
        self.lba_3 = (lba >> 24) as u8;
        self.lba_4 = (lba >> 32) as u8;
        self.lba_5 = (lba >> 40) as u8;
    }
}
