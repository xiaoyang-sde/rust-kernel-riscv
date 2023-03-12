use core::ops::{Add, Sub};

use crate::constant::{PAGE_SIZE, PAGE_SIZE_BIT};
use crate::mem::FrameNumber;

const PHYSICAL_ADDRESS_SIZE: usize = 56;

/// The `PhysicalAddress` struct represents a 56-bit physical address defined in the Sv39
/// page table format.
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct PhysicalAddress {
    bits: usize,
}

impl PhysicalAddress {
    /// Returns the [FrameNumber] that represents the frame that contains the physical address.
    pub fn floor(&self) -> FrameNumber {
        FrameNumber {
            bits: self.bits / PAGE_SIZE,
        }
    }

    /// Returns the [FrameNumber] that represents the frame that contains the physical address,
    /// rounding up to the next frame if the physical address is not aligned to a frame.
    pub fn ceil(&self) -> FrameNumber {
        FrameNumber {
            bits: (self.bits + PAGE_SIZE - 1) / PAGE_SIZE,
        }
    }

    /// Returns the byte offset of the physical address within its containing frame.
    pub fn page_offset(&self) -> usize {
        self.bits & (PAGE_SIZE - 1)
    }

    /// Returns `true` if the physical address is aligned to a frame.
    pub fn is_aligned(&self) -> bool {
        self.page_offset() == 0
    }

    /// Returns a raw pointer to the physical address.
    pub fn as_ptr(&self) -> *const u8 {
        self.bits as *const u8
    }

    /// Returns a mutable raw pointer to the physical address.
    pub fn as_ptr_mut(&self) -> *mut u8 {
        self.bits as *mut u8
    }
}

impl Add<usize> for PhysicalAddress {
    type Output = Self;

    fn add(self, rhs: usize) -> Self {
        Self::from(self.bits + rhs)
    }
}

impl Sub<usize> for PhysicalAddress {
    type Output = Self;

    fn sub(self, rhs: usize) -> Self {
        Self::from(self.bits - rhs)
    }
}

impl From<usize> for PhysicalAddress {
    fn from(value: usize) -> Self {
        Self {
            bits: value & ((1 << PHYSICAL_ADDRESS_SIZE) - 1),
        }
    }
}

impl From<PhysicalAddress> for usize {
    fn from(value: PhysicalAddress) -> Self {
        value.bits
    }
}

impl From<FrameNumber> for PhysicalAddress {
    fn from(value: FrameNumber) -> Self {
        Self {
            bits: value.bits << PAGE_SIZE_BIT,
        }
    }
}
