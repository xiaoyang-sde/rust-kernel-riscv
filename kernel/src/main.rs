#![no_std]
#![no_main]
#![feature(panic_info_message)]

mod console;
mod lang_items;
mod logging;
mod sbi;

use core::arch::global_asm;

use log::info;

global_asm!(include_str!("boot.asm"));

#[no_mangle]
pub fn rust_main() -> ! {
    clear_bss();
    logging::init();

    info!("hello, world!");

    sbi::shutdown();
}

/// The `.bss` section in an object file holds uninitialized data.
/// The kernel initializes the data with zeros.
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
