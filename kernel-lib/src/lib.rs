#![no_std]
#![no_main]
#![feature(linkage)]
#![feature(panic_info_message)]

pub mod console;
mod lang_items;
mod logging;
mod syscall;

use syscall::{sys_exit, sys_sched_yield, sys_write, sys_get_time};

#[no_mangle]
#[link_section = ".text.init"]
pub extern "C" fn _start() -> ! {
    clear_bss();
    logging::init();

    exit(main());
    panic!("failed to invoke `exit`")
}

#[linkage = "weak"]
#[no_mangle]
fn main() -> i32 {
    panic!("failed to find the `main` function");
}

/// Initialize the `.bss` section with zeros.
fn clear_bss() {
    // The `bss_start` and `bss_end` symbols are declared in the `src/linker.ld`,
    // which represent the start address and the end address of the `.bss` section.
    // For more details, please refer to the
    // [ld documentation](https://sourceware.org/binutils/docs/ld/Source-Code-Reference.html).
    extern "C" {
        fn bss_start();
        fn bss_end();
    }

    (bss_start as usize..bss_end as usize)
        .for_each(|address| unsafe { (address as *mut u8).write_volatile(0) })
}

pub fn write(fd: usize, buffer: &[u8]) -> isize {
    sys_write(fd, buffer)
}

pub fn exit(exit_code: i32) -> isize {
    sys_exit(exit_code)
}

pub fn sched_yield() -> isize {
    sys_sched_yield()
}

pub fn get_time() -> isize {
    sys_get_time()
}
