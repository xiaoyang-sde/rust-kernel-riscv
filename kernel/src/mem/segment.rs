use core::arch::asm;

use alloc::collections::BTreeMap;
use alloc::sync::Arc;
use alloc::vec::Vec;
use bitflags::bitflags;
use lazy_static::lazy_static;
use riscv::register::satp;
use xmas_elf::{program::Type, ElfFile};

use crate::constant::{MEM_END, PAGE_SIZE, TRAMPOLINE, TRAP_CONTEXT, USER_STACK_SIZE};
use crate::mem::{
    address::PageRange,
    frame_allocator::{FrameTracker, allocate_frame},
    page_table::PageTableEntry,
    page_table::{PTEFlags, PageTable},
    FrameNumber, PageNumber, PhysicalAddress, VirtualAddress,
};
use crate::sync::SharedRef;

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

extern "C" {
    fn text_start();
    fn text_end();
    fn rodata_start();
    fn rodata_end();
    fn data_start();
    fn data_end();
    fn bss_stack_start();
    fn bss_end();
    fn kernel_end();
    fn trampoline_start();
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

    pub fn get_end(&self) -> PageNumber {
        self.page_range.end
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

    pub fn init(&self) {
        unsafe {
            satp::write(self.get_satp());
            asm!("sfence.vma");
        }
    }

    pub fn get_satp(&self) -> usize {
        self.page_table.get_satp()
    }

    pub fn translate(&self, page_number: PageNumber) -> Option<PageTableEntry> {
        self.page_table.translate(page_number)
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

    pub fn from_kernel() -> Self {
        let mut page_set = Self::new();
        page_set.page_table.map(
            VirtualAddress::from(TRAMPOLINE).into(),
            PhysicalAddress::from(trampoline_start as usize).into(),
            PTEFlags::R | PTEFlags::X,
        );

        page_set.push(
            PageSegment::new(
                (text_start as usize).into(),
                (text_end as usize).into(),
                MapType::Identical,
                MapPermission::R | MapPermission::X,
            ),
            None,
        );

        page_set.push(
            PageSegment::new(
                (rodata_start as usize).into(),
                (rodata_end as usize).into(),
                MapType::Identical,
                MapPermission::R,
            ),
            None,
        );

        page_set.push(
            PageSegment::new(
                (data_start as usize).into(),
                (data_end as usize).into(),
                MapType::Identical,
                MapPermission::R | MapPermission::W,
            ),
            None,
        );

        page_set.push(
            PageSegment::new(
                (bss_stack_start as usize).into(),
                (bss_end as usize).into(),
                MapType::Identical,
                MapPermission::R | MapPermission::W,
            ),
            None,
        );

        page_set.push(
            PageSegment::new(
                (kernel_end as usize).into(),
                MEM_END.into(),
                MapType::Identical,
                MapPermission::R | MapPermission::W,
            ),
            None,
        );

        page_set
    }

    pub fn from_elf(elf_data: &[u8]) -> (Self, VirtualAddress, VirtualAddress) {
        let mut page_set = Self::new();
        page_set.page_table.map(
            VirtualAddress::from(TRAMPOLINE).into(),
            PhysicalAddress::from(trampoline_start as usize).into(),
            PTEFlags::R | PTEFlags::X,
        );

        let elf = ElfFile::new(elf_data).unwrap();
        assert_eq!(
            elf.header.pt1.magic,
            [0x7f, b'E', b'L', b'F'],
            "the ELF is invalid"
        );

        let mut virtual_address_limit = VirtualAddress::from(0);
        for program_header_index in 0..elf.header.pt2.ph_count() {
            let program_header = elf.program_header(program_header_index).unwrap();
            if program_header.get_type().unwrap() == Type::Load {
                let start_address = VirtualAddress::from(program_header.virtual_addr() as usize);
                let end_address = VirtualAddress::from(
                    (program_header.virtual_addr() + program_header.mem_size()) as usize,
                );

                let mut map_permission = MapPermission::U;
                if program_header.flags().is_read() {
                    map_permission |= MapPermission::R;
                }

                if program_header.flags().is_write() {
                    map_permission |= MapPermission::W;
                }

                if program_header.flags().is_execute() {
                    map_permission |= MapPermission::X;
                }

                let page_segment =
                    PageSegment::new(start_address, end_address, MapType::Framed, map_permission);
                virtual_address_limit = page_segment.get_end().into();

                page_set.push(
                    page_segment,
                    Some(
                        &elf.input[program_header.offset() as usize
                            ..(program_header.offset() + program_header.file_size()) as usize],
                    ),
                );
            }
        }

        let user_stack_start = virtual_address_limit + PAGE_SIZE;
        let user_stack_end = user_stack_start + USER_STACK_SIZE;

        page_set.push(
            PageSegment::new(
                user_stack_start,
                user_stack_end,
                MapType::Framed,
                MapPermission::R | MapPermission::W | MapPermission::U,
            ),
            None,
        );

        page_set.push(
            PageSegment::new(
                user_stack_end,
                user_stack_end,
                MapType::Framed,
                MapPermission::R | MapPermission::W | MapPermission::U,
            ),
            None,
        );

        page_set.push(
            PageSegment::new(
                TRAP_CONTEXT.into(),
                TRAMPOLINE.into(),
                MapType::Framed,
                MapPermission::R | MapPermission::W,
            ),
            None,
        );

        (
            page_set,
            user_stack_end,
            (elf.header.pt2.entry_point() as usize).into(),
        )
    }
}

lazy_static! {
    pub static ref KERNEL_SPACE: Arc<SharedRef<PageSet>> =
        Arc::new(unsafe { SharedRef::new(PageSet::from_kernel()) });
}
