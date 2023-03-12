//! The `frame_allocator` module provides a frame allocator for the kernel.

use alloc::vec::Vec;
use lazy_static::lazy_static;

use crate::constant::MEM_LIMIT;
use crate::mem::{FrameNumber, PhysicalAddress};
use crate::sync::SharedRef;

/// The `FrameTracker` struct represents a frame in the physical memory.
/// It contains the frame number and is responsible for zeroing out the frame when it is created.
/// It deallocates the frame when it is dropped, which follows the RAII idiom.
pub struct FrameTracker {
    frame_number: FrameNumber,
}

impl FrameTracker {
    /// Initializes a frame with a specific [FrameNumber] and zeros out the frame.
    pub fn new(frame_number: FrameNumber) -> Self {
        for byte in frame_number.as_bytes_mut() {
            *byte = 0;
        }

        Self { frame_number }
    }

    pub fn frame_number(&self) -> FrameNumber {
        self.frame_number
    }
}

impl Drop for FrameTracker {
    fn drop(&mut self) {
        deallocate_frame(self.frame_number)
    }
}

trait FrameAllocator {
    fn new() -> Self;
    fn allocate(&mut self) -> Option<FrameNumber>;
    fn deallocate(&mut self, frame_number: FrameNumber);
}

pub struct StackFrameAllocator {
    frame_start: FrameNumber,
    frame_end: FrameNumber,
    deallocated_page: Vec<FrameNumber>,
}

impl StackFrameAllocator {
    fn init(&mut self, frame_start: FrameNumber, frame_end: FrameNumber) {
        self.frame_start = frame_start;
        self.frame_end = frame_end;
    }
}

impl FrameAllocator for StackFrameAllocator {
    fn new() -> Self {
        StackFrameAllocator {
            frame_start: FrameNumber::from(0),
            frame_end: FrameNumber::from(0),
            deallocated_page: Vec::new(),
        }
    }

    fn allocate(&mut self) -> Option<FrameNumber> {
        if let Some(frame_number) = self.deallocated_page.pop() {
            Some(frame_number)
        } else if self.frame_start == self.frame_end {
            None
        } else {
            let result = Some(self.frame_start);
            self.frame_start += 1;
            result
        }
    }

    fn deallocate(&mut self, frame_number: FrameNumber) {
        if self.frame_start <= frame_number || self.deallocated_page.contains(&frame_number) {
            panic!("the frame {:#x} has not been allocated", frame_number.bits)
        }
        self.deallocated_page.push(frame_number);
    }
}

lazy_static! {
    pub static ref FRAME_ALLOCATOR: SharedRef<StackFrameAllocator> =
        unsafe { SharedRef::new(StackFrameAllocator::new()) };
}

/// Initializes a frame allocator that manages the physical address from `kernel_end` to
/// [MEM_LIMIT].
pub fn init_frame() {
    extern "C" {
        fn kernel_end();
    }

    FRAME_ALLOCATOR.borrow_mut().init(
        PhysicalAddress::from(kernel_end as usize).ceil(),
        PhysicalAddress::from(MEM_LIMIT).floor(),
    );
}

/// Allocates a frame and returns a [FrameTracker] to track the allocated frame when succeeded.
pub fn allocate_frame() -> Option<FrameTracker> {
    FRAME_ALLOCATOR
        .borrow_mut()
        .allocate()
        .map(FrameTracker::new)
}

/// Deallocates the frame with a specific [FrameNumber].
pub fn deallocate_frame(frame_number: FrameNumber) {
    FRAME_ALLOCATOR.borrow_mut().deallocate(frame_number);
}
