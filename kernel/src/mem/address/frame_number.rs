use core::{
    mem,
    ops::{Add, AddAssign},
    slice,
};

use crate::{
    constant::PAGE_SIZE,
    executor::TrapContext,
    mem::{page_table::PageTableEntry, PhysicalAddress},
};

const FRAME_NUMBER_SIZE: usize = 44;

/// The `FrameNumber` struct represents the number of a 44-bit page frame defined in the Sv39
/// page table format.
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct FrameNumber {
    pub bits: usize,
}

impl FrameNumber {
    /// Interprets the frame as a slice of `u8` and return a reference to the slice.
    pub fn as_bytes(&self) -> &'static [u8] {
        let physical_address = PhysicalAddress::from(*self);
        unsafe {
            slice::from_raw_parts(physical_address.as_ptr(), PAGE_SIZE / mem::size_of::<u8>())
        }
    }

    /// Interprets the frame as a slice of `u8` and return a mutable reference to the slice.
    pub fn as_bytes_mut(&self) -> &'static mut [u8] {
        let physical_address = PhysicalAddress::from(*self);
        unsafe {
            slice::from_raw_parts_mut(
                physical_address.as_ptr_mut(),
                PAGE_SIZE / mem::size_of::<u8>(),
            )
        }
    }

    /// Interprets the frame as a slice of [PageTableEntry] and return a reference to the slice.
    pub fn as_pte(&self) -> &'static [PageTableEntry] {
        let physical_address = PhysicalAddress::from(*self);
        unsafe {
            slice::from_raw_parts(
                physical_address.as_ptr() as *const PageTableEntry,
                PAGE_SIZE / mem::size_of::<PageTableEntry>(),
            )
        }
    }

    /// Interprets the frame as a slice of [PageTableEntry] and return a mutable reference to the
    /// slice.
    pub fn as_pte_mut(&self) -> &'static mut [PageTableEntry] {
        let physical_address = PhysicalAddress::from(*self);
        unsafe {
            slice::from_raw_parts_mut(
                physical_address.as_ptr_mut() as *mut PageTableEntry,
                PAGE_SIZE / mem::size_of::<PageTableEntry>(),
            )
        }
    }

    /// Interprets the frame as a [TrapContext] and return a mutable reference to it.
    pub fn as_trap_context_mut(&self) -> &'static mut TrapContext {
        let physical_address = PhysicalAddress::from(*self);
        unsafe {
            (physical_address.as_ptr_mut() as *mut TrapContext)
                .as_mut()
                .unwrap()
        }
    }
}

impl Add<usize> for FrameNumber {
    type Output = Self;

    fn add(self, rhs: usize) -> Self {
        Self::from(self.bits + rhs)
    }
}

impl AddAssign<usize> for FrameNumber {
    fn add_assign(&mut self, rhs: usize) {
        *self = *self + rhs;
    }
}

impl From<usize> for FrameNumber {
    fn from(value: usize) -> Self {
        Self {
            bits: value & ((1 << FRAME_NUMBER_SIZE) - 1),
        }
    }
}

impl From<FrameNumber> for usize {
    fn from(value: FrameNumber) -> Self {
        value.bits
    }
}

impl From<PhysicalAddress> for FrameNumber {
    fn from(value: PhysicalAddress) -> Self {
        assert_eq!(value.page_offset(), 0);
        value.floor()
    }
}
