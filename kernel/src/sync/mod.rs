//! The `sync` module provides synchronization primitives for concurrent programming.

mod mutex;

pub use mutex::{Mutex, MutexGuard};
