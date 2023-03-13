//! The `syscall` module provides system calls for interacting with the operating system.

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

/// Invokes a system call with the given arguments.
pub fn syscall(syscall_id: usize, arg0: usize, arg1: usize, arg2: usize) -> isize {
    match syscall_id {
        SYSCALL_READ => fs::sys_read(arg0, arg1 as *const u8, arg2),
        SYSCALL_WRITE => fs::sys_write(arg0, arg1 as *const u8, arg2),
        SYSCALL_EXIT => process::sys_exit(arg0 as i32),
        SYSCALL_SCHED_YIELD => process::sys_sched_yield(),
        SYSCALL_GET_TIME => timer::sys_get_time(),
        SYSCALL_FORK => process::sys_fork(),
        SYSCALL_EXEC => process::sys_exec(arg0 as *const u8),
        SYSCALL_WAITPID => process::sys_waitpid(arg0 as isize, arg1 as *mut i32),
        _ => panic!("unsupported syscall {}", syscall_id),
    }
}
