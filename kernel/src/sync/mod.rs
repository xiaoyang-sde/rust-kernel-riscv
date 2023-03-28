//! The `sync` module provides synchronization primitives for concurrent programming.

mod event_bus;
mod mutex;

pub use mutex::{Mutex, MutexGuard};
