use anyhow::Context;
use fatfs::Dir;
use flate2::{Compression, GzBuilder};
use std::collections::BTreeMap;
use std::fs::File;
use std::path::{Path, PathBuf};
use std::{fs, io};
use std::{io::Seek, io::SeekFrom};
use tempfile::NamedTempFile;

const KERNEL: &str = "kernel.gz";
const LIMINE_EFI: &str = "efi/boot/bootx64.efi";
const LIMINE_CONFIG: &str = "limine.cfg";

pub struct ImageBuilder;

impl ImageBuilder {
    pub fn build(
        kernel: PathBuf,
        limine_elf: PathBuf,
        limine_config: PathBuf,
        image_path: &Path,
    ) -> anyhow::Result<()> {
        let mut encoder = GzBuilder::new().read(File::open(kernel)?, Compression::best());
        let compressed_kernel = NamedTempFile::new().context("failed to create temp file")?;

        let kernel_path = compressed_kernel.path().to_owned();
        io::copy(&mut encoder, &mut File::create(kernel_path.clone())?).unwrap();

        let mut files = BTreeMap::new();
        files.insert(KERNEL.into(), kernel_path);
        files.insert(LIMINE_EFI, limine_elf);
        files.insert(LIMINE_CONFIG, limine_config);

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
    pub fn create(files: BTreeMap<&str, PathBuf>, out_path: &Path) -> anyhow::Result<()> {
        let fat_file = fs::OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(out_path)
            .unwrap();

        let fat_size = {
            let mut files_size = 0;
            for source in files.values() {
                let len = fs::metadata(source)
                    .with_context(|| {
                        format!("failed to read metadata of file `{}`", source.display())
                    })?
                    .len();
                files_size += len;
            }
            const ADDITIONAL_SPACE: u64 = 1024 * 64;
            files_size + ADDITIONAL_SPACE
        };
        fat_file.set_len(fat_size).unwrap();

        let format_options = fatfs::FormatVolumeOptions::new();
        fatfs::format_volume(&fat_file, format_options).context("Failed to format FAT file")?;
        let filesystem = fatfs::FileSystem::new(&fat_file, fatfs::FsOptions::new())
            .context("Failed to open FAT file system of UEFI FAT file")?;
        let root_dir = filesystem.root_dir();

        Self::add_files(&root_dir, files)
    }

    pub fn add_files(root_dir: &Dir<&File>, files: BTreeMap<&str, PathBuf>) -> anyhow::Result<()> {
        for (target_path_raw, source) in files {
            let target_path = Path::new(target_path_raw);
            let ancestors: Vec<_> = target_path.ancestors().skip(1).collect();

            for ancestor in ancestors.into_iter().rev().skip(1) {
                root_dir
                    .create_dir(&ancestor.display().to_string())
                    .with_context(|| {
                        format!(
                            "failed to create directory `{}` on FAT filesystem",
                            ancestor.display()
                        )
                    })?;
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
    pub fn create(fat_image: &Path, out_gpt_path: &Path) -> anyhow::Result<()> {
        let mut disk = fs::OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(out_gpt_path)
            .with_context(|| {
                format!("failed to create GPT file at `{}`", out_gpt_path.display())
            })?;

        let partition_size: u64 = fs::metadata(fat_image)
            .context("failed to read metadata of fat image")?
            .len();
        let disk_size = partition_size + 1024 * 64;
        disk.set_len(disk_size)
            .context("failed to set GPT image file length")?;

        let mbr = gpt::mbr::ProtectiveMBR::with_lb_size(
            u32::try_from((disk_size / 512) - 1).unwrap_or(0xFF_FF_FF_FF),
        );
        mbr.overwrite_lba0(&mut disk)
            .context("failed to write protective MBR")?;

        let block_size = gpt::disk::LogicalBlockSize::Lb512;
        let mut gpt = gpt::GptConfig::new()
            .writable(true)
            .initialized(false)
            .logical_block_size(block_size)
            .create_from_device(Box::new(&mut disk), None)
            .context("failed to create GPT structure in file")?;
        gpt.update_partitions(Default::default())
            .context("failed to update GPT partitions")?;

        let partition_id = gpt
            .add_partition("boot", partition_size, gpt::partition_types::EFI, 0, None)
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
