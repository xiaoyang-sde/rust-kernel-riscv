//! `rust-kernel-riscv` is an open-source project that implements an operating system kernel on RISC-V architecture with Rust programming language. The project draws inspiration from several open-source implementations, such as [xv6-riscv](https://github.com/mit-pdos/xv6-riscv) and [zCore](https://github.com/rcore-os/zCore).
//!
//! - The kernel leverages Rust's asynchronous programming model to schedule threads in both the
//!   kernel and user space, which makes context switching more efficient and eliminates the need of
//!   allocating a separate kernel stack for each user process.
//!
//! - The kernel implements the kernel page-table isolation, which prevents the kernel space and the
//!   user space to share a same page table and mitigates potential Meltdown attacks.

#![no_std]
#![no_main]
#![feature(alloc_error_handler)]
#![feature(panic_info_message)]
#![feature(naked_functions)]

extern crate alloc;

#[macro_use]
mod console;
mod constant;
mod executor;
mod file;
mod lang_items;
mod logging;
mod mem;
mod sbi;
mod sync;
mod syscall;
mod task;
mod timer;

use core::arch::global_asm;

use log::info;

global_asm!(include_str!("asm/boot.asm"));
global_asm!(include_str!("asm/linkage.asm"));

/// Initializes the thread executor and spawns the `INIT_PROCESS`.
#[no_mangle]
pub fn rust_main() {
    clear_bss();
    logging::init();

    info!("rust-kernel has booted");
    mem::init();

    timer::enable_timer_interrupt();
    timer::set_trigger();

    task::init();
    executor::init();
    executor::run_until_complete();

    sbi::shutdown();
}

/// Initializes the `.bss` section with zeros.
fn clear_bss() {
    extern "C" {
        /// The `bss_start` is a symbol declared in the `src/linker.ld`,
        /// which represent the start address of the `.bss` section.
        fn bss_start();
        /// The `bss_end` is a symbol declared in the `src/linker.ld`,
        /// which represent the end address of the `.bss` section.
        fn bss_end();
    }

    (bss_start as usize..bss_end as usize)
        .for_each(|address| unsafe { (address as *mut u8).write_volatile(0) })
}
