    .align 3
    .section .data
    .global _bin_count
    .global _bin_address
    .global _bin_name

_bin_count:
    .quad 6

_bin_address:
    .quad bin_0_start
    .quad bin_0_end
    .quad bin_1_start
    .quad bin_1_end
    .quad bin_2_start
    .quad bin_2_end
    .quad bin_3_start
    .quad bin_3_end
    .quad bin_4_start
    .quad bin_4_end
    .quad bin_5_start
    .quad bin_5_end

_bin_name:
    .string "hello_world"
    .string "init"
    .string "page_fault"
    .string "privileged_instruction"
    .string "shell"
    .string "sleep"

    .section .data
    .global bin_0_start
    .global bin_0_end
    .align 3
bin_0_start:
    .incbin "../kernel-lib/target/riscv64gc-unknown-none-elf/debug/hello_world"
bin_0_end:

    .section .data
    .global bin_1_start
    .global bin_1_end
    .align 3
bin_1_start:
    .incbin "../kernel-lib/target/riscv64gc-unknown-none-elf/debug/init"
bin_1_end:

    .section .data
    .global bin_2_start
    .global bin_2_end
    .align 3
bin_2_start:
    .incbin "../kernel-lib/target/riscv64gc-unknown-none-elf/debug/page_fault"
bin_2_end:

    .section .data
    .global bin_3_start
    .global bin_3_end
    .align 3
bin_3_start:
    .incbin "../kernel-lib/target/riscv64gc-unknown-none-elf/debug/privileged_instruction"
bin_3_end:

    .section .data
    .global bin_4_start
    .global bin_4_end
    .align 3
bin_4_start:
    .incbin "../kernel-lib/target/riscv64gc-unknown-none-elf/debug/shell"
bin_4_end:

    .section .data
    .global bin_5_start
    .global bin_5_end
    .align 3
bin_5_start:
    .incbin "../kernel-lib/target/riscv64gc-unknown-none-elf/debug/sleep"
bin_5_end:
