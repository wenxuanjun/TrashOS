use anyhow::Result;
use argh::{FromArgValue, FromArgs};
use ovmf_prebuilt::{Arch, FileType, Prebuilt, Source};
use std::path::Path;
use std::process::Command;

#[derive(FromArgs)]
#[argh(description = "TrashOS kernel builder and runner")]
struct Args {
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

    #[argh(option, short = 'd')]
    #[argh(default = "StorageDevice::Nvme")]
    #[argh(description = "boot device")]
    storage: StorageDevice,
}

#[derive(Debug, Default)]
enum StorageDevice {
    #[default]
    Nvme,
    Ahci,
    Virtio,
}

impl FromArgValue for StorageDevice {
    fn from_arg_value(value: &str) -> Result<Self, String> {
        match value {
            "nvme" => Ok(StorageDevice::Nvme),
            "ahci" => Ok(StorageDevice::Ahci),
            "virtio" => Ok(StorageDevice::Virtio),
            _ => Err(format!("Invalid storage device: {value}")),
        }
    }
}

fn main() -> Result<()> {
    let args: Args = argh::from_env();
    let img_path = Path::new(env!("IMG_PATH"));
    println!("Image path: {img_path:?}");

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

    cmd.arg("-device").arg("qemu-xhci,id=xhci");
    cmd.args(["-device", "usb-kbd", "-device", "usb-mouse"]);

    if let Some(backend) = match std::env::consts::OS {
        "linux" => Some("pa"),
        "macos" => Some("coreaudio"),
        "windows" => Some("dsound"),
        _ => None,
    } {
        cmd.arg("-audiodev").arg(format!("{backend},id=sound"));
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
        StorageDevice::Virtio => {
            cmd.arg("-device").arg("virtio-blk-pci,drive=disk");
        }
    }

    let param = "if=none,format=raw,id=disk";
    cmd.args(["-drive", &format!("{param},file={}", img_path.display())]);

    let param = "if=pflash,format=raw";
    let ovmf_path = Prebuilt::fetch(Source::LATEST, "target/ovmf")
        .expect("failed to update prebuilt")
        .get_file(Arch::X64, FileType::Code);
    cmd.args(["-drive", &format!("{param},file={}", ovmf_path.display())]);

    cmd.spawn()?.wait()?;
    Ok(())
}
