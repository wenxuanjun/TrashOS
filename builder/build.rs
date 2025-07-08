use anyhow::{Context, Result, anyhow};
use fatfs::{FileSystem, FormatVolumeOptions, FsOptions, format_volume};
use gpt::GptConfig;
use gpt::disk::LogicalBlockSize;
use gpt::mbr::ProtectiveMBR;
use gpt::partition_types::EFI;
use std::collections::BTreeMap;
use std::fs::{File, OpenOptions};
use std::path::{Path, PathBuf};
use std::{env, fs, io};
use std::{io::Seek, io::SeekFrom};
use tempfile::NamedTempFile;

type Files = BTreeMap<&'static str, PathBuf>;

fn main() -> Result<()> {
    let env_path = env::var("CARGO_BIN_FILE_KERNEL")?;
    let kernel_path = Path::new(&env_path);

    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let assets_dir = manifest_dir.join("assets");

    let mut files = BTreeMap::new();
    files.insert("kernel", kernel_path.to_path_buf());
    files.insert("efi/boot/bootx64.efi", assets_dir.join("BOOTX64.EFI"));
    files.insert("limine.conf", assets_dir.join("limine.conf"));

    let img_path = manifest_dir
        .parent()
        .ok_or_else(|| anyhow!("Failed to get parent directory"))?
        .join("TrashOS.img");
    build_img(files, &img_path).expect("Failed to build UEFI disk image");
    println!("cargo:rustc-env=IMG_PATH={}", img_path.to_str().unwrap());

    Ok(())
}

fn build_img(files: Files, image_path: &Path) -> Result<()> {
    let fat_partition = NamedTempFile::new()?;
    create_fat(&files, fat_partition.path())?;
    create_disk(fat_partition.path(), image_path)?;
    fat_partition.close()?;
    Ok(())
}

fn create_fat(files: &Files, out_path: &Path) -> Result<()> {
    let fat_file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(true)
        .open(out_path)?;

    const ADDITIONAL_SPACE: u64 = 1024 * 96;
    let total_size: u64 = files
        .values()
        .map(|p| fs::metadata(p).map(|m| m.len()))
        .sum::<Result<u64, _>>()
        .context("Failed to read files metadata")?
        + ADDITIONAL_SPACE;
    fat_file.set_len(total_size)?;

    format_volume(&fat_file, FormatVolumeOptions::new())?;
    let filesystem = FileSystem::new(&fat_file, FsOptions::new())
        .context("Failed to open FAT file system of UEFI FAT file")?;

    for (target_path, source) in files {
        let path = Path::new(&target_path);
        let root_dir = filesystem.root_dir();
        let ancestors = path.ancestors().collect::<Vec<_>>();

        for ancestor in ancestors.iter().skip(1).rev().skip(1) {
            root_dir.create_dir(&ancestor.to_string_lossy())?;
        }

        let mut new_file = root_dir.create_file(target_path)?;
        new_file.truncate()?;
        io::copy(&mut fs::File::open(source)?, &mut new_file)?;
    }
    Ok(())
}

fn create_disk(fat_image: &Path, out_path: &Path) -> Result<()> {
    let mut disk = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(true)
        .open(out_path)?;

    let partition_size = fs::metadata(fat_image)?.len();
    let disk_size = partition_size + 1024 * 64;
    disk.set_len(disk_size)?;

    let mbr = ProtectiveMBR::with_lb_size((disk_size / 512) as u32);
    mbr.overwrite_lba0(&mut disk)?;

    let block_size = LogicalBlockSize::Lb512;
    let mut gpt = GptConfig::new()
        .writable(true)
        .logical_block_size(block_size)
        .create_from_device(Box::new(&mut disk), None)
        .context("Failed to create GPT structure in file")?;
    gpt.update_partitions(Default::default())?;

    let part_id = gpt.add_partition("boot", partition_size, EFI, 0, None)?;
    let start_offset = gpt
        .partitions()
        .get(&part_id)
        .context("Failed to open boot partition after creation")?
        .bytes_start(block_size)?;

    gpt.write()?;
    disk.seek(SeekFrom::Start(start_offset))?;
    io::copy(&mut File::open(fat_image)?, &mut disk)?;

    Ok(())
}
