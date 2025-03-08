use anyhow::{Result, anyhow};
use argh::FromArgs;
use builder::ImageBuilder;
use derive_more::FromStr;
use ovmf_prebuilt::{Arch, FileType, Prebuilt, Source};
use std::collections::BTreeMap;
use std::fs::File;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

#[derive(FromArgs)]
#[argh(description = "TrashOS bootloader and kernel builder")]
struct Args {
    #[argh(switch, short = 'b')]
    #[argh(description = "boot the constructed image")]
    boot: bool,

    #[argh(switch, short = 'd')]
    #[argh(description = "dump kernel assembly")]
    dump: bool,

    #[argh(switch, short = 'k')]
    #[argh(description = "use KVM acceleration")]
    kvm: bool,

    #[argh(switch, short = 'w')]
    #[argh(description = "use Hyper-V acceleration")]
    whpx: bool,

    #[argh(option, short = 'c')]
    #[argh(default = "4")]
    #[argh(description = "number of CPU cores")]
    cores: usize,

    #[argh(switch, short = 's')]
    #[argh(description = "redirect serial to stdio")]
    serial: bool,

    #[argh(option, short = 'q')]
    #[argh(default = "StorageDevice::Ahci")]
    #[argh(description = "boot device")]
    storage: StorageDevice,
}

#[derive(FromStr)]
enum StorageDevice {
    Ahci,
    Nvme,
}

fn main() -> Result<()> {
    let img_path = build_img()?;
    let args: Args = argh::from_env();

    if args.dump {
        run_dump()?;
    }

    if args.boot {
        run_qemu(&args, &img_path)?;
    }

    Ok(())
}

fn run_qemu(args: &Args, img_path: &Path) -> Result<()> {
    let mut cmd = Command::new("qemu-system-x86_64");

    cmd.arg("-machine").arg("q35");
    cmd.arg("-m").arg("256m");
    cmd.arg("-smp").arg(format!("cores={}", args.cores));
    cmd.arg("-cpu").arg("qemu64,+x2apic");

    if args.kvm {
        cmd.arg("--enable-kvm");
    }
    if args.whpx {
        cmd.arg("-accel").arg("whpx");
    }
    if args.serial {
        cmd.arg("-serial").arg("stdio");
    }

    if let Some(backend) = match std::env::consts::OS {
        "linux" => Some("pa"),
        "macos" => Some("coreaudio"),
        "windows" => Some("dsound"),
        _ => None,
    } {
        cmd.arg("-audiodev").arg(format!("{},id=sound", backend));
        cmd.arg("-machine").arg("pcspk-audiodev=sound");
        cmd.arg("-device").arg("intel-hda");
        cmd.arg("-device").arg("hda-output,audiodev=sound");
    }

    match args.storage {
        StorageDevice::Ahci => {
            cmd.arg("-device").arg("ahci,id=ahci");
            cmd.arg("-device").arg("ide-hd,drive=disk,bus=ahci.0");
        }
        StorageDevice::Nvme => {
            cmd.arg("-device").arg("nvme,drive=disk,serial=deadbeef");
        }
    }

    let param = "if=none,format=raw,id=disk";
    cmd.args(["-drive", &format!("{param},file={}", img_path.display())]);

    let param = "if=pflash,format=raw";
    cmd.args(["-drive", &format!("{param},file={}", get_ovmf().display())]);

    cmd.spawn()?.wait()?;
    Ok(())
}

fn run_dump() -> Result<()> {
    let file = File::create("TrashOS.txt")?;
    let mut cmd = Command::new("objdump");
    cmd.arg("-d").arg(env!("CARGO_BIN_FILE_KERNEL"));
    cmd.stdout(Stdio::from(file)).spawn()?.wait()?;
    Ok(())
}

fn get_ovmf() -> PathBuf {
    Prebuilt::fetch(Source::LATEST, "target/ovmf")
        .expect("failed to update prebuilt")
        .get_file(Arch::X64, FileType::Code)
}

fn build_img() -> Result<PathBuf> {
    let kernel_path = Path::new(env!("CARGO_BIN_FILE_KERNEL"));
    println!("Building UEFI disk image for kernel at {:#?}", &kernel_path);

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
    ImageBuilder::build(files, &img_path).expect("Failed to build UEFI disk image");
    println!("Created bootable UEFI disk image at {:#?}", &img_path);

    Ok(img_path)
}
