use crate::trap::context::TrapContext;
use core::mem;

const KERNEL_STACK_SIZE: usize = 4096 * 2;

#[repr(align(4096))]
pub struct KernelStack {
    stack: [u8; KERNEL_STACK_SIZE],
}

impl KernelStack {
    fn get_stack_pointer(&self) -> usize {
        self.stack.as_ptr() as usize + KERNEL_STACK_SIZE
    }

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

#[repr(align(4096))]
pub struct UserStack {
    stack: [u8; USER_STACK_SIZE],
}

impl UserStack {
    pub fn get_stack_pointer(&self) -> usize {
        self.stack.as_ptr() as usize + USER_STACK_SIZE
    }
}

pub static USER_STACK: UserStack = UserStack {
    stack: [0; USER_STACK_SIZE],
};
