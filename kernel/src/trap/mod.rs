//! The `trap` module provides functions and data structuresfor handling traps
//! (exceptions and interrupts) in a RISC-V kernel.
//! For more details, please refer to the
//! [RISC-V Supervisor-Level ISA Documentation](https://five-embeddev.com/riscv-isa-manual/latest/supervisor.html).

pub use self::context::TrapContext;
use crate::{task, syscall};
use core::arch::global_asm;
use log::error;
use riscv::register::{
    scause::{self, Exception},
    stvec,
    stvec::TrapMode,
};

mod context;

global_asm!(include_str!("trap.asm"));

extern "C" {
    fn _trap();
}

/// Initialize the `stvec` register to `Direct` mode
/// with the address of the `_trap` function in `trap.asm`.
pub fn init() {
    unsafe {
        stvec::write(_trap as usize, TrapMode::Direct);
    }
}

/// Handle traps (exceptions and interrupts) raised from the user mode.
///
/// It takes a mutable reference to a [TrapContext] struct that contains the
/// trap context (registers and other state) at the time of the trap. It then
/// inspects the cause of the trap, handles it as appropriate (e.g., invoke a syscall),
/// and returns the updated trap context.
#[no_mangle]
pub extern "C" fn trap_handler(context: &mut TrapContext) -> &mut TrapContext {
    let scause = scause::read();

    match scause.cause() {
        scause::Trap::Exception(Exception::UserEnvCall) => {
            context.sepc += 4;
            context.register[10] = syscall::syscall(
                context.register[17],
                context.register[10],
                context.register[11],
                context.register[12],
            ) as usize;
        }
        scause::Trap::Exception(Exception::StoreFault)
        | scause::Trap::Exception(Exception::StorePageFault) => {
            error!("page fault");
            task::execute_next_bin();
        }
        scause::Trap::Exception(Exception::IllegalInstruction) => {
            error!("illegal instruction");
            task::execute_next_bin();
        }
        _ => {
            panic!("unsupported trap {:?}", scause.cause())
        }
    }
    context
}
