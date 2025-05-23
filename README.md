# TrashOS

A stupidly simple OS written in Rust. Tons of crates are used.

### Build

Before building, you may need to add target `x86_64-unknown-none` to your Rust toolchain:

```bash
$ rustup target add x86_64-unknown-none
```

Build the boybox first (release mode is required):

```bash
$ cargo build --package boybox --release
```

Then you build run the builder to generate the disk image:

```bash
$ cargo build
```

The disk image will be located at the root of the project directory.

### Run

Add `--help` to the command line to see the help:

```bash
$ cargo run -- --help
```

For example, to build optimized kernel and boot with KVM enabled and redirect the serial output to the terminal:

```bash
$ cargo run --release -- --kvm --serial
```

### Planned features

- [x] APIC support
- [x] Preemptive multitasking
- [x] Memory management
- [x] Task lifecycle management
- [ ] Inter process communication
- [x] Symmetric multiprocessing
- [x] PCIe support
- [x] VT100 codes supported terminal
- [x] AHCI support
- [ ] Block device abstraction
- [ ] Filesystem support
- [ ] Shell
- [x] NVMe support
- [ ] Brain Fuck Scheduler
- [ ] Enlargable & shrinkable heap
