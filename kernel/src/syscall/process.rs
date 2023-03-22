//! The `process` module provides system calls to interact with processes.

use log::info;

use crate::executor::TaskAction;

use super::SystemCall;

impl SystemCall<'_> {
    /// Exits the current process with an exit code.
    pub fn sys_exit(&self, exit_code: i32) -> (isize, TaskAction) {
        info!("exited with {}", exit_code);
        (0, TaskAction::Break)
    }

    pub fn sys_sched_yield(&self) -> (isize, TaskAction) {
        (0, TaskAction::Yield)
    }

    pub fn sys_fork(&self) -> (isize, TaskAction) {
        (0, TaskAction::Continue)
    }

    pub fn sys_waitpid(&self, _pid: isize, _exit_code: *mut i32) -> (isize, TaskAction) {
        (0, TaskAction::Continue)
    }

    pub fn sys_exec(&self, _path: *const u8) -> (isize, TaskAction) {
        (0, TaskAction::Continue)
    }
}
