//! The `task` module provides types for representing processes and threads.

mod pid;
mod process;
mod thread;
mod tid;

use alloc::sync::Arc;

use lazy_static::{initialize, lazy_static};
pub use process::{Process, Status};
pub use thread::Thread;

lazy_static! {
    static ref INIT_PROCESS: Arc<Process> = Process::new("init");
}

/// Spawns the init process.
pub fn init() {
    initialize(&INIT_PROCESS);
}
