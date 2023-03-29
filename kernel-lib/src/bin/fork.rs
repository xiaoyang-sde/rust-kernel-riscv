#![no_std]
#![no_main]

use kernel_lib::{exit, fork, wait};
use log::info;

extern crate kernel_lib;

#[no_mangle]
fn main() -> i32 {
    const MAX_CHILD: i32 = 8;
    for i in 0..MAX_CHILD {
        let pid = fork();
        if pid == 0 {
            info!("the child process {} has been spawned", i);
            exit(0);
        } else {
            info!("forked a child process with PID = {}", pid);
        }
        assert!(pid > 0);
    }

    let mut exit_code: usize = 0;
    for _ in 0..MAX_CHILD {
        wait(&mut exit_code);
    }
    return 0;
}
