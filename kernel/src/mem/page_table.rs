use bitflags::bitflags;

use crate::mem::PhysicalPageNumber;

bitflags! {
  pub struct PTEFlags: u8 {
    const V = 1 << 0;
    const R = 1 << 1;
        const W = 1 << 2;
        const X = 1 << 3;
        const U = 1 << 4;
        const G = 1 << 5;
        const A = 1 << 6;
        const D = 1 << 7;
  }
}

#[derive(Copy, Clone, Default)]
#[repr(C)]
pub struct PageTableEntry {
    pub bits: usize,
}

impl PageTableEntry {
    pub fn new(physical_page_number: PhysicalPageNumber, flag: PTEFlags) -> Self {
        PageTableEntry {
            bits: physical_page_number.bits << 10 | flag.bits as usize,
        }
    }

    pub fn physical_page_number(&self) -> PhysicalPageNumber {
        (self.bits >> 10 & ((1 << 44) - 1)).into()
    }

    pub fn flags(&self) -> PTEFlags {
        PTEFlags::from_bits(self.bits as u8).unwrap()
    }

    pub fn is_valid(&self) -> bool {
        (self.flags() & PTEFlags::V) != PTEFlags::empty()
    }
}
