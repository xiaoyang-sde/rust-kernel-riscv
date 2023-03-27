mod address;
mod frame_allocator;
mod heap_allocator;
mod page_table;
mod segment;

pub use address::{FrameNumber, PageNumber, PhysicalAddress, VirtualAddress};
pub use frame_allocator::deallocate_frame;
pub use page_table::{translate_buffer, translate_string};
pub use segment::{MapPermission, PageSet, KERNEL_SPACE};

pub fn init() {
    heap_allocator::init();
    frame_allocator::init();
    KERNEL_SPACE.borrow_mut().init();
}
