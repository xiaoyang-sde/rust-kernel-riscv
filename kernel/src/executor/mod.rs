use core::future::Future;

use alloc::collections::VecDeque;
use async_task::{Runnable, Task};
use lazy_static::lazy_static;
use riscv::register::{stvec, utvec::TrapMode};

use crate::{constant::TRAMPOLINE, sync::SharedRef};

mod context;
mod future;

pub use context::TrapContext;
pub use future::spawn_thread;
pub use future::TaskAction;

pub struct TaskQueue {
    queue: VecDeque<Runnable>,
}

impl TaskQueue {
    pub fn new() -> Self {
        Self {
            queue: VecDeque::new(),
        }
    }

    pub fn len(&self) -> usize {
        self.queue.len()
    }

    pub fn push_back(&mut self, runnable: Runnable) {
        self.queue.push_back(runnable)
    }

    pub fn pop_front(&mut self) -> Option<Runnable> {
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

pub fn init() {
    unsafe {
        stvec::write(TRAMPOLINE, TrapMode::Direct);
    }
}
