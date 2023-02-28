use crate::trap::TrapContext;
use core::mem;

const KERNEL_STACK_SIZE: usize = 4096 * 2;

#[repr(align(4096))]
pub struct KernelStack {
    stack: [u8; KERNEL_STACK_SIZE],
}

/// The `KernelStack` struct represents the kernel stack.
impl KernelStack {
    fn get_stack_pointer(&self) -> usize {
        self.stack.as_ptr() as usize + KERNEL_STACK_SIZE
    }

    /// Push a [TrapContext] to the kernel stack
    /// and return a mutable reference to the [TrapContext].
    pub unsafe fn push_context(&self, context: TrapContext) -> &'static mut TrapContext {
        let context_size = mem::size_of::<TrapContext>();
        let context_pointer = (self.get_stack_pointer() - context_size) as *mut TrapContext;
        *context_pointer = context;
        context_pointer.as_mut().unwrap()
    }
}

pub static KERNEL_STACK: KernelStack = KernelStack {
    stack: [0; KERNEL_STACK_SIZE],
};

const USER_STACK_SIZE: usize = 4096 * 2;

/// The `UserStack` struct represents the user stack.
#[repr(align(4096))]
pub struct UserStack {
    stack: [u8; USER_STACK_SIZE],
}

impl UserStack {
    /// Return a pointer to the top of the user stack.
    pub fn get_stack_pointer(&self) -> usize {
        self.stack.as_ptr() as usize + USER_STACK_SIZE
    }

    /// Check if a pointer is on the user stack
    pub fn is_valid_pointer(&self, pointer: *const u8, length: usize) -> bool {
        let lower_bound = self.stack.as_ptr() as usize;
        let upper_bound = lower_bound + USER_STACK_SIZE;
        lower_bound <= (pointer as usize) && (pointer as usize) + length <= upper_bound
    }
}

pub static USER_STACK: UserStack = UserStack {
    stack: [0; USER_STACK_SIZE],
};
