//! The `heap_allocator` module provides a heap allocator for the user.
//! The heap is initialized with a fixed size as the [USER_HEAP_SIZE] constant.

use core::alloc::Layout;
use linked_list_allocator::LockedHeap;

use crate::constant::USER_HEAP_SIZE;

#[global_allocator]
static HEAP_ALLOCATOR: LockedHeap = LockedHeap::empty();

static mut USER_HEAP: [u8; USER_HEAP_SIZE] = [0; USER_HEAP_SIZE];

/// Initializes the user heap with a fixed size as the [KERNEL_HEAP_SIZE]
/// constant. This function must be called before the heap can be used.
pub fn init() {
    unsafe {
        HEAP_ALLOCATOR
            .lock()
            .init(USER_HEAP.as_mut_ptr(), USER_HEAP_SIZE);
    }
}

/// Panics when heap allocation fails.
#[alloc_error_handler]
pub fn handle_alloc_error(layout: Layout) -> ! {
    panic!("failed to allocate the desired layout {:?}", layout);
}
