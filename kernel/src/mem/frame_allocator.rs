use alloc::vec::Vec;
use lazy_static::lazy_static;

use crate::constant::MEM_END;
use crate::mem::{FrameNumber, PhysicalAddress};
use crate::sync::SharedRef;

pub struct FrameTracker {
    pub frame_number: FrameNumber,
}

impl FrameTracker {
    pub fn new(frame_number: FrameNumber) -> Self {
        for byte in frame_number.get_bytes_mut() {
            *byte = 0;
        }

        Self { frame_number }
    }
}

impl Drop for FrameTracker {
    fn drop(&mut self) {
        frame_dealloc(self.frame_number)
    }
}

trait FrameAllocator {
    fn new() -> Self;
    fn alloc(&mut self) -> Option<FrameNumber>;
    fn dealloc(&mut self, frame_number: FrameNumber);
}

pub struct StackFrameAllocator {
    physical_page_start: FrameNumber,
    physical_page_end: FrameNumber,
    deallocated_page: Vec<FrameNumber>,
}

impl StackFrameAllocator {
    fn init(&mut self, physical_page_start: FrameNumber, physical_page_end: FrameNumber) {
        self.physical_page_start = physical_page_start;
        self.physical_page_end = physical_page_end;
    }
}

impl FrameAllocator for StackFrameAllocator {
    fn new() -> Self {
        StackFrameAllocator {
            physical_page_start: 0.into(),
            physical_page_end: 0.into(),
            deallocated_page: Vec::new(),
        }
    }

    fn alloc(&mut self) -> Option<FrameNumber> {
        if let Some(frame_number) = self.deallocated_page.pop() {
            Some(frame_number)
        } else if self.physical_page_start == self.physical_page_end {
            None
        } else {
            let result = Some(self.physical_page_start);
            self.physical_page_start = self.physical_page_start.offset(1);
            result
        }
    }

    fn dealloc(&mut self, frame_number: FrameNumber) {
        if self.physical_page_start <= frame_number
            || self.deallocated_page.iter().any(|v| *v == frame_number)
        {
            panic!("the frame {:#x} has not been allocated", frame_number.bits)
        }
        self.deallocated_page.push(frame_number);
    }
}

lazy_static! {
    pub static ref FRAME_ALLOCATOR: SharedRef<StackFrameAllocator> =
        unsafe { SharedRef::new(StackFrameAllocator::new()) };
}

pub fn init_frame() {
    extern "C" {
        fn kernel_end();
    }

    FRAME_ALLOCATOR.borrow_mut().init(
        PhysicalAddress::from(kernel_end as usize).ceil(),
        PhysicalAddress::from(MEM_END).floor(),
    );
}

pub fn frame_alloc() -> Option<FrameTracker> {
    FRAME_ALLOCATOR.borrow_mut().alloc().map(FrameTracker::new)
}

pub fn frame_dealloc(frame_number: FrameNumber) {
    FRAME_ALLOCATOR.borrow_mut().dealloc(frame_number);
}
