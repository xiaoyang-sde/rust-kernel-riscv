#![no_std]
#![no_main]
#![feature(panic_info_message)]

mod console;
mod lang_items;
mod sbi;

use core::arch::global_asm;

use crate::sbi::shutdown;

global_asm!(include_str!("boot.asm"));

#[no_mangle]
pub fn rust_main() -> ! {
    clear_bss();
    println!("[kernel] hello, world!");
    shutdown();
}

fn clear_bss() {
    extern "C" {
        fn sbss();
        fn ebss();
    }

    (sbss as usize..ebss as usize)
        .for_each(|address| unsafe { (address as *mut u8).write_volatile(0) })
}
