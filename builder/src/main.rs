use argh::FromArgs;
use std::path::{Path, PathBuf};
use std::{process::exit, process::Command};

#[derive(FromArgs)]
#[argh(description = "TrashOS bootloader and kernel builder")]
struct Args {
    #[argh(switch, short = 'b')]
    #[argh(description = "boot the constructed image")]
    boot: bool,

    #[argh(switch, short = 'h')]
    #[argh(description = "use HAXM acceleration")]
    haxm: bool,

    #[argh(switch, short = 'k')]
    #[argh(description = "use KVM acceleration")]
    kvm: bool,

    #[argh(switch, short = 's')]
    #[argh(description = "redirect serial to stdio")]
    serial_stdio: bool,

    #[argh(switch, short = 'x')]
    #[argh(description = "enable x2APIC")]
    x2apic: bool,
}

fn main() {
    let root_dir = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
    let kernel_dir = Path::new(env!("CARGO_BIN_FILE_KERNEL_kernel"));

    let out_path = PathBuf::from(root_dir).join("TrashOS.img");
    bootloader::UefiBoot::new(&kernel_dir)
        .create_disk_image(&out_path)
        .unwrap();
    println!("Created bootable UEFI disk image at {:#?}", &out_path);

    let args: Args = argh::from_env();

    if args.boot {
        let mut cmd = Command::new("qemu-system-x86_64");
        let drive_config = format!("format=raw,file={}", &out_path.display());

        cmd.arg("-bios").arg(ovmf_prebuilt::ovmf_pure_efi());
        cmd.arg("-drive").arg(drive_config);

        if args.haxm {
            cmd.arg("-accel").arg("hax");
        }
        if args.kvm {
            cmd.arg("-accel").arg("kvm");
        }
        if args.serial_stdio {
            cmd.arg("-serial").arg("stdio");
        }
        if args.x2apic {
            if !args.haxm && !args.kvm {
                eprintln!("x2APIC requires HAXM or KVM acceleration.");
                exit(1);
            }
            cmd.arg("-cpu").arg("host,+x2apic");
        }

        let mut child = cmd.spawn().unwrap();
        child.wait().unwrap();
    }
}
