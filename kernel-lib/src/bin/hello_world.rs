#![no_std]
#![no_main]

#[macro_use]
extern crate kernel_lib;

#[no_mangle]
fn main() -> i32 {
    println!("hello, world!");
    0
}
