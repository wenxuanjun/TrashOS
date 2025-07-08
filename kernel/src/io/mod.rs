use alloc::sync::Arc;
use anyhow::Result;
use manager::{DeviceManager, DeviceSource};

use crate::drivers::{ahci, nvme};

pub mod block;
pub mod manager;

pub fn init_manager() -> Result<()> {
    let mut manager = DeviceManager::default();

    for device in nvme::NVME.iter() {
        manager.register(DeviceSource::NvmeController(device))?;
    }

    for device in ahci::AHCI.iter() {
        manager.register(DeviceSource::ScsiLike(Arc::new(device)))?;
    }

    Ok(())
}
