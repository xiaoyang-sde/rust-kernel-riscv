pub mod fs;
pub mod process;

const SYSCALL_WRITE: usize = 64;
const SYSCALL_EXIT: usize = 93;

pub fn syscall(syscall_id: usize, arg0: usize, arg1: usize, arg2: usize) -> isize {
    match syscall_id {
        SYSCALL_WRITE => fs::sys_write(arg0, arg1 as *const u8, arg2),
        SYSCALL_EXIT => process::sys_exit(arg0 as i32),
        _ => panic!("unsupported syscall {}", syscall_id),
    }
}
