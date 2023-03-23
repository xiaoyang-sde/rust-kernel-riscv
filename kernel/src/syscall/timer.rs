use crate::{executor::ControlFlow, syscall::SystemCall, timer};

impl SystemCall<'_> {
    pub fn sys_get_time(&self) -> (isize, ControlFlow) {
        (timer::get_time() as isize, ControlFlow::Continue)
    }
}
