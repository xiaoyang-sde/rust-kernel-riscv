#![no_std]
#![no_main]

extern crate kernel_lib;

use log::info;
use core::arch::asm;

#[no_mangle]
fn main() -> i32 {
    info!("execute privileged instruction in U Mode");
    unsafe {
        asm!("sret");
    }
    0
}
