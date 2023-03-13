//! The `process` module provides system calls to interact with processes.

use log::info;

use crate::task;

/// Exits the current process with an exit code.
pub fn sys_exit(exit_code: i32) -> ! {
    info!("exited with {}", exit_code);
    task::exit_task();
    panic!("unreachable code in process::sys_exit");
}

pub fn sys_sched_yield() -> isize {
    task::suspend_task();
    0
}

pub fn sys_fork() -> isize {
    -1
}

pub fn sys_waitpid(pid: isize, exit_code: *mut i32) -> isize {
    -1
}

pub fn sys_exec(path: *const u8) -> isize {
    -1
}
