#![no_std]
#![no_main]

extern crate kernel_lib;

use core::arch::asm;

use log::warn;

#[no_mangle]
fn main() -> i32 {
    warn!("attempt to execute privileged instruction");
    unsafe {
        asm!("sret");
    }
    0
}
