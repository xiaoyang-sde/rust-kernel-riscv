pub const KERNEL_STACK_SIZE: usize = 4096 * 2;
pub const KERNEL_HEAP_SIZE: usize = 0x30_0000;
pub const USER_STACK_SIZE: usize = 4096 * 2;

pub const PAGE_SIZE: usize = 4096;
pub const PAGE_SIZE_BIT: usize = 12;
pub const MEM_END: usize = 0x80800000;
pub const TRAMPOLINE: usize = usize::MAX - PAGE_SIZE + 1;
pub const TRAP_CONTEXT: usize = TRAMPOLINE - PAGE_SIZE;

pub const CLOCK_FREQ: usize = 12500000;
