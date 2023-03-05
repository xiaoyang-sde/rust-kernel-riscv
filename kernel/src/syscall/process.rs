//! The `process` module provides system calls to interact with processes.

use log::info;

use crate::task;

/// Exit the current process with an exit code.
pub fn sys_exit(exit_code: i32) -> ! {
    info!("exited with {}", exit_code);
    task::exit_task();
    panic!("unreachable code in process::sys_exit");
}

pub fn sys_sched_yield() -> isize {
    task::suspend_task();
    0
}
