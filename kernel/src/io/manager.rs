use alloc::collections::btree_map::BTreeMap;
use alloc::format;
use alloc::string::{String, ToString};
use alloc::sync::Arc;
use alloc::vec::Vec;
use anyhow::{Result, anyhow};
use core::sync::atomic::{AtomicUsize, Ordering};
use derive_more::Display;
use gpt_disk_io::Disk;
use thiserror::Error;
use x86_64::structures::paging::{PageSize, Size4KiB};

use super::block::{BlockDevice, BlockDeviceError};
use super::block::{BlockDeviceWrapper, PartitionBlockDevice};
use crate::{driver::nvme::NvmeBlockDevice, mem::AlignedBuffer};

#[derive(Error, Debug)]
pub enum DeviceManagerError {
    #[error("Device not found")]
    DeviceNotFound,
    #[error("Device name '{0}' already exists")]
    NameAlreadyExists(String),
    #[error("Invalid parent device for operation")]
    InvalidParent,
    #[error("Block device operation failed")]
    DeviceError(#[from] BlockDeviceError),
    #[error("Other error: {0}")]
    Other(String),
    #[error("Memory allocation failed: {0}")]
    AllocationError(String),
}

#[derive(Debug, Display, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct DeviceId(usize);

impl DeviceId {
    fn new() -> Self {
        static NEXT_ID: AtomicUsize = AtomicUsize::new(0);
        DeviceId(NEXT_ID.fetch_add(1, Ordering::Relaxed))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RootDeviceType {
    ScsiLike,
    NvmeNamespace,
}

pub enum DeviceSource {
    ScsiLike(Arc<dyn BlockDevice>),
    NvmeController(Vec<NvmeBlockDevice>),
}

impl DeviceSource {
    pub fn get_prefix(&self) -> &'static str {
        match self {
            DeviceSource::ScsiLike(_) => "sd",
            DeviceSource::NvmeController(_) => "nvme",
        }
    }
}

#[derive(Debug, Clone, Display)]
pub enum DeviceKind {
    #[display("Root device")]
    Root(RootDeviceType),
    #[display("Partition device (parent: {parent})")]
    Partition { parent: DeviceId },
}

#[derive(Clone)]
#[allow(dead_code)]
pub struct DeviceInfo {
    id: DeviceId,
    name: String,
    device: Arc<dyn BlockDevice>,
    kind: DeviceKind,
}

#[derive(Default)]
pub struct DeviceManager {
    devices: BTreeMap<DeviceId, DeviceInfo>,
    names: BTreeMap<String, DeviceId>,
}

impl DeviceManager {
    pub fn register(&mut self, device: DeviceSource) -> Result<()> {
        let prefix = device.get_prefix();

        let find_name = |prefix, mapper: fn(usize) -> String| {
            (0..)
                .map(|i| format!("{}{}", prefix, mapper(i)))
                .find(|name| !self.names.contains_key(name))
                .expect("Failed to find an available name")
        };

        let create_scsi_name = |index| {
            let mut suffix = String::new();
            let mut current = index;

            suffix.push((b'a' + (current % 26) as u8) as char);
            current /= 26;

            while current > 0 {
                current -= 1;
                suffix.insert(0, (b'a' + (current % 26) as u8) as char);
                current /= 26;
            }

            suffix
        };

        match device {
            DeviceSource::ScsiLike(device) => {
                let final_name = find_name(prefix, create_scsi_name);

                self.register_internal(
                    final_name,
                    device,
                    DeviceKind::Root(RootDeviceType::ScsiLike),
                )
            }
            DeviceSource::NvmeController(devices) => {
                let id = find_name(prefix, |index| index.to_string());

                for device in devices {
                    self.register_internal(
                        format!("{}n{}", id, device.namespace.id()),
                        Arc::new(device),
                        DeviceKind::Root(RootDeviceType::NvmeNamespace),
                    )?;
                }

                Ok(())
            }
        }
    }

    fn register_internal(
        &mut self,
        name: String,
        device: Arc<dyn BlockDevice>,
        kind: DeviceKind,
    ) -> Result<()> {
        let id = DeviceId::new();

        let info = DeviceInfo {
            id,
            name: name.clone(),
            device,
            kind: kind.clone(),
        };

        self.devices.insert(id, info);
        self.names.insert(name.clone(), id);
        log::info!("{kind} (name: {}, id: {:?}) registered", name, id);

        if matches!(kind, DeviceKind::Root(_)) {
            self.scan_partitions(id)?;
        }

        Ok(())
    }

    #[allow(dead_code)]
    pub fn unregister(&mut self, id: DeviceId) -> Result<()> {
        let info = self
            .devices
            .get(&id)
            .ok_or(DeviceManagerError::DeviceNotFound)?;

        if matches!(info.kind, DeviceKind::Root { .. }) {
            let partitions = self
                .devices
                .iter()
                .filter(|(_, info)| match &info.kind {
                    DeviceKind::Partition { parent, .. } => *parent == id,
                    _ => false,
                })
                .map(|(part_id, _)| *part_id)
                .collect::<Vec<DeviceId>>();

            for partition in partitions {
                self.unregister(partition)?;
            }
        }

        if let Some(device) = self.devices.remove(&id) {
            self.names.remove(&device.name);
            log::info!("{} (name: {}) unregistered.", device.kind, device.name);
        }

        Ok(())
    }

    pub fn scan_partitions(&mut self, root_id: DeviceId) -> Result<()> {
        let root_info = self
            .devices
            .get(&root_id)
            .ok_or(DeviceManagerError::DeviceNotFound)?
            .clone();

        if !matches!(root_info.kind, DeviceKind::Root { .. }) {
            anyhow::bail!(DeviceManagerError::InvalidParent);
        }

        let mut block_buf = AlignedBuffer::new(512, Size4KiB::SIZE as usize)
            .ok_or(anyhow!("Failed to allocate buffer for GPT scan"))?;
        let mut disk = Disk::new(BlockDeviceWrapper(root_info.device.clone()))?;

        let primary_header = disk.read_primary_gpt_header(&mut block_buf)?;
        let layout = primary_header.get_partition_entry_array_layout()?;

        let is_nvme = matches!(
            root_info.kind,
            DeviceKind::Root(RootDeviceType::NvmeNamespace)
        );

        for (index, entry) in disk
            .gpt_partition_entry_array_iter(layout, &mut block_buf)?
            .enumerate()
            .filter_map(|(i, e)| e.ok().map(|e| (i, e)))
            .filter(|(_, e)| e.is_used())
        {
            let part_name = format!(
                "{}{}{}",
                root_info.name,
                if is_nvme { "p" } else { "" },
                index + 1
            );

            let partition = PartitionBlockDevice::new(
                root_info.device.clone(),
                entry.starting_lba.to_u64(),
                entry.ending_lba.to_u64() - entry.starting_lba.to_u64() + 1,
            )?;

            self.register_internal(
                part_name,
                Arc::new(partition) as Arc<dyn BlockDevice>,
                DeviceKind::Partition { parent: root_id },
            )?;
        }

        Ok(())
    }
}
