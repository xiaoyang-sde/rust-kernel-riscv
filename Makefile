.PHONY: build qemu qemu-gdb clean

build:
	make -C kernel-lib build
	make -C kernel build

qemu: build
	make -C kernel qemu

qemu-gdb: build
	make -C kernel qemu-gdb

clean:
	make -C kernel-lib clean
	make -C kernel clean
