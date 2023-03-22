# `rust-kernel-riscv`

`rust-kernel-riscv` is an open-source project that implements an operating system kernel on RISC-V architecture with Rust programming language. The project draws inspiration from several open-source implementations, such as [xv6-riscv](https://github.com/mit-pdos/xv6-riscv) and [zCore](https://github.com/rcore-os/zCore).

- The kernel leverages Rust's asynchronous programming model to schedule threads in both the kernel and user space, which makes context switching more efficient and eliminates the need of allocating a separate kernel stack for each user process.

- The kernel implements the kernel page-table isolation, which prevents the kernel space and the user space to share a same page table and mitigates potential Meltdown attacks.

## Build

- Install the `riscv64gc-unknown-none-elf` target and related components:

```console
rustup install nightly

rustup target add riscv64gc-unknown-none-elf
rustup component add llvm-tools-preview
rustup component add rust-src

cargo install cargo-binutils
```

- Install [QEMU](https://www.qemu.org) with a package manager such as Homebrew:

```console
brew install qemu
```

- Build and run the kernel with QEMU:

```console
make qemu
```

## Development Roadmap

- [ ] File system with asynchronous interface
- [ ] Virtio driver
- [ ] TCP/IP stack
- [ ] Linux-compatible system call interface
- [ ] [musl libc-test](https://wiki.musl-libc.org/libc-test.html)
