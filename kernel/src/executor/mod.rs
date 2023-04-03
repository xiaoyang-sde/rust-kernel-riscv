//! The `executor` module provides an executor that schedules and runs both the kernel threads and
//! the user threads.

use alloc::boxed::Box;
use core::future::Future;

use async_task::{Runnable, Task};
use lazy_static::lazy_static;
use riscv::register::{stvec, utvec::TrapMode};

use crate::{
    constant::TRAMPOLINE,
    executor::scheduler::{Scheduler, TaskQueue},
    sync::Mutex,
};

mod context;
mod future;
mod scheduler;

pub use context::TrapContext;
pub use future::{spawn_thread, yield_now, ControlFlow};

/// Initializes the `stvec` to the address of the `_enter_kernel_space` function, which is located
/// at the beginning of the [TRAMPOLINE] page.
pub fn init() {
    unsafe {
        stvec::write(TRAMPOLINE, TrapMode::Direct);
    }
}

lazy_static! {
    static ref SCHEDULER: Mutex<Box<dyn Scheduler>> = Mutex::new(Box::new(TaskQueue::new()));
}

fn spawn<F>(future: F) -> (Runnable, Task<F::Output>)
where
    F: Future + Send + 'static,
    F::Output: Send + 'static,
{
    async_task::spawn(future, |runnable| {
        SCHEDULER.lock().schedule(runnable);
    })
}

/// Runs an event loop that executes all the tasks in the `TASK_QUEUE` until there are no more task
/// left.
pub fn run_until_complete() {
    loop {
        let task = SCHEDULER.lock().task();
        if let Some(task) = task {
            task.run();
        } else {
            break;
        }
    }
}
