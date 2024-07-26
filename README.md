# TrashOS

A stupidly simple OS written in Rust. Tons of crates are used.

### Build

Before building, you may need to add target `x86_64-unknown-none` to your Rust toolchain:

```bash
$ rustup target add x86_64-unknown-none
```

Build the apps first (release mode is required):

```bash
$ cargo build --package apps --release
```

Then you can run the builder to generate the disk image:

```bash
$ cargo run
```

The disk image will be located at the root of the project directory.

### Run

Add `--help` to the command line to see the help:

```bash
$ cargo run -- --help
```

For example, to build optimized kernel and boot with KVM enabled and redirect the serial output to the terminal:

```bash
$ cargo run --release -- --boot --kvm --serial
```

### Planned features

- [x] APIC support
- [x] Preemptive multitasking
- [ ] Memory management
- [ ] Task lifecycle management
- [ ] Inter process communication
- [x] Symmetric multiprocessing
- [ ] PCI support
- [x] VT100 codes supported terminal
- [ ] AHCI support
- [ ] Filesystem support
- [ ] Shell
- [ ] NVMe support
- [ ] MLFQ scheduler
