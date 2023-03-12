use core::ops::{Add, Sub};

use crate::constant::{PAGE_SIZE, PAGE_SIZE_BIT};
use crate::mem::PageNumber;

/// The `VirtualAddress` struct represents a 39-bit virtual address defined in the Sv39
/// page table format.
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct VirtualAddress {
    bits: usize,
}

impl VirtualAddress {
    /// Returns the [PageNumber] that represents the page that contains the virtual address.
    pub fn floor(&self) -> PageNumber {
        PageNumber::from(self.bits / PAGE_SIZE)
    }

    /// Returns the [PageNumber] that represents the page that contains the virtual address,
    /// rounding up to the next frame if the physical address is not aligned to a frame.
    pub fn ceil(&self) -> PageNumber {
        PageNumber::from((self.bits - 1 + PAGE_SIZE) / PAGE_SIZE)
    }

    /// Returns the byte offset of the virtual address within its containing page.
    pub fn page_offset(&self) -> usize {
        self.bits & (PAGE_SIZE - 1)
    }

    /// Returns `true` if the virtual address is aligned to a page.
    pub fn is_aligned(&self) -> bool {
        self.page_offset() == 0
    }
}

impl Add<usize> for VirtualAddress {
    type Output = Self;

    fn add(self, rhs: usize) -> Self {
        Self::from(self.bits + rhs)
    }
}

impl Sub<usize> for VirtualAddress {
    type Output = Self;

    fn sub(self, rhs: usize) -> Self {
        Self::from(self.bits - rhs)
    }
}

impl From<usize> for VirtualAddress {
    fn from(value: usize) -> Self {
        assert!((value >> 39) == 0 || (value >> 39) == (1 << 25) - 1);
        Self { bits: value }
    }
}

impl From<VirtualAddress> for usize {
    fn from(value: VirtualAddress) -> Self {
        value.bits
    }
}

impl From<PageNumber> for VirtualAddress {
    fn from(value: PageNumber) -> Self {
        Self {
            bits: usize::from(value) << PAGE_SIZE_BIT,
        }
    }
}
