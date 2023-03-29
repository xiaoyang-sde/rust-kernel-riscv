#![no_std]
#![no_main]

use kernel_lib::{exec, fork, wait};
use log::info;

extern crate kernel_lib;

#[no_mangle]
fn main() -> i32 {
    if fork() == 0 {
        exec("fork\0");
    } else {
        loop {
            let mut exit_code = 0;
            let pid = wait(&mut exit_code);
            info!(
                "released a zombie process (pid: {}, exit_code: {})",
                pid, exit_code
            );
        }
    }
    0
}
