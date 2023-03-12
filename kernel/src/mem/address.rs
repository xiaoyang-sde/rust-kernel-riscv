//! The `address` module defines various structs for the page table.

use core::ops::{Add, Sub};
use core::slice;

use crate::constant::{PAGE_SIZE, PAGE_SIZE_BIT};
use crate::mem::page_table::PageTableEntry;
use crate::trap::TrapContext;

const PHYSICAL_ADDRESS_SIZE: usize = 56;
const FRAME_NUMBER_SIZE: usize = PHYSICAL_ADDRESS_SIZE - PAGE_SIZE_BIT;

const VIRTUAL_ADDRESS_SIZE: usize = 39;
const PAGE_NUMBER_SIZE: usize = VIRTUAL_ADDRESS_SIZE - PAGE_SIZE_BIT;

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct PhysicalAddress {
    pub bits: usize,
}

impl PhysicalAddress {
    pub fn floor(&self) -> FrameNumber {
        FrameNumber {
            bits: self.bits / PAGE_SIZE,
        }
    }

    pub fn ceil(&self) -> FrameNumber {
        FrameNumber {
            bits: (self.bits + PAGE_SIZE - 1) / PAGE_SIZE,
        }
    }

    pub fn page_offset(&self) -> usize {
        self.bits & (PAGE_SIZE - 1)
    }

    pub fn aligned(&self) -> bool {
        self.page_offset() == 0
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

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct FrameNumber {
    pub bits: usize,
}

impl FrameNumber {
    /// Interpret the frame as a slice of `u8` and return a reference to the slice.
    pub fn get_bytes(&self) -> &'static [u8] {
        let physical_address: PhysicalAddress = (*self).into();
        unsafe { slice::from_raw_parts(physical_address.bits as *const u8, 4096) }
    }

    /// Interpret the frame as a slice of `u8` and return a mutable reference to the slice.
    pub fn get_bytes_mut(&self) -> &'static mut [u8] {
        let physical_address: PhysicalAddress = (*self).into();
        unsafe { slice::from_raw_parts_mut(physical_address.bits as *mut u8, 4096) }
    }

    /// Interpret the frame as a slice of [PageTableEntry] and return a reference to the slice.
    pub fn get_pte(&self) -> &'static [PageTableEntry] {
        let physical_address: PhysicalAddress = (*self).into();
        unsafe { slice::from_raw_parts(physical_address.bits as *const PageTableEntry, 512) }
    }

    /// Interpret the frame as a slice of [PageTableEntry] and return a mutable reference to the slice.
    pub fn get_pte_mut(&self) -> &'static mut [PageTableEntry] {
        let physical_address: PhysicalAddress = (*self).into();
        unsafe { slice::from_raw_parts_mut(physical_address.bits as *mut PageTableEntry, 512) }
    }

    /// Interpret the frame as a [TrapContext] and return a reference to it.
    pub fn get_trap_context(&self) -> &'static mut TrapContext {
        let physical_address: PhysicalAddress = (*self).into();
        unsafe {
            (physical_address.bits as *mut TrapContext)
                .as_mut()
                .unwrap()
        }
    }

    pub fn offset(&mut self, rhs: usize) -> Self {
        FrameNumber {
            bits: (self.bits + rhs) & ((1 << FRAME_NUMBER_SIZE) - 1),
        }
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

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct VirtualAddress {
    pub bits: usize,
}

impl VirtualAddress {
    pub fn floor(&self) -> PageNumber {
        PageNumber {
            bits: self.bits / PAGE_SIZE,
        }
    }

    pub fn ceil(&self) -> PageNumber {
        PageNumber {
            bits: (self.bits + PAGE_SIZE - 1) / PAGE_SIZE,
        }
    }

    pub fn page_offset(&self) -> usize {
        self.bits & (PAGE_SIZE - 1)
    }

    pub fn aligned(&self) -> bool {
        self.page_offset() == 0
    }
}

impl Add<usize> for VirtualAddress {
    type Output = Self;

    fn add(self, rhs: usize) -> Self {
        Self::from(self.bits + rhs)
    }
}

impl From<usize> for VirtualAddress {
    fn from(value: usize) -> Self {
        Self {
            bits: value & ((1 << VIRTUAL_ADDRESS_SIZE) - 1),
        }
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
            bits: value.bits << PAGE_SIZE_BIT,
        }
    }
}

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct PageNumber {
    pub bits: usize,
}

impl PageNumber {
    pub fn index(&self) -> [usize; 3] {
        let mask = (1 << 9) - 1;
        [
            (self.bits >> 18) & mask,
            (self.bits >> 9) & mask,
            self.bits & mask,
        ]
    }

    pub fn offset(&mut self, rhs: usize) -> Self {
        PageNumber {
            bits: (self.bits + rhs) & ((1 << PAGE_NUMBER_SIZE) - 1),
        }
    }
}

impl From<usize> for PageNumber {
    fn from(value: usize) -> Self {
        Self {
            bits: value & ((1 << PAGE_NUMBER_SIZE) - 1),
        }
    }
}

impl From<PageNumber> for usize {
    fn from(value: PageNumber) -> Self {
        value.bits
    }
}

impl Sub<PageNumber> for PageNumber {
    type Output = usize;

    fn sub(self, rhs: PageNumber) -> usize {
        self.bits - rhs.bits
    }
}

impl From<VirtualAddress> for PageNumber {
    fn from(value: VirtualAddress) -> Self {
        value.floor()
    }
}

/// The `PageRange` struct represents a range of page numbers,
/// with `start` and `end` field holding [PageNumber] values.
pub struct PageRange {
    pub start: PageNumber,
    pub end: PageNumber,
}

impl PageRange {
    pub fn new(start: PageNumber, end: PageNumber) -> Self {
        Self { start, end }
    }

    pub fn iter(&self) -> PageRangeIterator {
        PageRangeIterator::new(self.start, self.end)
    }

    pub fn len(&self) -> usize {
        self.end - self.start
    }
}

impl IntoIterator for PageRange {
    type Item = PageNumber;
    type IntoIter = PageRangeIterator;

    fn into_iter(self) -> Self::IntoIter {
        PageRangeIterator::new(self.start, self.end)
    }
}

pub struct PageRangeIterator {
    state: PageNumber,
    end: PageNumber,
}

impl PageRangeIterator {
    pub fn new(start: PageNumber, end: PageNumber) -> Self {
        Self { state: start, end }
    }
}

impl Iterator for PageRangeIterator {
    type Item = PageNumber;

    fn next(&mut self) -> Option<Self::Item> {
        if self.state == self.end {
            None
        } else {
            let result = self.state;
            self.state = self.state.offset(1);
            Some(result)
        }
    }
}
