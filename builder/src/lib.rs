use anyhow::Context;
use fatfs::Dir;
use gpt::GptConfig;
use gpt::disk::LogicalBlockSize;
use gpt::mbr::ProtectiveMBR;
use gpt::partition_types::EFI;
use std::collections::BTreeMap;
use std::fs::File;
use std::path::{Path, PathBuf};
use std::{fs, io};
use std::{io::Seek, io::SeekFrom};
use tempfile::NamedTempFile;

type Files = BTreeMap<&'static str, PathBuf>;

pub struct ImageBuilder;

impl ImageBuilder {
    pub fn build(files: Files, image_path: &Path) -> anyhow::Result<()> {
        let fat_partition = NamedTempFile::new().context("failed to create temp file")?;

        FatBuilder::create(files, fat_partition.path())
            .context("failed to create FAT filesystem")?;
        DiskCreator::create(fat_partition.path(), image_path)
            .context("failed to create UEFI GPT disk image")?;

        fat_partition
            .close()
            .context("failed to delete FAT partition after disk image creation")?;

        Ok(())
    }
}

struct FatBuilder;

impl FatBuilder {
    pub fn create(files: Files, out_path: &Path) -> anyhow::Result<()> {
        let fat_file = fs::OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(true)
            .open(out_path)
            .with_context(|| format!("failed to write file to `{}`", out_path.display()))?;

        let files_size = files
            .values()
            .map(|source| fs::metadata(source).map(|meta| meta.len()))
            .collect::<Result<Vec<u64>, _>>()
            .with_context(|| "failed to read files metadata")?;

        const ADDITIONAL_SPACE: u64 = 1024 * 128;
        let fat_size = files_size.iter().sum::<u64>() + ADDITIONAL_SPACE;
        fat_file.set_len(fat_size).unwrap();

        let format_options = fatfs::FormatVolumeOptions::new();
        fatfs::format_volume(&fat_file, format_options).context("Failed to format FAT file")?;

        let filesystem = fatfs::FileSystem::new(&fat_file, fatfs::FsOptions::new())
            .context("Failed to open FAT file system of UEFI FAT file")?;

        Self::add_files(files, filesystem.root_dir())
    }

    pub fn add_files(files: Files, root_dir: Dir<&File>) -> anyhow::Result<()> {
        for (target_path_raw, source) in files {
            let target_path = Path::new(&target_path_raw);
            let ancestors = target_path.ancestors().collect::<Vec<_>>();

            for ancestor in ancestors.into_iter().skip(1).rev().skip(1) {
                root_dir.create_dir(&ancestor.to_string_lossy())?;
            }

            let mut new_file = root_dir
                .create_file(target_path_raw)
                .with_context(|| format!("failed to create file at `{}`", target_path.display()))?;

            new_file.truncate().unwrap();
            io::copy(&mut fs::File::open(source)?, &mut new_file)?;
        }

        Ok(())
    }
}

struct DiskCreator;

impl DiskCreator {
    pub fn create(fat_image: &Path, out_path: &Path) -> anyhow::Result<()> {
        let mut disk = fs::OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(true)
            .open(out_path)
            .with_context(|| format!("failed to create GPT file at `{}`", out_path.display()))?;

        let partition_size: u64 = fs::metadata(fat_image)
            .context("failed to read metadata of fat image")?
            .len();
        let disk_size = partition_size + 1024 * 64;
        disk.set_len(disk_size)
            .context("failed to set GPT image file length")?;

        let mbr =
            ProtectiveMBR::with_lb_size(u32::try_from((disk_size / 512) - 1).unwrap_or(0xffffffff));
        mbr.overwrite_lba0(&mut disk)
            .context("failed to write protective MBR")?;

        let block_size = LogicalBlockSize::Lb512;
        let mut gpt = GptConfig::new()
            .writable(true)
            .logical_block_size(block_size)
            .create_from_device(Box::new(&mut disk), None)
            .context("failed to create GPT structure in file")?;
        gpt.update_partitions(Default::default())
            .context("failed to update GPT partitions")?;

        let partition_id = gpt
            .add_partition("boot", partition_size, EFI, 0, None)
            .context("failed to add boot EFI partition")?;
        let partition = gpt
            .partitions()
            .get(&partition_id)
            .context("failed to open boot partition after creation")?;
        let start_offset = partition
            .bytes_start(block_size)
            .context("failed to get start offset of boot partition")?;

        gpt.write().context("failed to write out GPT changes")?;

        disk.seek(SeekFrom::Start(start_offset))
            .context("failed to seek to start offset")?;
        let mut fat_image = File::open(fat_image).context("failed to open FAT image")?;
        io::copy(&mut fat_image, &mut disk).context("failed to copy FAT image to GPT disk")?;

        Ok(())
    }
}
