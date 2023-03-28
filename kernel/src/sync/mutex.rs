use core::{
    cell::UnsafeCell,
    hint,
    marker::PhantomData,
    ops::{Deref, DerefMut},
    sync::atomic::{AtomicBool, Ordering},
};

pub struct Mutex<T> {
    lock: AtomicBool,
    cell: UnsafeCell<T>,
    phantom: PhantomData<T>,
}

impl<T> Mutex<T> {
    pub fn new(value: T) -> Self {
        Self {
            lock: AtomicBool::new(false),
            cell: UnsafeCell::new(value),
            phantom: PhantomData,
        }
    }

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

    pub fn unlock(&self) {
        self.lock.store(false, Ordering::Release);
    }
}

unsafe impl<T> Sync for Mutex<T> {}

unsafe impl<T> Send for Mutex<T> {}

pub struct MutexGuard<'a, T> {
    mutex: &'a Mutex<T>,
}

impl<'a, T> MutexGuard<'a, T> {
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
