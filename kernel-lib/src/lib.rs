#![no_std]
#![no_main]
#![feature(linkage)]
#![feature(panic_info_message)]
#![feature(alloc_error_handler)]

pub mod console;
mod constant;
mod heap_allocator;
mod lang_items;
mod logging;
mod syscall;

use syscall::{
    sys_exec,
    sys_exit,
    sys_fork,
    sys_get_time,
    sys_read,
    sys_sched_yield,
    sys_waitpid,
    sys_write,
};

#[no_mangle]
#[link_section = ".text.init"]
pub extern "C" fn _start() -> ! {
    logging::init();
    heap_allocator::init();

    exit(main());
    panic!("failed to invoke `exit`")
}

#[linkage = "weak"]
#[no_mangle]
fn main() -> i32 {
    panic!("failed to find the `main` function");
}

pub fn read(fd: usize, buffer: &mut [u8]) -> isize {
    sys_read(fd, buffer)
}

pub fn write(fd: usize, buffer: &[u8]) -> isize {
    sys_write(fd, buffer)
}

pub fn exit(exit_code: i32) -> isize {
    sys_exit(exit_code)
}

pub fn sched_yield() -> isize {
    sys_sched_yield()
}

pub fn get_time() -> isize {
    sys_get_time()
}

pub fn fork() -> isize {
    sys_fork()
}

pub fn exec(path: &str) -> isize {
    sys_exec(path)
}

pub fn wait(exit_code: &mut i32) -> isize {
    loop {
        match sys_waitpid(-1, exit_code as *mut i32) {
            -2 => {
                sched_yield();
            }
            pid => return pid,
        }
    }
}

pub fn waitpid(pid: usize, exit_code: &mut i32) -> isize {
    loop {
        match sys_waitpid(pid as isize, exit_code as *mut i32) {
            -2 => {
                sched_yield();
            }
            pid => return pid,
        }
    }
}
