  .section .text.boot
  .globl _start
_start:
  la sp, boot_stack_top
  call rust_main

  .section .bss.stack
boot_stack_bottom:
  .space 4096 * 16
  .globl boot_stack_top
boot_stack_top:
