use core::arch::asm;
use core::future::Future;
use core::mem::transmute;
use core::pin::Pin;
use core::task::{Context, Poll};

use alloc::sync::Arc;
use log::error;
use riscv::register::scause::{Exception, Interrupt};
use riscv::register::{scause, stval};

use crate::constant::TRAMPOLINE;
use crate::task::Thread;
use crate::{executor, executor::TrapContext};
use crate::{syscall, timer};

pub enum TaskAction {
    Continue,
    Yield,
    Break,
}

async fn thread_loop(thread: Arc<Thread>) {
    let enter_user_space: fn(&mut TrapContext, usize) = {
        let _enter_user_space_address =
            _enter_user_space as usize - _enter_kernel_space as usize + TRAMPOLINE;
        unsafe { transmute(_enter_user_space_address) }
    };

    loop {
        let trap_context = thread.state().user_trap_context_mut().unwrap();
        enter_user_space(trap_context, thread.satp());

        let scause = scause::read();
        let stval = stval::read();
        let task_action = match scause.cause() {
            scause::Trap::Exception(Exception::UserEnvCall) => {
                syscall::SystemCall::new(&thread).execute().await
            }
            scause::Trap::Exception(Exception::StoreFault)
            | scause::Trap::Exception(Exception::StorePageFault)
            | scause::Trap::Exception(Exception::LoadFault)
            | scause::Trap::Exception(Exception::LoadPageFault) => {
                error!("page fault at {:#x}", stval);
                TaskAction::Break
            }
            scause::Trap::Exception(Exception::IllegalInstruction) => {
                error!("illegal instruction");
                TaskAction::Break
            }
            scause::Trap::Exception(Exception::InstructionMisaligned) => {
                error!("misaligned instruction");
                TaskAction::Break
            }
            scause::Trap::Interrupt(Interrupt::SupervisorTimer) => {
                timer::set_trigger();
                TaskAction::Yield
            }
            _ => {
                panic!("unsupported trap {:?}", scause.cause())
            }
        };

        match task_action {
            TaskAction::Continue => continue,
            TaskAction::Yield => yield_now().await,
            TaskAction::Break => break,
        }
    }
}

pub fn spawn_thread(thread: Arc<Thread>) {
    let (runnable, task) = executor::spawn(thread_loop(thread));
    runnable.schedule();
    task.detach();
}

#[naked]
#[link_section = ".text.trampoline"]
pub unsafe extern "C" fn _enter_kernel_space() {
    asm!(
        ".p2align 2",
        // read the address of `trap_context` from sscratch
        // and store the user stack pointer to sscratch
        "csrrw sp, sscratch, sp",
        // store the registers to `trap_context.user_register`
        "sd zero, 0 * 8(sp)",
        "sd ra, 1 * 8(sp)",
        "sd gp, 3 * 8(sp)",
        "sd tp, 4 * 8(sp)",
        "sd t0, 5 * 8(sp)",
        "sd t1, 6 * 8(sp)",
        "sd t2, 7 * 8(sp)",
        "sd s0, 8 * 8(sp)",
        "sd s1, 9 * 8(sp)",
        "sd a0, 10 * 8(sp)",
        "sd a1, 11 * 8(sp)",
        "sd a2, 12 * 8(sp)",
        "sd a3, 13 * 8(sp)",
        "sd a4, 14 * 8(sp)",
        "sd a5, 15 * 8(sp)",
        "sd a6, 16 * 8(sp)",
        "sd a7, 17 * 8(sp)",
        "sd s2, 18 * 8(sp)",
        "sd s3, 19 * 8(sp)",
        "sd s4, 20 * 8(sp)",
        "sd s5, 21 * 8(sp)",
        "sd s6, 22 * 8(sp)",
        "sd s7, 23 * 8(sp)",
        "sd s8, 24 * 8(sp)",
        "sd s9, 25 * 8(sp)",
        "sd s10, 26 * 8(sp)",
        "sd s11, 27 * 8(sp)",
        "sd t3, 28 * 8(sp)",
        "sd t4, 29 * 8(sp)",
        "sd t5, 30 * 8(sp)",
        "sd t6, 31 * 8(sp)",
        // save sstatus to `trap_context.user_sstatus`
        "csrr t0, sstatus",
        "sd t0, 32 * 8(sp)",
        // save sepc to `trap_context.user_sepc`
        "csrr t1, sepc",
        "sd  t1, 33 * 8(sp)",
        // store the address of `trap_context` to sscratch
        // and read the user stack pointer to t2
        "csrrw t2, sscratch, sp",
        // store the user stack pointer to `trap_context.user_register`
        "sd t2, 2 * 8(sp)",
        // read `trap_context.kernel_satp` to t3
        "ld t3, 35 * 8(sp)",
        // read the stack pointer from `trap_context.kernel_stack`
        "ld sp, 34 * 8(sp)",
        // write the address of the page table of the kernel to satp
        "csrw satp, t3",
        "sfence.vma",
        // read the return address, global pointer, thread pointer from the kernerl stack
        "ld ra, 0 * 8(sp)",
        "ld gp, 1 * 8(sp)",
        "ld tp, 2 * 8(sp)",
        // store the callee-saved registers on the kernel stack
        "ld s0, 3 * 8(sp)",
        "ld s1, 4 * 8(sp)",
        "ld s2, 5 * 8(sp)",
        "ld s3, 6 * 8(sp)",
        "ld s4, 7 * 8(sp)",
        "ld s5, 8 * 8(sp)",
        "ld s6, 9 * 8(sp)",
        "ld s7, 10 * 8(sp)",
        "ld s8, 11 * 8(sp)",
        "ld s9, 12 * 8(sp)",
        "ld s10, 13 * 8(sp)",
        "ld s11, 14 * 8(sp)",
        // deallocate 15 words on the kernel stack
        "addi sp, sp, 15 * 8",
        "jr ra",
        options(noreturn)
    )
}

#[naked]
#[link_section = ".text.trampoline"]
pub unsafe extern "C" fn _enter_user_space(trap_context: &mut TrapContext, user_satp: usize) {
    asm!(
        ".p2align 2",
        // allocate 15 words on the kernel stack
        "addi sp, sp, -15 * 8",
        // store the return address, global pointer, thread pointer on the kernerl stack
        "sd ra, 0 * 8(sp)",
        "sd gp, 1 * 8(sp)",
        "sd tp, 2 * 8(sp)",
        // store the callee-saved registers on the kernel stack
        "sd s0, 3 * 8(sp)",
        "sd s1, 4 * 8(sp)",
        "sd s2, 5 * 8(sp)",
        "sd s3, 6 * 8(sp)",
        "sd s4, 7 * 8(sp)",
        "sd s5, 8 * 8(sp)",
        "sd s6, 9 * 8(sp)",
        "sd s7, 10 * 8(sp)",
        "sd s8, 11 * 8(sp)",
        "sd s9, 12 * 8(sp)",
        "sd s10, 13 * 8(sp)",
        "sd s11, 14 * 8(sp)",
        // write the address of the page table of the process to satp
        // and read the address of the page table of the kernel to a1
        "csrrw a1, satp, a1",
        "sfence.vma",
        // store the stack pointer to `trap_context.kernel_stack`
        // and move the stack pointer to `trap_context`
        "sd sp, 34 * 8(a0)",
        "mv sp, a0",
        // store the address of the page table of the kernel to `trap_context.kernel_satp`
        "sd a1, 35 * 8(sp)",
        // read `trap_context.user_sstatus` to t0
        "ld t0, 32 * 8(sp)",
        "csrw sstatus, t0",
        // read `trap_context.user_sepc` to t1
        "ld t1, 33 * 8(sp)",
        "csrw sepc, t1",
        // read the registers from `trap_context.user_register`
        "ld zero, 0 * 8(sp)",
        "ld ra, 1 * 8(sp)",
        "ld gp, 3 * 8(sp)",
        "ld tp, 4 * 8(sp)",
        "ld t0, 5 * 8(sp)",
        "ld t1, 6 * 8(sp)",
        "ld t2, 7 * 8(sp)",
        "ld s0, 8 * 8(sp)",
        "ld s1, 9 * 8(sp)",
        "ld a0, 10 * 8(sp)",
        "ld a1, 11 * 8(sp)",
        "ld a2, 12 * 8(sp)",
        "ld a3, 13 * 8(sp)",
        "ld a4, 14 * 8(sp)",
        "ld a5, 15 * 8(sp)",
        "ld a6, 16 * 8(sp)",
        "ld a7, 17 * 8(sp)",
        "ld s2, 18 * 8(sp)",
        "ld s3, 19 * 8(sp)",
        "ld s4, 20 * 8(sp)",
        "ld s5, 21 * 8(sp)",
        "ld s6, 22 * 8(sp)",
        "ld s7, 23 * 8(sp)",
        "ld s8, 24 * 8(sp)",
        "ld s9, 25 * 8(sp)",
        "ld s10, 26 * 8(sp)",
        "ld s11, 27 * 8(sp)",
        "ld t3, 28 * 8(sp)",
        "ld t4, 29 * 8(sp)",
        "ld t5, 30 * 8(sp)",
        "ld t6, 31 * 8(sp)",
        // save the address of `trap_context` to sscratch
        "csrw sscratch, sp",
        // read the user stack pointer from `trap_context.user_register`
        "ld sp, 2 * 8(sp)",
        "sret",
        options(noreturn)
    )
}

pub async fn yield_now() {
    YieldFuture::new().await
}

struct YieldFuture {
    state: bool,
}

impl YieldFuture {
    fn new() -> Self {
        YieldFuture { state: false }
    }
}

impl Future for YieldFuture {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        if self.state {
            return Poll::Ready(());
        }
        self.state = true;
        cx.waker().wake_by_ref();
        Poll::Pending
    }
}
