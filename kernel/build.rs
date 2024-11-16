fn main() {
    println!("cargo:rustc-link-arg=-T./kernel/linker.ld");
    println!("cargo:rerun-if-changed=linker.ld");
}
