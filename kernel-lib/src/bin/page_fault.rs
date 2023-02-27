#![no_std]
#![no_main]

extern crate kernel_lib;

use core::ptr::null_mut;

use log::warn;

#[no_mangle]
fn main() -> i32 {
    warn!("attempt to write to an invalid location");
    unsafe {
        null_mut::<u8>().write_volatile(0);
    }
    0
}
