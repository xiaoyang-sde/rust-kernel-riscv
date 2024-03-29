.PHONY: build fmt doc qemu qemu-gdb gdb clean

build:
	cargo build

fmt:
	cargo fmt

doc:
	cargo doc --no-deps --bin kernel --lib

qemu: build
	qemu-system-riscv64 \
    -machine virt \
    -nographic \
    -bios bootloader/opensbi-jump.bin \
    -device loader,file=target/riscv64gc-unknown-none-elf/debug/kernel,addr=0x80200000

qemu-gdb: build
	qemu-system-riscv64 \
    -machine virt \
    -nographic \
    -bios bootloader/opensbi-jump.bin \
    -device loader,file=target/riscv64gc-unknown-none-elf/debug/kernel,addr=0x80200000 \
    -s -S

gdb:
	riscv64-unknown-elf-gdb \
    -ex 'file target/riscv64gc-unknown-none-elf/debug/kernel' \
    -ex 'set arch riscv:rv64' \
    -ex 'target remote localhost:1234'

clean:
	cargo clean
