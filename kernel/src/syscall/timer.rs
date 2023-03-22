use crate::{executor::TaskAction, timer};

use super::SystemCall;

impl SystemCall<'_> {
    pub fn sys_get_time(&self) -> (isize, TaskAction) {
        (timer::get_time() as isize, TaskAction::Continue)
    }
}
