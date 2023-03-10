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

Then you can build the TrashOS by running following command:

```bash
$ cargo bin
```

### Running

You can run the TrashOS by running following command:

```bash
$ cargo bin --boot
```

### TODO

- [x] APIC support
- [ ] Preemptive multitasking
- [ ] Memory management
- [ ] AHCI support
- [ ] Filesystem support 