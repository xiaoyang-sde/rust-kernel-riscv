//! The `page_table` module defines a 3-level page table
//! that follows the RISC-V Sv39 page table specification.
//! The page table supports 512 GB of virtual-address space.

#![macro_use]
use alloc::vec;

use alloc::vec::Vec;
use bitflags::bitflags;

use crate::{
    constant::PAGE_SIZE,
    mem::{
        address::PageRange,
        frame_allocator::{allocate_frame, FrameTracker},
        FrameNumber, PageNumber, VirtualAddress,
    },
};

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
    bits: usize,
}

impl PageTableEntry {
    pub fn new(frame_number: FrameNumber, flag: PTEFlags) -> Self {
        PageTableEntry {
            bits: frame_number.bits << 10 | flag.bits as usize,
        }
    }

    pub fn frame_number(&self) -> FrameNumber {
        FrameNumber::from(self.bits >> 10 & ((1 << 44) - 1))
    }

    pub fn flags(&self) -> PTEFlags {
        PTEFlags::from_bits(self.bits as u8).unwrap()
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
        8 << 60 | self.root_frame_number.bits
    }

    /// Creates a [PageTable] where the `root_frame_number` points to the framed in the `satp`
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

    /// Finds the page table with a [PageNumber] and returns a [PageTableEntry].
    pub fn translate(&self, page_number: PageNumber) -> Option<PageTableEntry> {
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

pub fn translate_buffer(satp: usize, buffer: *const u8, length: usize) -> Vec<&'static [u8]> {
    let page_table = PageTable::from_satp(satp);
    let mut translated_buffer = Vec::new();

    let buffer_address_start = VirtualAddress::from(buffer as usize);
    let buffer_address_end = buffer_address_start + length;

    let page_range = PageRange::new(
        PageNumber::from(buffer_address_start),
        PageNumber::from(buffer_address_end).offset(1),
    );

    for (index, page_number) in page_range.iter().enumerate() {
        let frame_number = page_table.translate(page_number).unwrap().frame_number();

        let lower_bound = {
            if index == 0 {
                buffer_address_start.page_offset()
            } else {
                0
            }
        };

        let upper_bound = {
            if index == page_range.len() - 1 {
                buffer_address_end.page_offset()
            } else {
                PAGE_SIZE
            }
        };
        translated_buffer.push(&frame_number.as_bytes()[lower_bound..upper_bound]);
    }

    translated_buffer
}
