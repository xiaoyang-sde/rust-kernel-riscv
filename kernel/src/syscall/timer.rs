use crate::{executor::TaskAction, syscall::SystemCall, timer};

impl SystemCall<'_> {
    pub fn sys_get_time(&self) -> (isize, TaskAction) {
        (timer::get_time() as isize, TaskAction::Continue)
    }
}
