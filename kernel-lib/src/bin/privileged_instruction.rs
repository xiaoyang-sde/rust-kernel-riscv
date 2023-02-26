#![no_std]
#![no_main]

#[macro_use]
extern crate kernel_lib;

use core::arch::asm;

#[no_mangle]
fn main() -> i32 {
    println!("execute privileged instruction in U Mode");
    unsafe {
        asm!("sret");
    }
    0
}
