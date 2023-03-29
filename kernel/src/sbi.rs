//! The `sbi` module contains functions that invokes the RISC-V interface.
//! SBI is an interface between the Supervisor Execution Environment and the supervisor.
//! It allows the supervisor to execute some privileged operations with the `ecall` instruction.
//! For more details, please refer to the
//! [RISC-V SBI Specification](https://github.com/riscv-non-isa/riscv-sbi-doc/blob/master/riscv-sbi.adoc).

use core::arch::asm;

use log::info;

const CONSOLE_PUTCHAR_EXTENSION: usize = 0x01;
const CONSOLE_GETCHAR_EXTENSION: usize = 0x02;
const SYSTEM_RESET_EXTENSION: usize = 0x53525354;
const TIMER_EXTENSION: usize = 0x54494D45;

#[inline]
fn sbi_call(extension: usize, function: usize, arg0: usize, arg1: usize) -> (isize, isize) {
    let (error, value);
    unsafe {
        asm!(
            "ecall",
            inlateout("a0") arg0 => error,
            inlateout("a1") arg1 => value,
            in("a6") function,
            in("a7") extension,
        )
    }
    (error, value)
}

#[inline]
pub fn set_timer(stime_value: usize) {
    sbi_call(TIMER_EXTENSION, 0, stime_value, 0);
}

/// Write data present in `char` to debug console.
#[inline]
pub fn console_putchar(char: usize) {
    sbi_call(CONSOLE_PUTCHAR_EXTENSION, 0, char, 0);
}

#[inline]
pub fn console_getchar() -> usize {
    let (value, _) = sbi_call(CONSOLE_GETCHAR_EXTENSION, 0, 0, 0);
    value.max(0) as usize
}

/// Put all the harts to shutdown state.
#[inline]
pub fn shutdown() -> ! {
    info!("shutdown");
    sbi_call(SYSTEM_RESET_EXTENSION, 0, 0, 0);
    panic!("failed to shutdown");
}
