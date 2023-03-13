#![no_std]
#![no_main]

use log::info;

use kernel_lib::{exec, fork, sched_yield, wait};

extern crate kernel_lib;

#[no_mangle]
fn main() -> i32 {
    if fork() == 0 {
        exec("shell\0");
    } else {
        loop {
            let mut exit_code = 0;
            let pid = wait(&mut exit_code);
            if pid == -1 {
                sched_yield();
                continue;
            }
            info!(
                "released a zombie process (pid: {}, exit_code: {})",
                pid, exit_code
            );
        }
    }
    0
}
