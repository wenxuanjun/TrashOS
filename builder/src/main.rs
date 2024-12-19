use argh::FromArgs;
use builder::ImageBuilder;
use ovmf_prebuilt::{Arch, FileType, Prebuilt, Source};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(FromArgs)]
#[argh(description = "TrashOS bootloader and kernel builder")]
struct Args {
    #[argh(switch, short = 'b')]
    #[argh(description = "boot the constructed image")]
    boot: bool,

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
}

fn main() {
    let img_path = build_img();
    let args: Args = argh::from_env();

    if args.boot {
        let mut cmd = Command::new("qemu-system-x86_64");

        let ovmf_path = Prebuilt::fetch(Source::LATEST, "target/ovmf")
            .expect("failed to update prebuilt")
            .get_file(Arch::X64, FileType::Code);
        let ovmf_config = format!("if=pflash,format=raw,file={}", ovmf_path.display());

        cmd.arg("-machine").arg("q35");
        cmd.arg("-drive").arg(ovmf_config);
        cmd.arg("-m").arg("256m");
        cmd.arg("-smp").arg(format!("cores={}", args.cores));
        cmd.arg("-cpu").arg("qemu64,+x2apic");

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

        let drive_config = format!("if=none,format=raw,id=disk1,file={}", img_path.display());
        cmd.arg("-device").arg("ahci,id=ahci");
        cmd.arg("-device").arg("ide-hd,drive=disk1,bus=ahci.0");
        cmd.arg("-drive").arg(drive_config);

        let drive_config = format!("if=none,format=raw,id=disk2,file={}", img_path.display());
        cmd.arg("-device").arg("nvme,drive=disk2,serial=deadbeef");
        cmd.arg("-drive").arg(drive_config);

        if args.kvm {
            cmd.arg("--enable-kvm");
        }
        if args.whpx {
            cmd.arg("-accel").arg("whpx");
        }
        if args.serial {
            cmd.arg("-serial").arg("stdio");
        }

        let mut child = cmd.spawn().unwrap();
        child.wait().unwrap();
    }
}

fn build_img() -> PathBuf {
    let kernel_path = Path::new(env!("CARGO_BIN_FILE_KERNEL_kernel"));
    println!("Building UEFI disk image for kernel at {:#?}", &kernel_path);

    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let assets_dir = manifest_dir.join("assets");

    let files = BTreeMap::from([
        ("kernel", kernel_path.to_path_buf()),
        ("efi/boot/bootx64.efi", assets_dir.join("BOOTX64.EFI")),
        ("limine.conf", assets_dir.join("limine.conf")),
    ]);

    let img_path = manifest_dir.parent().unwrap().join("TrashOS.img");
    ImageBuilder::build(files, &img_path).expect("Failed to build UEFI disk image");
    println!("Created bootable UEFI disk image at {:#?}", &img_path);

    img_path
}
