use core::slice;

use crate::constant::{PAGE_SIZE, PAGE_SIZE_BIT};
use crate::mem::page_table::PageTableEntry;

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
    pub fn get_bytes(&self) -> &'static [u8] {
        let physical_address: PhysicalAddress = (*self).into();
        unsafe { slice::from_raw_parts(physical_address.bits as *const u8, 4096) }
    }

    pub fn get_bytes_mut(&self) -> &'static mut [u8] {
        let physical_address: PhysicalAddress = (*self).into();
        unsafe { slice::from_raw_parts_mut(physical_address.bits as *mut u8, 4096) }
    }

    pub fn get_pte(&self) -> &'static [PageTableEntry] {
        let physical_address: PhysicalAddress = (*self).into();
        unsafe { slice::from_raw_parts(physical_address.bits as *const PageTableEntry, 512) }
    }

    pub fn get_pte_mut(&self) -> &'static mut [PageTableEntry] {
        let physical_address: PhysicalAddress = (*self).into();
        unsafe { slice::from_raw_parts_mut(physical_address.bits as *mut PageTableEntry, 512) }
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

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct PageNumber {
    pub bits: usize,
}

impl PageNumber {
    pub fn get_index(&self) -> [usize; 3] {
        let mask = (1 << 9) - 1;
        [
            (self.bits >> 18) & mask,
            (self.bits >> 9) & mask,
            self.bits & mask,
        ]
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
