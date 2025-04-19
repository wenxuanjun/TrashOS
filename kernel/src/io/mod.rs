use alloc::sync::Arc;
use anyhow::Result;
use manager::{DeviceManager, DeviceSource};

pub mod block;
pub mod manager;
// pub mod vfs;

pub fn test_manager() -> Result<()> {
    let mut manager = DeviceManager::default();

    for device in crate::driver::nvme::NVME.iter() {
        manager.register(DeviceSource::NvmeController(device))?;
    }

    for device in crate::driver::ahci::AHCI.iter() {
        manager.register(DeviceSource::ScsiLike(Arc::new(device)))?;
    }

    Ok(())
}
