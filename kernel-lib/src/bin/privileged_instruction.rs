#![no_std]
#![no_main]

extern crate kernel_lib;

use core::arch::asm;
use log::info;

#[no_mangle]
fn main() -> i32 {
    info!("execute privileged instruction in U Mode");
    unsafe {
        asm!("sret");
    }
    0
}
