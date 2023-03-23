//! The `syscall` module provides system calls for interacting with the operating system.

use crate::{executor::ControlFlow, task::Thread};

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

pub struct SystemCall<'a> {
    thread: &'a Thread,
}

impl<'a> SystemCall<'a> {
    pub fn new(thread: &'a Thread) -> Self {
        Self { thread }
    }

    /// Invokes a system call with the given arguments.
    pub async fn execute(&mut self) -> ControlFlow {
        let trap_context = self.thread.state().kernel_trap_context_mut().unwrap();
        let system_call_id = trap_context.user_register(17);
        let argument_0 = trap_context.user_register(10);
        let argument_1 = trap_context.user_register(11);
        let argument_2 = trap_context.user_register(12);

        let (exit_code, task_action) = match system_call_id {
            SYSCALL_READ => self.sys_read(argument_0, argument_1 as *const u8, argument_2),
            SYSCALL_WRITE => self.sys_write(argument_0, argument_1 as *const u8, argument_2),
            SYSCALL_EXIT => self.sys_exit(argument_0 as i32),
            SYSCALL_SCHED_YIELD => self.sys_sched_yield(),
            SYSCALL_GET_TIME => self.sys_get_time(),
            SYSCALL_FORK => self.sys_fork(),
            SYSCALL_EXEC => self.sys_exec(argument_0 as *const u8),
            SYSCALL_WAITPID => self.sys_waitpid(argument_0 as isize, argument_1 as *mut i32),
            _ => panic!("unsupported syscall {}", system_call_id),
        };

        trap_context.set_user_register(10, exit_code as usize);
        trap_context.set_user_sepc(trap_context.user_sepc() + 4);
        task_action
    }
}
