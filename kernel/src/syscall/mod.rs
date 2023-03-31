//! The `syscall` module provides system calls for interacting with the operating system.

use crate::{executor::ControlFlow, mem::UserPtr, task::Thread};

mod fs;
mod process;
mod timer;

const SYSCALL_READ: usize = 63;
const SYSCALL_WRITE: usize = 64;
const SYSCALL_EXIT: usize = 93;
const SYSCALL_SCHED_YIELD: usize = 128;
const SYSCALL_GET_TIME: usize = 169;
const SYSCALL_FORK: usize = 220;
const SYSCALL_EXEC: usize = 221;
const SYSCALL_WAITPID: usize = 260;

/// The `SystemCall` struct provides an interface for invoking system calls on a given thread.
pub struct SystemCall<'a> {
    thread: &'a Thread,
}

impl<'a> SystemCall<'a> {
    /// Constructs a new `SystemCall` instance with the given thread.
    pub fn new(thread: &'a Thread) -> Self {
        Self { thread }
    }

    /// Invokes a system call with the given arguments.
    pub async fn execute(&mut self) -> ControlFlow {
        let trap_context = self.thread.state().lock().kernel_trap_context_mut();

        // Increments the program counter to skip the `ecall` instruction
        trap_context.set_user_sepc(trap_context.user_sepc() + 4);

        let system_call_id = trap_context.user_register(17);
        let argument_0 = trap_context.user_register(10);
        let argument_1 = trap_context.user_register(11);
        let argument_2 = trap_context.user_register(12);

        let (exit_code, control_flow) = match system_call_id {
            SYSCALL_READ => {
                self.sys_read(
                    argument_0,
                    UserPtr::new(self.thread.satp(), argument_1),
                    argument_2,
                )
                .await
            }
            SYSCALL_WRITE => self.sys_write(
                argument_0,
                UserPtr::new(self.thread.satp(), argument_1),
                argument_2,
            ),
            SYSCALL_EXIT => self.sys_exit(argument_0),
            SYSCALL_SCHED_YIELD => self.sys_sched_yield(),
            SYSCALL_GET_TIME => self.sys_get_time(),
            SYSCALL_FORK => self.sys_fork(),
            SYSCALL_EXEC => self.sys_exec(UserPtr::new(self.thread.satp(), argument_0)),
            SYSCALL_WAITPID => {
                self.sys_waitpid(
                    argument_0 as isize,
                    UserPtr::new(self.thread.satp(), argument_1),
                )
                .await
            }
            _ => panic!("unsupported syscall {}", system_call_id),
        };

        if control_flow == ControlFlow::Continue || control_flow == ControlFlow::Yield {
            trap_context.set_user_register(10, exit_code as usize);
        }
        control_flow
    }
}
