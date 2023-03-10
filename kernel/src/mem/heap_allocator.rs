//! The `heap_allocator` module provides a heap allocator for the kernel.
//! The heap is initialized with a fixed size as the [KERNEL_HEAP_SIZE] constant.

use core::alloc::Layout;
use linked_list_allocator::LockedHeap;

use crate::constant::KERNEL_HEAP_SIZE;

#[global_allocator]
static HEAP_ALLOCATOR: LockedHeap = LockedHeap::empty();

static mut KERNEL_HEAP: [u8; KERNEL_HEAP_SIZE] = [0; KERNEL_HEAP_SIZE];

/// Initialize the kernel heap with a fixed size as the [KERNEL_HEAP_SIZE]
/// constant. This function must be called before the heap can be used.
pub fn init_heap() {
    unsafe {
        HEAP_ALLOCATOR
            .lock()
            .init(KERNEL_HEAP.as_mut_ptr(), KERNEL_HEAP_SIZE);
    }
}

/// Panic when heap allocation fails.
#[alloc_error_handler]
pub fn handle_alloc_error(layout: Layout) -> ! {
    panic!("failed to allocate the desired layout {:?}", layout);
}
