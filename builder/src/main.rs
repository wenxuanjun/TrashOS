use argh::FromArgs;
use builder::ImageBuilder;
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

    #[argh(switch, short = 'h')]
    #[argh(description = "use HAXM acceleration")]
    haxm: bool,

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
        let drive_config = format!("format=raw,file={}", &img_path.display());

        let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
        let ovmf_path = manifest_dir.join("OVMF_CODE.fd");
        let ovmf_config = format!("if=pflash,format=raw,file={}", ovmf_path.display());

        cmd.arg("-machine").arg("q35");
        cmd.arg("-drive").arg(ovmf_config);
        cmd.arg("-drive").arg(drive_config);
        cmd.arg("-m").arg("256m");
        cmd.arg("-smp").arg(format!("cores={}", args.cores));
        cmd.arg("-cpu").arg("qemu64,+x2apic");

        if args.kvm {
            cmd.arg("--enable-kvm");
        }
        if args.haxm {
            cmd.arg("-accel").arg("hax");
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
    let img_path = manifest_dir.parent().unwrap().join("TrashOS.img");

    let limine_elf = manifest_dir.join("BOOTX64.EFI");
    let limine_config = manifest_dir.join("limine.cfg");

    ImageBuilder::build(
        kernel_path.to_path_buf(),
        limine_elf,
        limine_config,
        &img_path,
    )
    .expect("Failed to build UEFI disk image");
    println!("Created bootable UEFI disk image at {:#?}", &img_path);

    img_path
}
