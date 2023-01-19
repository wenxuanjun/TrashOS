use std::{env, path::Path};
use std::{process::Command, process::exit};

fn main() {
    let kernel = Path::new(env!("CARGO_BIN_FILE_TRASHOS_TrashOS"));
    let root_dir = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
    let out_str = format!("{}/TrashOS.img", &root_dir.display());
    let out_path = Path::new(&out_str);

    bootloader::UefiBoot::new(&kernel).create_disk_image(&out_path).unwrap();
    println!("Created bootable UEFI disk image at {:#?}", &out_path);
    println!("OVMF image at {:#?}", &ovmf_prebuilt::ovmf_pure_efi().display());

    if let Some(arg) = std::env::args().skip(1).next() {
        match arg.as_str() {
            "--boot" => {
                let mut cmd = Command::new("qemu-system-x86_64");
                cmd.arg("-bios").arg(ovmf_prebuilt::ovmf_pure_efi());
                cmd.arg("-drive").arg(format!("format=raw,file={}", &out_path.display()));
                cmd.arg("-serial").arg("stdio");
                let mut child = cmd.spawn().unwrap();
                child.wait().unwrap();
            }
            _ => {
                eprintln!("Unknown argument: {}", arg);
                exit(1);
            }
        }
    }
}