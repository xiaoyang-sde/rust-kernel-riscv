#![no_std]
#![no_main]

extern crate kernel_lib;

use kernel_lib::{get_time, sched_yield};
use log::info;

#[no_mangle]
fn main() -> i32 {
    let start_time = get_time();
    while get_time() < start_time + 1000 {
        sched_yield();
    }
    info!("slept for 1 second");
    0
}
