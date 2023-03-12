/*
The linker script determines the layout of the output file,
which moves the `.text.boot` section defined in `src/boot.asm`
to the base address of the kernel.
It defines various symbols that contains the start and the end
address of each section.
*/

OUTPUT_ARCH(riscv)
ENTRY(_start)
BASE_ADDRESS = 0x80200000;

SECTIONS
{
    . = BASE_ADDRESS;
    kernel_start = .;

    text_start = .;
    .text : {
        *(.text.boot)
        . = ALIGN(4k);

        trampoline_start = .;
        *(.text.trampoline)
        . = ALIGN(4k);
        trampoline_end = .;

        *(.text .text.*)
    }
    . = ALIGN(4K);
    text_end = .;

    rodata_start = .;
    .rodata : {
        *(.rodata .rodata.*)
        *(.srodata .srodata.*)
    }
    . = ALIGN(4K);
    rodata_end = .;

    data_start = .;
    .data : {
        *(.data .data.*)
        *(.sdata .sdata.*)
    }
    . = ALIGN(4K);
    data_end = .;

    bss_stack_start = .;
    .bss : {
        *(.bss.stack)
        bss_stack_end = .;

        bss_start = .;
        *(.bss .bss.*)
        *(.sbss .sbss.*)
    }
    . = ALIGN(4K);
    bss_end = .;

    kernel_end = .;

    /DISCARD/ : {
        *(.eh_frame)
    }
}
