.PHONY: build

build:
	cargo build
	rust-objcopy \
        --strip-all target/riscv64gc-unknown-none-elf/debug/hello_world \
        -O binary target/riscv64gc-unknown-none-elf/debug/hello_world.bin
	rust-objcopy \
        --strip-all target/riscv64gc-unknown-none-elf/debug/privileged_instruction \
        -O binary target/riscv64gc-unknown-none-elf/debug/privileged_instruction.bin
