#![no_std]
#![no_main]
#![feature(panic_info_message)]
#![feature(alloc_error_handler)]

extern crate alloc;

#[macro_use]
mod console;
mod constant;
mod file;
mod lang_items;
mod logging;
mod mem;
mod sbi;
mod sync;
mod syscall;
mod task;
mod timer;
mod trap;

use core::arch::global_asm;
use log::info;

global_asm!(include_str!("asm/boot.asm"));
global_asm!(include_str!("asm/linkage.asm"));

#[no_mangle]
pub fn rust_main() -> ! {
    clear_bss();
    logging::init();

    info!("rust-kernel has booted");
    mem::init();

    trap::init();

    timer::enable_timer_interrupt();
    timer::set_trigger();

    task::run_init_task();
    panic!("unreachable code in main")
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
