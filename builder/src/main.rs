use std::path::{Path, PathBuf};
use std::{process::exit, process::Command};

fn main() {
    let root_dir = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
    let mut out_path = PathBuf::from(root_dir);
    out_path.push("TrashOS.img");

    let kernel_dir = Path::new(env!("CARGO_BIN_FILE_TRASHOS_TrashOS"));

    bootloader::UefiBoot::new(&kernel_dir)
        .create_disk_image(&out_path)
        .unwrap();
    println!("Created bootable UEFI disk image at {:#?}", &out_path);

    if std::env::args().len() == 1 {
        println!("\x1b[32mCompile finished. Add --boot to boot the image.\x1b[0m");
        println!("\x1b[33mIf you want to redirect the serial to stdio, add --serial-stdio.\x1b[0m");
        println!("\x1b[36mAdditionnally, append --haxm to use HAXM acceleration.\x1b[0m");
        println!("\x1b[35mOr, append --kvm to use KVM acceleration.\x1b[0m");
        exit(0);
    }

    let args: Vec<String> = std::env::args().collect();
    if args.contains(&"--boot".to_string()) {
        let mut cmd = Command::new("qemu-system-x86_64");
        let drive_config = format!("format=raw,file={}", &out_path.display());

        cmd.arg("-bios").arg(ovmf_prebuilt::ovmf_pure_efi());
        cmd.arg("-drive").arg(drive_config);

        if args.contains(&"--haxm".to_string()) {
            cmd.arg("-accel").arg("hax");
        }
        if args.contains(&"--kvm".to_string()) {
            cmd.arg("-accel").arg("kvm");
            cmd.arg("-machine").arg("q35");
            cmd.arg("-cpu").arg("host");
        }
        if args.contains(&"--serial-stdio".to_string()) {
            cmd.arg("-serial").arg("stdio");
        }

        let mut child = cmd.spawn().unwrap();
        child.wait().unwrap();
    } else {
        eprintln!("Unknown argument: {:#?}", args);
        exit(1);
    }
}
