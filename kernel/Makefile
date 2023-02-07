.PHONY: build qemu gdb-client gdb-server

build:
	cargo build --release
	rust-objcopy \
        --strip-all target/riscv64gc-unknown-none-elf/release/kernel \
        -O binary target/riscv64gc-unknown-none-elf/release/kernel.bin

qemu:
	qemu-system-riscv64 \
        -machine virt \
        -nographic \
        -bios ../bootloader/rustsbi-qemu.bin \
        -device loader,file=target/riscv64gc-unknown-none-elf/release/kernel.bin,addr=0x80200000

gdb-server:
	qemu-system-riscv64 \
        -machine virt \
        -nographic \
        -bios ../bootloader/rustsbi-qemu.bin \
        -device loader,file=target/riscv64gc-unknown-none-elf/release/kernel.bin,addr=0x80200000 \
		-s -S

gdb-client:
	gdb-multiarch \
        -ex 'file target/riscv64gc-unknown-none-elf/release/kernel' \
        -ex 'set arch riscv:rv64' \
        -ex 'target remote localhost:1234'
