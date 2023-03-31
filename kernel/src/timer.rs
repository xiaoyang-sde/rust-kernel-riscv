//! The `timer` module provides functions to configure the timer interrupt.

use riscv::register::{sie, time};

use crate::{constant::CLOCK_FREQ, sbi};

/// The number of timer ticks per second.
const TICK_PER_SEC: usize = 100;

/// Returns the current system time in milliseconds.
pub fn get_time() -> usize {
    time::read() / (CLOCK_FREQ / 1000)
}

/// Sets the timer interrupt trigger for the next timer tick.
pub fn set_trigger() {
    sbi::set_timer(time::read() + CLOCK_FREQ / TICK_PER_SEC);
}

/// Enables the system timer interrupt.
pub fn enable_timer_interrupt() {
    unsafe {
        sie::set_stimer();
    }
}
