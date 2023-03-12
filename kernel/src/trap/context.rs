//! The `context` module provides a `TrapContext` struct that save and restore
//! the context of a thread when an exception or interrupt occurs.

use riscv::register::sstatus::{self, Sstatus};

/// The `TrapContext` struct is used to save and restore the context of a thread
/// when an exception or interrupt occurs. It contains the values of all the
/// general-purpose registers, the `sstatus` register, and the `sepc` register.
#[repr(C)]
pub struct TrapContext {
    /// The values of all the general-purpose registers.
    register: [usize; 32],
    /// The value of the `sstatus` register.
    sstatus: Sstatus,
    /// The value of the `sepc` register.
    sepc: usize,
    /// The address of the `trap_handler` function.
    trap_handler: usize,
    /// The value of the `satp` register of the kernel.
    kernel_satp: usize,
    /// The value of the `sp` register of the kernel.
    kernel_sp: usize,
}

impl TrapContext {
    /// Initializes a new `TrapContext` with an initial `sepc` and stack pointer value.
    pub fn init_context(
        sepc: usize,
        stack_pointer: usize,
        trap_handler: usize,
        kernel_satp: usize,
        kernel_sp: usize,
    ) -> Self {
        let mut context = Self {
            register: [0; 32],
            sstatus: sstatus::read(),
            sepc,
            trap_handler,
            kernel_satp,
            kernel_sp,
        };

        context.register[2] = stack_pointer;
        context
    }

    pub fn sepc(&self) -> usize {
        self.sepc
    }

    pub fn set_sepc(&mut self, sepc: usize) {
        self.sepc = sepc;
    }

    pub fn register(&self, index: usize) -> usize {
        self.register[index]
    }

    pub fn set_register(&mut self, index: usize, value: usize) {
        self.register[index] = value;
    }
}
