//! The `sync` module provides synchronization primitives for concurrent programming.

mod mutex;
mod shared_ref;

pub use mutex::{Mutex, MutexGuard};
pub use shared_ref::SharedRef;
