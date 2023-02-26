use self::context::TrapContext;
use crate::{batch::runtime, syscall};
use core::arch::global_asm;
use log::error;
use riscv::register::{
    scause::{self, Exception}, stvec,
    stvec::TrapMode,
};

pub mod context;

global_asm!(include_str!("trap.asm"));

extern "C" {
    fn _trap();
}

pub fn init() {
    unsafe {
        stvec::write(_trap as usize, TrapMode::Direct);
    }
}

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
            runtime::load_next_bin();
        }
        scause::Trap::Exception(Exception::IllegalInstruction) => {
            error!("illegal instruction");
            runtime::load_next_bin();
        }
        _ => {
            panic!("unsupported trap {:?}", scause.cause())
        }
    }
    context
}
