.altmacro

.macro SAVE_REGISTER n
  sd x\n, \n*8(sp)
.endm

.macro LOAD_REGISTER n
    ld x\n, \n*8(sp)
.endm

.section .text
.globl _trap
.globl _restore
.align 2

_trap:
  // Swap sp and sscratch in an atomic operation
  // 1. Save the user stack pointer to sscratch
  // 2. Read the kernel stack pointer from sscratch
  csrrw sp, sscratch, sp

  // Allocate a `TrapContext` on the kernel stack
  addi sp, sp, -34*8

  // Save registers x1 through x31 to the stack
  .set n, 1
  .rept 31
    SAVE_REGISTER %n
    .set n, n + 1
  .endr

  // Save the sstatus register to the stack
  csrr t0, sstatus
  sd t0, 32*8(sp)

  // Save the sepc register to the stack
  csrr t1, sepc
  sd t1, 33*8(sp)

  // Save the user stack pointer on the kernel stack
  csrr t2, sscratch
  sd t2, 2*8(sp)

  // Call trap handler with the `TrapContext` as its argument
  mv a0, sp
  call trap_handler

_restore:
    # Restore the stack pointer from the first argument
    mv sp, a0

    # Restore the value of sstatus from the stack
    ld t0, 32*8(sp)
    csrw sstatus, t0

    # Restore the value of sepc from the stack
    ld t1, 33*8(sp)
    csrw sepc, t1

    # Restore the value of sscratch from the stack
    ld t2, 2*8(sp)
    csrw sscratch, t2

    # Restore registers x1 through x31 from the stack
    .set n, 1
    .rept 31
        LOAD_REGISTER %n
        .set n, n + 1
    .endr

    # Deallocate the `TrapContext`
    addi sp, sp, 34*8

    // Swap sp and sscratch in an atomic operation
    // 1. Save the kernel stack pointer to sscratch
    // 2. Read the user stack pointer from sscratch
    csrrw sp, sscratch, sp
    sret
