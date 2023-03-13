.PHONY: build qemu qemu-gdb clean

build:
	cargo build

qemu:
	qemu-system-riscv64 \
    -machine virt \
    -nographic \
    -bios ../bootloader/opensbi-jump.bin \
    -device loader,file=target/riscv64gc-unknown-none-elf/debug/kernel,addr=0x80200000

qemu-gdb:
	qemu-system-riscv64 \
    -machine virt \
    -nographic \
    -bios ../bootloader/opensbi-jump.bin \
    -device loader,file=target/riscv64gc-unknown-none-elf/debug/kernel,addr=0x80200000 \
    -s -S

clean:
	cargo clean
