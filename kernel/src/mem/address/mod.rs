//! The `address` module defines various structs for the Sv39 page table specification.

mod frame_number;
mod page_number;
mod physical_address;
mod virtual_address;

pub use frame_number::FrameNumber;
pub use page_number::{PageNumber, PageRange};
pub use physical_address::PhysicalAddress;
pub use virtual_address::VirtualAddress;
