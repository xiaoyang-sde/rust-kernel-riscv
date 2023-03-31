//! The `mutex` module provides a mutual exclusion primitive useful for protecting shared data.

use core::{
    cell::UnsafeCell,
    hint,
    marker::PhantomData,
    ops::{Deref, DerefMut},
    sync::atomic::{AtomicBool, Ordering},
};

/// The `Mutex` struct is a mutual exclusion primitive useful for protecting shared data, which
/// implements the [Send] and [Sync] traits.
pub struct Mutex<T> {
    lock: AtomicBool,
    cell: UnsafeCell<T>,
    phantom: PhantomData<T>,
}

impl<T> Mutex<T> {
    /// Creates a new `Mutex` with the given initial value.
    pub fn new(value: T) -> Self {
        Self {
            lock: AtomicBool::new(false),
            cell: UnsafeCell::new(value),
            phantom: PhantomData,
        }
    }

    /// Acquires a lock on the `Mutex` and returns a [MutexGuard] that provides exclusive access to
    /// the shared resource.
    pub fn lock(&self) -> MutexGuard<T> {
        while self
            .lock
            .compare_exchange(false, true, Ordering::Acquire, Ordering::Acquire)
            .is_err()
        {
            while self.lock.load(Ordering::Relaxed) {
                hint::spin_loop();
            }
        }

        MutexGuard::new(self)
    }

    /// Releases the lock on the `Mutex`.
    pub fn unlock(&self) {
        self.lock.store(false, Ordering::Release);
    }
}

unsafe impl<T> Sync for Mutex<T> {}

unsafe impl<T> Send for Mutex<T> {}

/// The `MutexGuard` struct is an RAII guard to allow scoped unlock of the lock. When the guard goes
/// out of scope, the [Mutex] it guards will be unlocked.
pub struct MutexGuard<'a, T> {
    mutex: &'a Mutex<T>,
}

impl<'a, T> MutexGuard<'a, T> {
    /// Creates a new `MutexGuard` for the given [Mutex].
    pub fn new(mutex: &'a Mutex<T>) -> Self {
        Self { mutex }
    }
}

impl<'a, T> Deref for MutexGuard<'a, T> {
    type Target = T;

    fn deref(&self) -> &T {
        unsafe { &*self.mutex.cell.get() }
    }
}

impl<'a, T> DerefMut for MutexGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { &mut *self.mutex.cell.get() }
    }
}

impl<'a, T> Drop for MutexGuard<'a, T> {
    fn drop(&mut self) {
        self.mutex.unlock();
    }
}
