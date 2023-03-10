use alloc::collections::BTreeMap;
use alloc::vec::Vec;
use bitflags::bitflags;

use crate::constant::PAGE_SIZE;
use crate::mem::{
    address::PageRange,
    allocate_frame,
    frame_allocator::FrameTracker,
    page_table::{PTEFlags, PageTable},
    FrameNumber, PageNumber, VirtualAddress,
};

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum MapType {
    Identical,
    Framed,
}

bitflags! {
  pub struct MapPermission: u8 {
      const R = 1 << 1;
      const W = 1 << 2;
      const X = 1 << 3;
      const U = 1 << 4;
  }
}

/// The `PageSegment` struct represents a consective range of pages,
/// which are mapped to frames in the same method (identical or framed)
/// and have the same permissions.
pub struct PageSegment {
    page_range: PageRange,
    frame_map: BTreeMap<PageNumber, FrameTracker>,
    map_type: MapType,
    map_permission: MapPermission,
}

impl PageSegment {
    pub fn new(
        start_address: VirtualAddress,
        end_address: VirtualAddress,
        map_type: MapType,
        map_permission: MapPermission,
    ) -> Self {
        Self {
            page_range: PageRange::new(start_address.floor(), end_address.ceil()),
            frame_map: BTreeMap::new(),
            map_type,
            map_permission,
        }
    }

    /// Map the range of pages represented with `page_range` to frames in the `page_table`.
    pub fn map_range(&mut self, page_table: &mut PageTable) {
        for page_number in self.page_range.iter() {
            self.map_page(page_table, page_number);
        }
    }

    /// Unmap the range of pages represented with `page_range` from frames in the `page_table`.
    pub fn unmap_range(&mut self, page_table: &mut PageTable) {
        for page_number in self.page_range.iter() {
            self.unmap_page(page_table, page_number);
        }
    }

    /// Map a page with `page_number` to a frame in the `page_table`.
    pub fn map_page(&mut self, page_table: &mut PageTable, page_number: PageNumber) {
        let frame_number = match self.map_type {
            MapType::Identical => FrameNumber {
                bits: page_number.bits,
            },
            MapType::Framed => {
                let frame = allocate_frame().unwrap();
                let frame_number = frame.frame_number;
                self.frame_map.insert(page_number, frame);
                frame_number
            }
        };

        let pte_flags = PTEFlags::from_bits(self.map_permission.bits).unwrap();
        page_table.map(page_number, frame_number, pte_flags);
    }

    /// Unmap a page with `page_number` from a frame in the `page_table`.
    pub fn unmap_page(&mut self, page_table: &mut PageTable, page_number: PageNumber) {
        if self.map_type == MapType::Framed {
            self.frame_map.remove(&page_number);
        }
        page_table.unmap(page_number);
    }

    /// Write `bytes` to the pages represented with `page_range`.
    pub fn clone_bytes(&mut self, page_table: &mut PageTable, bytes: &[u8]) {
        let mut offset = 0;
        for state in self.page_range.iter() {
            let source = &bytes[offset..bytes.len().min(offset + PAGE_SIZE)];
            let destination = &mut page_table
                .translate(state)
                .unwrap()
                .frame_number()
                .get_bytes_mut()[..source.len()];
            destination.clone_from_slice(source);

            offset += PAGE_SIZE;
            if offset >= bytes.len() {
                break;
            }
        }
    }
}

/// The `PageSet` struct represents a collection of related [PageSegment].
pub struct PageSet {
    page_table: PageTable,
    segment_list: Vec<PageSegment>,
}

impl PageSet {
    pub fn new() -> Self {
        Self {
            page_table: PageTable::new(),
            segment_list: Vec::new(),
        }
    }

    fn push(&mut self, mut segment: PageSegment, bytes: Option<&[u8]>) {
        segment.map_range(&mut self.page_table);
        if let Some(bytes) = bytes {
            segment.clone_bytes(&mut self.page_table, bytes);
        }
        self.segment_list.push(segment);
    }

    pub fn insert_frame(
        &mut self,
        start_address: VirtualAddress,
        end_address: VirtualAddress,
        map_permission: MapPermission,
    ) {
        self.push(
            PageSegment::new(start_address, end_address, MapType::Framed, map_permission),
            None,
        );
    }
}
