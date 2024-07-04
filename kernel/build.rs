use std::path::PathBuf;

fn main() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    println!(
        "cargo:rustc-link-arg=-T{}/linker.ld",
        manifest_dir.display()
    );
    println!("cargo:rerun-if-changed=linker.ld");
}
