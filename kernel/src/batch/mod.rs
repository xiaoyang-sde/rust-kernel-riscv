//! The `batch` module contains a runtime for binaries
//! that run in the user mode.

mod runtime;
mod stack;

pub use runtime::{init, is_valid_pointer, load_next_bin};
