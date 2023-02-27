//! The `context` module provides a `TrapContext` struct that save and restore
//! the context of a thread when an exception or interrupt occurs.

use riscv::register::sstatus::{self, Sstatus};

/// The `TrapContext` struct is used to save and restore the context of a thread
/// when an exception or interrupt occurs. It contains the values of all the
/// general-purpose registers, the `sstatus` register, and the `sepc` register.
#[repr(C)]
pub struct TrapContext {
    /// The values of all the general-purpose registers.
    pub register: [usize; 32],
    /// The value of the `sstatus` register.
    pub sstatus: Sstatus,
    /// The value of the `sepc` register.
    pub sepc: usize,
}

impl TrapContext {
    /// Set the stack pointer value for the context.
    pub fn set_stack_pointer(&mut self, stack_pointer: usize) {
        self.register[2] = stack_pointer;
    }

    /// Initialize a new `TrapContext` with an initial `sepc` and stack pointer value.
    pub fn init_context(sepc: usize, stack_pointer: usize) -> Self {
        let mut context = Self {
            register: [0; 32],
            sstatus: sstatus::read(),
            sepc,
        };
        context.set_stack_pointer(stack_pointer);
        context
    }
}
