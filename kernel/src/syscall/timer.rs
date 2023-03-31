//! The `timer` module provides time-related system calls.

use crate::{executor::ControlFlow, syscall::SystemCall, timer};

impl SystemCall<'_> {
    /// Returns the current system time in milliseconds.
    pub fn sys_get_time(&self) -> (isize, ControlFlow) {
        (timer::get_time() as isize, ControlFlow::Continue)
    }
}
