use crate::{
    constant::{KERNEL_STACK_SIZE, MAX_BIN_NUM, USER_STACK_SIZE},
    trap::TrapContext,
};
use core::mem;

#[repr(align(4096))]
#[derive(Copy, Clone)]
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
    pub fn push_context(&self, context: TrapContext) -> *mut TrapContext {
        let context_size = mem::size_of::<TrapContext>();
        let context_pointer = (self.get_stack_pointer() - context_size) as *mut TrapContext;
        unsafe {
            *context_pointer = context;
        }
        context_pointer
    }
}

pub static KERNEL_STACK: [KernelStack; MAX_BIN_NUM] = [KernelStack {
    stack: [0; KERNEL_STACK_SIZE],
}; MAX_BIN_NUM];

/// The `UserStack` struct represents the user stack.
#[repr(align(4096))]
#[derive(Copy, Clone)]
pub struct UserStack {
    stack: [u8; USER_STACK_SIZE],
}

impl UserStack {
    /// Return a pointer to the top of the user stack.
    pub fn get_stack_pointer(&self) -> usize {
        self.stack.as_ptr() as usize + USER_STACK_SIZE
    }
}

pub static USER_STACK: [UserStack; MAX_BIN_NUM] = [UserStack {
    stack: [0; USER_STACK_SIZE],
}; MAX_BIN_NUM];
