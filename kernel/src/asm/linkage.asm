    .align 3
    .section .data
    .global _bin_count
    .global _bin_address

_bin_count:
    .quad 3

_bin_address:
    .quad bin_0_start
    .quad bin_0_end
    .quad bin_1_start
    .quad bin_1_end
    .quad bin_2_start
    .quad bin_2_end

    .section .data
    .global bin_0_start
    .global bin_0_end
bin_0_start:
    .incbin "../kernel-lib/target/riscv64gc-unknown-none-elf/debug/hello_world.bin"
bin_0_end:

    .section .data
    .global bin_1_start
    .global bin_1_end
bin_1_start:
    .incbin "../kernel-lib/target/riscv64gc-unknown-none-elf/debug/page_fault.bin"
bin_1_end:

    .section .data
    .global bin_2_start
    .global bin_2_end
bin_2_start:
    .incbin "../kernel-lib/target/riscv64gc-unknown-none-elf/debug/privileged_instruction.bin"
bin_2_end:
