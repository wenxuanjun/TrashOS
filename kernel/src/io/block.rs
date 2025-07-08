use alloc::string::String;
use alloc::sync::Arc;
use core::any::Any;
use core::fmt::Debug;
use gpt_disk_io::BlockIo;
use gpt_disk_types::{BlockSize, Lba};
use thiserror::Error;

use crate::drivers::ahci::{self, AhciBlockDevice};
use crate::drivers::nvme::NvmeBlockDevice;

#[derive(Error, Debug)]
pub enum BlockDeviceError {
    #[error("I/O Error: {0}")]
    IoError(String),
    #[error("NVMe specific error: {0}")]
    Nvme(#[from] nvme::Error),
    #[error("Access out of bounds")]
    OutOfBounds,
    #[error("Block device not found")]
    DeviceNotFound,
    #[error("Invalid input argument")]
    InvalidInput,
    #[error("Device naming error: {0}")]
    NamingError(String),
}

pub type BlockDeviceResult<T> = Result<T, BlockDeviceError>;

pub trait BlockDevice: Send + Sync + Any {
    fn block_size(&self) -> usize;
    fn block_count(&self) -> u64;

    fn flush(&self) -> BlockDeviceResult<()>;
    fn read_block(&self, block_id: u64, buffer: &mut [u8]) -> BlockDeviceResult<()>;
    fn write_block(&self, block_id: u64, buffer: &[u8]) -> BlockDeviceResult<()>;
}

impl BlockDevice for AhciBlockDevice {
    fn block_size(&self) -> usize {
        ahci::BLOCK_SIZE
    }

    fn block_count(&self) -> u64 {
        self.identify.block_count
    }

    fn flush(&self) -> BlockDeviceResult<()> {
        Ok(())
    }

    fn read_block(&self, lba: u64, buffer: &mut [u8]) -> BlockDeviceResult<()> {
        self.device.lock().read_block(lba, buffer);
        Ok(())
    }

    fn write_block(&self, lba: u64, buffer: &[u8]) -> BlockDeviceResult<()> {
        self.device.lock().write_block(lba, buffer);
        Ok(())
    }
}

impl BlockDevice for NvmeBlockDevice {
    fn block_size(&self) -> usize {
        self.namespace.block_size() as usize
    }

    fn block_count(&self) -> u64 {
        self.namespace.block_count()
    }

    fn flush(&self) -> BlockDeviceResult<()> {
        let qpair = self
            .qpairs
            .get(&1)
            .ok_or(BlockDeviceError::DeviceNotFound)?;
        qpair.lock().flush().map_err(BlockDeviceError::from)
    }

    fn read_block(&self, lba: u64, buffer: &mut [u8]) -> BlockDeviceResult<()> {
        let qpair = self
            .qpairs
            .get(&1)
            .ok_or(BlockDeviceError::DeviceNotFound)?;
        qpair
            .lock()
            .read(buffer.as_mut_ptr(), buffer.len(), lba)
            .map_err(BlockDeviceError::from)?;
        qpair.lock().flush().map_err(BlockDeviceError::from)
    }

    fn write_block(&self, lba: u64, buffer: &[u8]) -> BlockDeviceResult<()> {
        let qpair = self
            .qpairs
            .get(&1)
            .ok_or(BlockDeviceError::DeviceNotFound)?;
        qpair
            .lock()
            .write(buffer.as_ptr(), buffer.len(), lba)
            .map_err(BlockDeviceError::from)?;
        qpair.lock().flush().map_err(BlockDeviceError::from)
    }
}

pub struct BlockDeviceWrapper<T: BlockDevice + ?Sized>(pub Arc<T>);

impl<T: BlockDevice + ?Sized> BlockIo for BlockDeviceWrapper<T> {
    type Error = BlockDeviceError;

    fn block_size(&self) -> BlockSize {
        match <T as BlockDevice>::block_size(&self.0) {
            512 => BlockSize::BS_512,
            4096 => BlockSize::BS_4096,
            _ => panic!("Unsupported block size"),
        }
    }

    fn num_blocks(&mut self) -> Result<u64, Self::Error> {
        Ok(self.0.block_count())
    }

    fn flush(&mut self) -> Result<(), Self::Error> {
        self.0.flush()
    }

    fn read_blocks(&mut self, lba: Lba, dst: &mut [u8]) -> Result<(), Self::Error> {
        self.0.read_block(lba.0, dst)
    }

    fn write_blocks(&mut self, lba: Lba, src: &[u8]) -> Result<(), Self::Error> {
        self.0.write_block(lba.0, src)
    }
}

pub struct PartitionBlockDevice {
    parent: Arc<dyn BlockDevice>,
    start_block: u64,
    block_count: u64,
}

impl PartitionBlockDevice {
    pub fn new(
        parent: Arc<dyn BlockDevice>,
        start_block: u64,
        block_count: u64,
    ) -> BlockDeviceResult<Self> {
        if start_block + block_count > parent.block_count() {
            return Err(BlockDeviceError::OutOfBounds);
        }
        Ok(PartitionBlockDevice {
            parent,
            start_block,
            block_count,
        })
    }
}

impl BlockDevice for PartitionBlockDevice {
    fn block_size(&self) -> usize {
        self.parent.block_size()
    }

    fn block_count(&self) -> u64 {
        self.block_count
    }

    fn flush(&self) -> BlockDeviceResult<()> {
        self.parent.flush()
    }

    fn read_block(&self, block_id: u64, buffer: &mut [u8]) -> BlockDeviceResult<()> {
        if block_id >= self.block_count {
            return Err(BlockDeviceError::OutOfBounds);
        }
        self.parent.read_block(self.start_block + block_id, buffer)
    }

    fn write_block(&self, block_id: u64, buffer: &[u8]) -> BlockDeviceResult<()> {
        if block_id >= self.block_count {
            return Err(BlockDeviceError::OutOfBounds);
        }
        self.parent.write_block(self.start_block + block_id, buffer)
    }
}
