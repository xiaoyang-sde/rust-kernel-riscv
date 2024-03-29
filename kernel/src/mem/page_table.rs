//! The `page_table` module defines a 3-level page table
//! that follows the RISC-V Sv39 page table specification.
//! The page table supports 512 GB of virtual-address space.

#![macro_use]
use alloc::{vec, vec::Vec};

use bitflags::bitflags;

use crate::mem::{
    frame_allocator::{allocate_frame, FrameTracker},
    FrameNumber,
    PageNumber,
    PhysicalAddress,
    VirtualAddress,
};

bitflags! {
    #[derive(Copy, Clone)]
    pub struct PTEFlags: usize {
        const V = 1 << 0;
        const R = 1 << 1;
        const W = 1 << 2;
        const X = 1 << 3;
        const U = 1 << 4;
        const G = 1 << 5;
        const A = 1 << 6;
        const D = 1 << 7;
        const COW = 1 << 8;
    }
}

#[derive(Copy, Clone, Default)]
#[repr(C)]
pub struct PageTableEntry {
    bits: usize,
}

impl PageTableEntry {
    pub fn new(frame_number: FrameNumber, flag: PTEFlags) -> Self {
        PageTableEntry {
            bits: usize::from(frame_number) << 10 | flag.bits(),
        }
    }

    pub fn frame_number(&self) -> FrameNumber {
        FrameNumber::from(self.bits >> 10 & ((1 << 44) - 1))
    }

    pub fn flags(&self) -> PTEFlags {
        PTEFlags::from_bits(self.bits & ((1 << 10) - 1)).unwrap()
    }

    pub fn is_valid(&self) -> bool {
        self.flags().contains(PTEFlags::V)
    }

    pub fn is_readable(&self) -> bool {
        self.flags().contains(PTEFlags::R)
    }

    pub fn is_writable(&self) -> bool {
        self.flags().contains(PTEFlags::W)
    }

    pub fn is_executable(&self) -> bool {
        self.flags().contains(PTEFlags::X)
    }

    pub fn is_cow(&self) -> bool {
        self.flags().contains(PTEFlags::COW)
    }
}

#[repr(C)]
pub struct PageTable {
    root_frame_number: FrameNumber,
    frame_list: Vec<FrameTracker>,
}

impl PageTable {
    pub fn new() -> Self {
        let frame = allocate_frame().unwrap();
        PageTable {
            root_frame_number: frame.frame_number(),
            frame_list: vec![frame],
        }
    }

    /// Returns the value of the `satp` register that points to the page table.
    pub fn satp(&self) -> usize {
        8 << 60 | usize::from(self.root_frame_number)
    }

    /// Creates a [PageTable] where the `root_frame_number` points to the frame in the `satp`
    /// register.
    pub fn from_satp(satp: usize) -> Self {
        Self {
            root_frame_number: FrameNumber::from(satp & ((1 << 44) - 1)),
            frame_list: Vec::new(),
        }
    }

    /// Maps a [PageNumber] to a [FrameNumber] and sets the [PageTableEntry] with [PTEFlags].
    pub fn map(&mut self, page_number: PageNumber, frame_number: FrameNumber, flags: PTEFlags) {
        let pte = self.create_pte(page_number).unwrap();
        *pte = PageTableEntry::new(frame_number, flags | PTEFlags::V);
    }

    /// Clears the [PageTableEntry] corresponding to the [PageNumber].
    pub fn unmap(&mut self, page_number: PageNumber) {
        let pte = self.find_pte(page_number).unwrap();
        *pte = PageTableEntry::default();
    }

    /// Finds the page table with a [VirtualAddress] and returns a [PhysicalAddress].
    pub fn translate(&self, virtual_address: VirtualAddress) -> Option<PhysicalAddress> {
        let page_number = PageNumber::from(virtual_address);
        self.find_pte(page_number).map(|pte| {
            let frame_number = pte.frame_number();
            PhysicalAddress::from(frame_number) + virtual_address.page_offset()
        })
    }

    /// Finds the page table with a [PageNumber] and returns a [PageTableEntry].
    pub fn translate_page(&self, page_number: PageNumber) -> Option<PageTableEntry> {
        self.find_pte(page_number).map(|pte| *pte)
    }

    /// Finds the page table with a [PageNumber] and returns a mutable reference to a
    /// [PageTableEntry].
    fn find_pte(&self, page_number: PageNumber) -> Option<&mut PageTableEntry> {
        let index = page_number.index();
        let mut frame_number = self.root_frame_number;
        for (i, pte_index) in index.iter().enumerate() {
            let pte = &mut frame_number.as_pte_mut()[*pte_index];
            if i == 2 {
                return Some(pte);
            }

            if pte.is_valid() {
                frame_number = pte.frame_number();
            } else {
                return None;
            }
        }
        None
    }

    /// Finds the page table with a [PageNumber] and returns a mutable reference to a
    /// [PageTableEntry]. Creates a new [PageTableEntry] if not existed.
    fn create_pte(&mut self, page_number: PageNumber) -> Option<&mut PageTableEntry> {
        let index = page_number.index();
        let mut frame_number = self.root_frame_number;
        for (i, pte_index) in index.iter().enumerate() {
            let pte = &mut frame_number.as_pte_mut()[*pte_index];
            if i == 2 {
                return Some(pte);
            }

            if !pte.is_valid() {
                let frame = allocate_frame().unwrap();
                *pte = PageTableEntry::new(frame.frame_number(), PTEFlags::V);
                self.frame_list.push(frame);
            }
            frame_number = pte.frame_number();
        }
        None
    }
}
