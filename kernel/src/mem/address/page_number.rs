use core::ops::Sub;

use crate::{executor::TrapContext, mem::VirtualAddress};

const PAGE_NUMBER_SIZE: usize = 27;

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct PageNumber {
    bits: usize,
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

    /// Interprets the page as a [TrapContext] and return a mutable reference to it.
    pub fn as_trap_context_mut(&self) -> &'static mut TrapContext {
        let virtual_address = VirtualAddress::from(*self);
        unsafe {
            (virtual_address.as_ptr_mut() as *mut TrapContext)
                .as_mut()
                .unwrap()
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
#[derive(Clone)]
pub struct PageRange {
    start: PageNumber,
    end: PageNumber,
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

    pub fn start(&self) -> PageNumber {
        self.start
    }

    pub fn end(&self) -> PageNumber {
        self.end
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
