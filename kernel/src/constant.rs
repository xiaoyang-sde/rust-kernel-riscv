pub const MAX_BIN_NUM: usize = 4;
pub const BIN_BASE_ADDRESS: usize = 0x80400000;
pub const BIN_SIZE_LIMIT: usize = 0x20000;

pub const KERNEL_STACK_SIZE: usize = 4096 * 2;
pub const KERNEL_HEAP_SIZE: usize = 4096 * 2;
pub const USER_STACK_SIZE: usize = 4096 * 2;

pub const PAGE_SIZE: usize = 4096;
pub const PAGE_SIZE_BIT: usize = 12;
pub const MEM_END: usize = 0x80800000;

pub const CLOCK_FREQ: usize = 12500000;
