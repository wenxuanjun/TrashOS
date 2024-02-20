# TrashOS

A stupidly simple OS written in Rust. Tons of crates are used.

### Building

Before building, you may need to add target `x86_64-unknown-none` to your Rust toolchain:

```bash
$ rustup target add x86_64-unknown-none
```

And then add some components to your Rust toolchain:

```bash
$ rustup component add rust-src
$ rustup component add llvm-tools-preview
```

Then you can run the builder to generate the disk image:

```bash
$ cargo run
```

### Running

Add `--help` to the command line to see the help:

```bash
$ cargo run -- --help
```

For example, to build optimized binary and boot it with KVM:

```bash
$ cargo run --release -- --boot --kvm
```

### TODO

- [x] APIC support
- [x] Preemptive multitasking
- [ ] Memory management
- [ ] Inter process communication
- [ ] MLFQ scheduler
- [ ] Symmetric multiprocessing
- [ ] PCI support
- [ ] A simple shell
- [ ] AHCI support
- [ ] Filesystem support
