//! The `constant` module defines several parameters for the kernel.

/// The size of the kernel heap, in bytes.
pub const KERNEL_HEAP_SIZE: usize = 4096 * 768;

/// The size of the user stack, in bytes.
pub const USER_STACK_SIZE: usize = 4096 * 2;

/// The size of a page in memory, in bytes.
pub const PAGE_SIZE: usize = 4096;

/// The number of bits needed to represent a page size.
pub const PAGE_SIZE_BIT: usize = 12;

/// The memory limit for the kernel, in bytes.
pub const MEM_LIMIT: usize = 0x81000000;

/// The address of the trampoline page, which is used for switching between user and kernel space.
pub const TRAMPOLINE: usize = usize::MAX - PAGE_SIZE + 1;

/// The base address of the trap context.
pub const TRAP_CONTEXT_BASE: usize = usize::MAX - 256 * PAGE_SIZE + 1;

/// The clock frequency of the system, in Hz.
pub const CLOCK_FREQ: usize = 12500000;
