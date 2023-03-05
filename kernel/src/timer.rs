use riscv::register::{sie, sstatus, time};

use crate::{constant::CLOCK_FREQ, sbi};

const TICK_PER_SEC: usize = 100;

pub fn get_time() -> usize {
    time::read() / (CLOCK_FREQ / 1000)
}

pub fn set_trigger() {
    sbi::set_timer(time::read() + CLOCK_FREQ / TICK_PER_SEC);
}

pub fn enable_timer_interrupt() {
    unsafe {
        sie::set_stimer();
        sstatus::set_sie();
    }
}
