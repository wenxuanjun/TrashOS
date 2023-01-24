use std::{env, path::Path};
use std::{process::Command, process::exit};

fn main() {
    let kernel = Path::new(env!("CARGO_BIN_FILE_TRASHOS_TrashOS"));
    let root_dir = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
    let out_str = format!("{}/TrashOS.img", &root_dir.display());
    let out_path = Path::new(&out_str);

    bootloader::UefiBoot::new(&kernel).create_disk_image(&out_path).unwrap();
    println!("Created bootable UEFI disk image at {:#?}", &out_path);

    if env::args().len() == 1 {
        println!("\x1b[32mCompile finished. Run with --boot to boot the image.\x1b[0m");
        println!("\x1b[33mIf you want to redirect the serial output to stdio, add --serial-stdio.\x1b[0m");
        println!("\x1b[36mAdditionnally, you can add --haxm to use HAXM acceleration.\x1b[0m");
        exit(0);
    }

    let args: Vec<String> = env::args().collect();
    if args.contains(&"--boot".to_string()) {
        let mut cmd = Command::new("qemu-system-x86_64");
        cmd.arg("-bios").arg(ovmf_prebuilt::ovmf_pure_efi());
        cmd.arg("-drive").arg(format!("format=raw,file={}", &out_path.display()));
        if args.contains(&"--serial-stdio".to_string()) {
            cmd.arg("-serial").arg("stdio");
        }
        if args.contains(&"--haxm".to_string()) {
            cmd.arg("-accel").arg("hax");
        }
        let mut child = cmd.spawn().unwrap();
        child.wait().unwrap();
    } else {
        eprintln!("Unknown argument: {:#?}", args);
        exit(1);
    }
}