//! The `sync` module provides synchronization primitives for concurrent programming.

mod event_bus;
mod mutex;

pub use event_bus::{Event, EventBus};
pub use mutex::{Mutex, MutexGuard};
