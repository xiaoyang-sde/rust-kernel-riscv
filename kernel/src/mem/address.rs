use crate::constant::{PAGE_SIZE, PAGE_SIZE_BIT};

const PHYSICAL_ADDRESS_SIZE: usize = 56;
const PHYSICAL_PAGE_NUMBER_SIZE: usize = PHYSICAL_ADDRESS_SIZE - PAGE_SIZE_BIT;

const VIRTUAL_ADDRESS_SIZE: usize = 39;
const VIRTUAL_PAGE_NUMBER_SIZE: usize = VIRTUAL_ADDRESS_SIZE - PAGE_SIZE_BIT;

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct PhysicalAddress {
    pub bits: usize,
}

impl PhysicalAddress {
    pub fn floor(&self) -> PhysicalPageNumber {
        PhysicalPageNumber {
            bits: self.bits / PAGE_SIZE,
        }
    }

    pub fn ceil(&self) -> PhysicalPageNumber {
        PhysicalPageNumber {
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

impl From<PhysicalPageNumber> for PhysicalAddress {
    fn from(value: PhysicalPageNumber) -> Self {
        Self {
            bits: value.bits << PAGE_SIZE_BIT,
        }
    }
}

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct PhysicalPageNumber {
    pub bits: usize,
}

impl From<usize> for PhysicalPageNumber {
    fn from(value: usize) -> Self {
        Self {
            bits: value & ((1 << PHYSICAL_PAGE_NUMBER_SIZE) - 1),
        }
    }
}

impl From<PhysicalPageNumber> for usize {
    fn from(value: PhysicalPageNumber) -> Self {
        value.bits
    }
}

impl From<PhysicalAddress> for PhysicalPageNumber {
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
pub struct VirtualPageNumber {
    pub bits: usize,
}

impl From<usize> for VirtualPageNumber {
    fn from(value: usize) -> Self {
        Self {
            bits: value & ((1 << VIRTUAL_PAGE_NUMBER_SIZE) - 1),
        }
    }
}

impl From<VirtualPageNumber> for usize {
    fn from(value: VirtualPageNumber) -> Self {
        value.bits
    }
}
