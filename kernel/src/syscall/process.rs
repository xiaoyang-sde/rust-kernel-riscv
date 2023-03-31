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
    /// Terminate the current thread with the given exit code.
    pub fn sys_exit(&self, exit_code: usize) -> (isize, ControlFlow) {
        (0, ControlFlow::Exit(exit_code))
    }

    /// Yield the CPU to another thread.
    pub fn sys_sched_yield(&self) -> (isize, ControlFlow) {
        (0, ControlFlow::Yield)
    }

    /// Fork the current process and create a new child process.
    pub fn sys_fork(&self) -> (isize, ControlFlow) {
        let process = self.thread.process().fork();
        (process.pid() as isize, ControlFlow::Continue)
    }

    /// Wait for a child process with the given process to terminate, and return the PID and exit.
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

    /// Replace the current process with a new process loaded from the executable file with a given
    /// name.
    pub fn sys_exec(&self, path: UserPtr<u8>) -> (isize, ControlFlow) {
        self.thread.process().exec(&path.as_string(), Vec::new());
        (0, ControlFlow::Continue)
    }
}
