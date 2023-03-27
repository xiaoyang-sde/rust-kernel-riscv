//! The `executor` module provides an executor that schedules and runs both the kernel threads and
//! the user threads.

use alloc::collections::VecDeque;
use core::future::Future;

use async_task::{Runnable, Task};
use lazy_static::lazy_static;
use riscv::register::{stvec, utvec::TrapMode};

use crate::{constant::TRAMPOLINE, sync::SharedRef};

mod context;
mod future;

pub use context::TrapContext;
pub use future::{spawn_thread, ControlFlow};

/// Initializes the `stvec` to the address of the `_enter_kernel_space` function, which is located
/// at the beginning of the [TRAMPOLINE] page.
pub fn init() {
    unsafe {
        stvec::write(TRAMPOLINE, TrapMode::Direct);
    }
}

/// The `TaskQueue` struct represents a queue of [Runnable] tasks, which are either kernel threads
/// or user threads.
struct TaskQueue {
    queue: VecDeque<Runnable>,
}

impl TaskQueue {
    fn new() -> Self {
        Self {
            queue: VecDeque::new(),
        }
    }

    fn len(&self) -> usize {
        self.queue.len()
    }

    fn push_back(&mut self, runnable: Runnable) {
        self.queue.push_back(runnable)
    }

    fn pop_front(&mut self) -> Option<Runnable> {
        self.queue.pop_front()
    }
}

lazy_static! {
    static ref TASK_QUEUE: SharedRef<TaskQueue> = unsafe { SharedRef::new(TaskQueue::new()) };
}

fn spawn<F>(future: F) -> (Runnable, Task<F::Output>)
where
    F: Future + Send + 'static,
    F::Output: Send + 'static,
{
    async_task::spawn(future, |runnable| {
        TASK_QUEUE.borrow_mut().push_back(runnable);
    })
}

/// Runs an event loop that executes all the tasks in the `TASK_QUEUE` until there are no more task
/// left.
pub fn run_until_complete() {
    loop {
        let task = TASK_QUEUE.borrow_mut().pop_front();
        if let Some(task) = task {
            task.run();
        } else {
            break;
        }
    }
}
