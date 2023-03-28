//! The `process` module provides system calls to interact with processes.

use alloc::vec::Vec;

use crate::{executor::ControlFlow, mem::translate_string, syscall::SystemCall};

impl SystemCall<'_> {
    /// Exits the current process with an exit code.
    pub fn sys_exit(&self, exit_code: usize) -> (isize, ControlFlow) {
        let tid = self.thread.tid();
        let process = self.thread.process();
        process
            .state()
            .thread_list_mut()
            .retain(|thread| thread.tid() != tid);

        if process.state().thread_list().is_empty() {
            process.exit(exit_code);
        }
        (0, ControlFlow::Exit)
    }

    pub fn sys_sched_yield(&self) -> (isize, ControlFlow) {
        (0, ControlFlow::Yield)
    }

    pub fn sys_fork(&self) -> (isize, ControlFlow) {
        let process = self.thread.process().fork();
        (process.pid() as isize, ControlFlow::Continue)
    }

    pub fn sys_waitpid(&self, _pid: isize, _exit_code: *mut i32) -> (isize, ControlFlow) {
        (0, ControlFlow::Continue)
    }

    pub fn sys_exec(&self, path: *const u8) -> (isize, ControlFlow) {
        let path = translate_string(self.thread.satp(), path);
        self.thread.process().exec(&path, Vec::new());
        (0, ControlFlow::Exit)
    }
}
