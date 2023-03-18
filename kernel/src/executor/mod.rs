use core::future::Future;

use alloc::collections::VecDeque;
use async_task::{Runnable, Task};
use lazy_static::lazy_static;

use crate::sync::SharedRef;

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

    pub fn push_back(&self, runnable: Runnable) {
        self.queue.push_back(runnable)
    }

    pub fn pop_front(&self) -> Option<Runnable> {
        self.queue.pop_front()
    }
}

lazy_static! {
    static ref TASK_QUEUE: SharedRef<TaskQueue> = SharedRef::new(TaskQueue::new());
}

pub fn spawn<F>(future: F) -> (Runnable, Task<F::Output>)
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
        if let Some(task) = { TASK_QUEUE.borrow_mut().pop_front() } {
            task.run();
        } else {
            break;
        }
    }
}
