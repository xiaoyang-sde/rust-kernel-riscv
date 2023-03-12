#![no_std]
#![no_main]
#![feature(linkage)]
#![feature(panic_info_message)]

pub mod console;
mod lang_items;
mod logging;
mod syscall;

use syscall::{sys_exit, sys_get_time, sys_sched_yield, sys_write};

#[no_mangle]
#[link_section = ".text.init"]
pub extern "C" fn _start() -> ! {
    logging::init();

    exit(main());
    panic!("failed to invoke `exit`")
}

#[linkage = "weak"]
#[no_mangle]
fn main() -> i32 {
    panic!("failed to find the `main` function");
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
