use core::cell::{RefCell, RefMut};

/// The `SharedRef` struct is a wrapper of the `RefCell`.
/// It implements the `Sync` trait, so its references
/// can be shared between threads.
pub struct SharedRef<T> {
    refcell: RefCell<T>,
}

unsafe impl<T> Sync for SharedRef<T> {}

impl<T> SharedRef<T> {
    pub unsafe fn new(value: T) -> Self {
        Self {
            refcell: RefCell::new(value),
        }
    }

    pub fn borrow_mut(&self) -> RefMut<'_, T> {
        self.refcell.borrow_mut()
    }
}
