#![no_std]
#![no_main]
#![feature(linkage)]
#![feature(panic_info_message)]

pub mod console;
mod lang_items;
mod syscall;

use syscall::{sys_exit, sys_write};

#[no_mangle]
#[link_section = ".text.boot"]
pub extern "C" fn _start() -> ! {
    clear_bss();
    exit(main());
    panic!("failed to invoke `sys_exit`")
}

#[linkage = "weak"]
#[no_mangle]
fn main() -> i32 {
    panic!("failed to find the `main()` function");
}

/// The `.bss` section in an object file holds uninitialized data.
/// The kernel initializes the data with zeros.
fn clear_bss() {
    // The `sbss` and `ebss` symbols are declared in the `src/linker.ld`,
    // which represent the start address and the end address of the `.bss` section.
    extern "C" {
        fn sbss();
        fn ebss();
    }

    (sbss as usize..ebss as usize)
        .for_each(|address| unsafe { (address as *mut u8).write_volatile(0) })
}

pub fn write(fd: usize, buffer: &[u8]) -> isize {
    sys_write(fd, buffer)
}

pub fn exit(exit_code: i32) -> isize {
    sys_exit(exit_code)
}
