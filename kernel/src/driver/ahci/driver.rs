use alloc::vec::Vec;
use bit_field::BitField;
use x86_64::VirtAddr;

use super::cmd::{CommandHeader, CommandTable, FisRegH2D};
use super::hba::{HbaMemory, HbaPort};
use super::identify::{Identify, IdentifyData};
use crate::mem::DmaManager;

pub const BLOCK_SIZE: usize = 512;
const FIS_TYPE_REG_H2D: u8 = 0x27;
const CMD_READ_DMA_EXT: u8 = 0x25;
const CMD_WRITE_DMA_EXT: u8 = 0x35;
const CMD_IDENTIFY_DEVICE: u8 = 0xEC;

pub struct Ahci {
    pub data: &'static mut [u8],
    pub port: &'static HbaPort,
    pub cmd_list: &'static [CommandHeader],
    pub cmd_table: &'static mut CommandTable,
}

unsafe impl Send for Ahci {}
unsafe impl Sync for Ahci {}

impl Ahci {
    pub fn new(address: VirtAddr) -> Vec<Self> {
        let hba_memory = unsafe { &*address.as_mut_ptr::<HbaMemory>() };

        if !hba_memory.ahci_enabled() {
            return Vec::new();
        }

        (0..hba_memory.support_port_count())
            .filter(|&port_num| hba_memory.port_active(port_num))
            .flat_map(|port_num| hba_memory.get_port(port_num))
            .map(|port| unsafe { port.init_ahci() })
            .collect()
    }

    pub fn identity(&mut self) -> IdentifyData {
        unsafe {
            self.execute_command(CMD_IDENTIFY_DEVICE, 0);
            (&*(self.data.as_ptr() as *const Identify)).into()
        }
    }

    pub fn read_block(&mut self, start_sector: u64, buffer: &mut [u8]) {
        unsafe { self.execute_command(CMD_READ_DMA_EXT, start_sector) }
        let length = buffer.len().min(BLOCK_SIZE);
        buffer.copy_from_slice(&self.data[..length]);
    }

    pub fn write_block(&mut self, start_sector: u64, buffer: &[u8]) {
        let length = buffer.len().min(BLOCK_SIZE);
        self.data[..length].copy_from_slice(&buffer[..length]);
        unsafe { self.execute_command(CMD_WRITE_DMA_EXT, start_sector) }
    }

    unsafe fn execute_command(&mut self, command: u8, start_sector: u64) {
        let cmd_table = &mut *self.cmd_table;
        let fis = &mut *(cmd_table.cfis.as_mut_ptr() as *mut FisRegH2D);
        fis.fis_type = FIS_TYPE_REG_H2D;
        fis.cflags = 1 << 7;
        fis.command = command;

        fis.device = match command {
            CMD_READ_DMA_EXT | CMD_WRITE_DMA_EXT => 1 << 6,
            _ => 0,
        };

        fis.sector_count = if command == CMD_IDENTIFY_DEVICE { 0 } else { 1 };
        fis.set_lba(start_sector);

        self.port.command_issue.set(1 << 0);
        while self.port.command_issue.get().get_bit(0) {}
    }
}

impl Drop for Ahci {
    fn drop(&mut self) {
        DmaManager::deallocate(VirtAddr::from_ptr(self.cmd_list.as_ptr()));
        DmaManager::deallocate(VirtAddr::from_ptr(self.cmd_table));
        DmaManager::deallocate(VirtAddr::from_ptr(self.data.as_ptr()));
    }
}
