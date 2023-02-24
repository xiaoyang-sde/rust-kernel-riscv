.align 3
    .section .data
    .global _bin_num
_bin_num:
    .quad 2
    .quad bin_0_start
    .quad bin_1_start
    .quad bin_1_end

    .section .data
    .global bin_0_start
    .global bin_0_end
bin_0_start:
    .incbin "../kernel-lib/target/riscv64gc-unknown-none-bin/debug/hello_world.bin"
bin_0_end:

    .section .data
    .global bin_1_start
    .global bin_1_end
bin_1_start:
    .incbin "../kernel-lib/target/riscv64gc-unknown-none-bin/debug/privileged_instruction.bin"
bin_1_end:
