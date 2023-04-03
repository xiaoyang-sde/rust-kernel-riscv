use alloc::collections::VecDeque;

use async_task::Runnable;

pub trait Scheduler {
    fn schedule(&mut self, runnable: Runnable);

    fn task(&mut self) -> Option<Runnable>;
}

/// The `TaskQueue` struct represents a queue of [Runnable] tasks, which are either kernel threads
/// or user threads.
pub struct TaskQueue {
    queue: VecDeque<Runnable>,
}

impl TaskQueue {
    pub fn new() -> Self {
        Self {
            queue: VecDeque::new(),
        }
    }
}

impl Scheduler for TaskQueue {
    fn schedule(&mut self, runnable: Runnable) {
        self.queue.push_back(runnable)
    }

    fn task(&mut self) -> Option<Runnable> {
        self.queue.pop_front()
    }
}
