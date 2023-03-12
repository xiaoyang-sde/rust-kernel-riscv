//! The `trap` module provides functions and data structuresfor handling traps
//! (exceptions and interrupts) in a RISC-V kernel.
//! For more details, please refer to the
//! [RISC-V Supervisor-Level ISA Documentation](https://five-embeddev.com/riscv-isa-manual/latest/supervisor.html).

pub use self::context::TrapContext;
use crate::{
    constant::{TRAMPOLINE, TRAP_CONTEXT},
    syscall,
    task::{self, satp, trap_context},
    timer,
};
use core::arch::{asm, global_asm};
use log::error;
use riscv::register::{
    scause::{self, Exception, Interrupt},
    stval,
    stvec::{self, TrapMode},
};

mod context;

global_asm!(include_str!("trap.asm"));

extern "C" {
    fn _trap();
    fn _restore();
}

#[no_mangle]
fn kernel_trap_handler() -> ! {
    panic!("unsupported trap");
}

/// Initializes the `stvec` register to `Direct` mode
/// with the address of the [kernel_trap_handler] function.
pub fn init() {
    unsafe {
        stvec::write(kernel_trap_handler as usize, TrapMode::Direct);
    }
}

/// Handles traps (exceptions and interrupts) raised from the user mode.
#[no_mangle]
pub extern "C" fn trap_handler() -> ! {
    unsafe {
        stvec::write(kernel_trap_handler as usize, TrapMode::Direct);
    }

    let context = trap_context();
    let scause = scause::read();
    let stval = stval::read();

    match scause.cause() {
        scause::Trap::Exception(Exception::UserEnvCall) => {
            context.set_sepc(context.sepc() + 4);
            context.set_register(
                10,
                syscall::syscall(
                    context.register(17),
                    context.register(10),
                    context.register(11),
                    context.register(12),
                ) as usize,
            );
        }
        scause::Trap::Exception(Exception::StoreFault)
        | scause::Trap::Exception(Exception::StorePageFault)
        | scause::Trap::Exception(Exception::LoadFault)
        | scause::Trap::Exception(Exception::LoadPageFault) => {
            error!("page fault at {:#x}", stval);
            task::exit_task();
        }
        scause::Trap::Exception(Exception::IllegalInstruction) => {
            error!("illegal instruction");
            task::exit_task();
        }
        scause::Trap::Exception(Exception::InstructionMisaligned) => {
            error!("misaligned instruction");
            task::exit_task();
        }
        scause::Trap::Interrupt(Interrupt::SupervisorTimer) => {
            timer::set_trigger();
            task::suspend_task();
        }
        _ => {
            panic!("unsupported trap {:?}", scause.cause())
        }
    }

    trap_return();
}

#[no_mangle]
pub fn trap_return() -> ! {
    unsafe {
        stvec::write(TRAMPOLINE, TrapMode::Direct);
    }

    let restore_offset = _restore as usize - _trap as usize;
    unsafe {
        asm!(
            "fence.i",
            "jr {restore}",
            restore = in(reg) TRAMPOLINE + restore_offset,
            in("a0") TRAP_CONTEXT,
            in("a1") satp(),
            options(noreturn)
        );
    }
}
