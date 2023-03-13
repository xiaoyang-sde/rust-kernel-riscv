pub const KERNEL_STACK_SIZE: usize = 4096 * 2;
pub const KERNEL_HEAP_SIZE: usize = 4096 * 768;
pub const USER_STACK_SIZE: usize = 4096 * 2;

pub const PAGE_SIZE: usize = 4096;
pub const PAGE_SIZE_BIT: usize = 12;
pub const MEM_LIMIT: usize = 0x80800000;

pub const TRAMPOLINE: usize = usize::MAX - PAGE_SIZE + 1;
pub const TRAP_CONTEXT: usize = usize::MAX - 2 * PAGE_SIZE + 1;

pub const CLOCK_FREQ: usize = 12500000;