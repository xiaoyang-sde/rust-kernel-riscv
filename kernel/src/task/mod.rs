mod pid;
mod process;
mod thread;
mod tid;

use alloc::sync::Arc;

use lazy_static::{initialize, lazy_static};
pub use process::{Process, Status};
pub use thread::Thread;

lazy_static! {
    pub static ref INIT_PROCESS: Arc<Process> = Process::new("init");
}

pub fn init() {
    initialize(&INIT_PROCESS);
}
