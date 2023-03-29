//! The `sync` module provides synchronization primitives for concurrent programming.

mod event_bus;
mod mutex;

pub use event_bus::{wait_for_event, Event, EventBus};
pub use mutex::{Mutex, MutexGuard};
