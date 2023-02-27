//! The `sync` module provides synchronization primitives for concurrent programming.

mod shared_ref;

pub use self::shared_ref::SharedRef;
