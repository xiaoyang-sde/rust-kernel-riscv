.altmacro

.macro SAVE_REGISTER n
  sd x\n, \n * 8(sp)
.endm

.macro LOAD_REGISTER n
  ld x\n, \n * 8(sp)
.endm

.section .text.trampoline
.globl _trap
.globl _restore
.align 2

_trap:
  # Swap sp and sscratch in an atomic operation
  # 1. Save the user stack pointer to sscratch
  # 2. Read the kernel stack pointer from sscratch
  csrrw sp, sscratch, sp

  # Save register x1
  SAVE_REGISTER 1

  # Save registers x3 through x31 to the stack
  .set n, 3
  .rept 29
    SAVE_REGISTER %n
    .set n, n + 1
  .endr

  # Save the sstatus register to the stack
  csrr t0, sstatus
  sd t0, 32 * 8(sp)

  # Save the sepc register to the stack
  csrr t1, sepc
  sd t1, 33 * 8(sp)

  # Save the user stack pointer on the kernel stack
  csrr t2, sscratch
  sd t2, 2 * 8(sp)

  # Load `trap_handler` to t0
  ld t0, 34 * 8(sp)

  # Load `kernel_satp` to t1
  ld t1, 35 * 8(sp)

  # Load `kernel_sp` to sp
  ld sp, 36 * 8(sp)

  # Switch to the kernel space
  csrw satp, t1
  sfence.vma

  # Jump to `trap_handler`
  jr t0

_restore:
  # Switch to the user space
  csrw satp, a1
  sfence.vma

  # Restore the value of sscratch from the stack
  csrw sscratch, a0
  mv sp, a0

  # Restore the value of sstatus from the stack
  ld t0, 32 * 8(sp)
  csrw sstatus, t0

  # Restore the value of sepc from the stack
  ld t1, 33 * 8(sp)
  csrw sepc, t1

  # Restore registers x1 through x31 from the stack
  LOAD_REGISTER 1

  # Save registers x3 through x31 to the stack
  .set n, 3
  .rept 29
    LOAD_REGISTER %n
    .set n, n + 1
  .endr

  # Restore the user stack
  ld sp, 2 * 8(sp)
  sret
