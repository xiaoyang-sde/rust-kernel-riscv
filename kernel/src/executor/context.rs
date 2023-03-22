//! The `context` module provides a `TrapContext` struct that save and restore
//! the context of a thread when an exception or interrupt occurs.

use riscv::register::sstatus::Sstatus;

/// The `TrapContext` struct is used to save and restore the context of a thread when an exception
/// or interrupt occurs. It contains the values of all the general-purpose registers of the thread,
/// the `sstatus` register, the `sepc` register, the address of the kernel stack, and the `satp`
/// register value that refers to the kernel page table.
#[repr(C)]
pub struct TrapContext {
    user_register: [usize; 32],
    user_sstatus: Sstatus,
    user_sepc: usize,

    kernel_stack: usize,
    kernel_satp: usize,
}

impl TrapContext {
    pub fn user_status(&self) -> Sstatus {
        self.user_sstatus
    }

    pub fn set_user_status(&mut self, user_sstatus: Sstatus) {
        self.user_sstatus = user_sstatus;
    }

    pub fn user_sepc(&self) -> usize {
        self.user_sepc
    }

    pub fn set_user_sepc(&mut self, user_sepc: usize) {
        self.user_sepc = user_sepc;
    }

    pub fn user_register(&self, index: usize) -> usize {
        self.user_register[index]
    }

    pub fn set_user_register(&mut self, index: usize, value: usize) {
        self.user_register[index] = value;
    }
}
