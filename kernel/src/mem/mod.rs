mod address;
mod frame_allocator;
mod heap_allocator;
mod page_table;
mod segment;
mod user_ptr;

pub use address::{FrameNumber, PageNumber, PhysicalAddress, VirtualAddress};
pub use frame_allocator::deallocate_frame;
pub use segment::{MapPermission, PageSet, KERNEL_SPACE};
pub use user_ptr::UserPtr;

pub fn init() {
    heap_allocator::init();
    frame_allocator::init();
    KERNEL_SPACE.lock().init();
}
