//! The `process` module provides system calls to interact with processes.

use alloc::vec::Vec;

use crate::{
    executor::ControlFlow,
    mem::UserPtr,
    sync::{wait_for_event, Event},
    syscall::SystemCall,
    task::Status,
};

impl SystemCall<'_> {
    /// Exits the current process with an exit code.
    pub fn sys_exit(&self, exit_code: usize) -> (isize, ControlFlow) {
        let tid = self.thread.tid();
        let process = self.thread.process();
        process
            .state()
            .thread_list_mut()
            .retain(|thread| thread.tid() != tid);

        if process.state().thread_list().is_empty() {
            process.exit(exit_code);
        }
        (0, ControlFlow::Exit)
    }

    pub fn sys_sched_yield(&self) -> (isize, ControlFlow) {
        (0, ControlFlow::Yield)
    }

    pub fn sys_fork(&self) -> (isize, ControlFlow) {
        let process = self.thread.process().fork();
        (process.pid() as isize, ControlFlow::Continue)
    }

    pub async fn sys_waitpid(
        &self,
        pid: isize,
        mut wait_status: UserPtr<usize>,
    ) -> (isize, ControlFlow) {
        loop {
            let process = self.thread.process();
            let mut process_state = process.state();
            let child_list = process_state.child_list_mut();

            if let Some((pid, exit_code)) = match pid {
                -1 | 0 => child_list.iter().find_map(|child_process| {
                    if child_process.state().status() == Status::Zombie {
                        Some((child_process.pid(), child_process.state().exit_code()))
                    } else {
                        None
                    }
                }),
                pid => child_list.iter().find_map(|child_process| {
                    if child_process.pid() == pid as usize
                        && child_process.state().status() == Status::Zombie
                    {
                        Some((child_process.pid(), child_process.state().exit_code()))
                    } else {
                        None
                    }
                }),
            } {
                child_list.retain(|child_process| child_process.pid() != pid);
                *wait_status = exit_code;
                return (pid as isize, ControlFlow::Continue);
            } else {
                let event_bus = process.event_bus();
                drop(process_state);
                wait_for_event(event_bus.clone(), Event::CHILD_PROCESS_QUIT).await;
                event_bus.lock().clear(Event::CHILD_PROCESS_QUIT);
            }
        }
    }

    pub fn sys_exec(&self, path: UserPtr<u8>) -> (isize, ControlFlow) {
        self.thread.process().exec(&path.as_string(), Vec::new());
        (0, ControlFlow::Continue)
    }
}
