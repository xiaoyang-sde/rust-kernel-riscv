use core::ops::{Add, AddAssign};

use alloc::vec::Vec;
use lazy_static::lazy_static;

use crate::sync::SharedRef;

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct Pid {
    bits: usize,
}

impl From<usize> for Pid {
    fn from(value: usize) -> Self {
        Pid { bits: value }
    }
}

impl From<Pid> for usize {
    fn from(value: Pid) -> Self {
        value.bits
    }
}

impl Add<usize> for Pid {
    type Output = Pid;

    fn add(self, rhs: usize) -> Self {
        Pid::from(self.bits + rhs)
    }
}

impl AddAssign<usize> for Pid {
    fn add_assign(&mut self, rhs: usize) {
        self.bits += rhs
    }
}

pub struct PidHandle {
    pid: Pid,
}

impl PidHandle {
    pub fn new(pid: Pid) -> Self {
        Self { pid }
    }

    pub fn pid(&self) -> Pid {
        self.pid
    }
}

impl Drop for PidHandle {
    fn drop(&mut self) {
        PID_ALLOCATOR.borrow_mut().deallocate(self.pid);
    }
}

struct PidAllocator {
    state: Pid,
    deallocated_pid: Vec<Pid>,
}

impl PidAllocator {
    pub fn new() -> Self {
        PidAllocator {
            state: Pid::from(0),
            deallocated_pid: Vec::new(),
        }
    }

    pub fn allocate(&mut self) -> PidHandle {
        if let Some(pid) = self.deallocated_pid.pop() {
            PidHandle::new(pid)
        } else {
            let pid_handle = PidHandle::new(self.state);
            self.state += 1;
            pid_handle
        }
    }

    pub fn deallocate(&mut self, pid: Pid) {
        self.deallocated_pid.push(pid);
    }
}

lazy_static! {
    static ref PID_ALLOCATOR: SharedRef<PidAllocator> =
        unsafe { SharedRef::new(PidAllocator::new()) };
}

pub fn allocate_pid() -> PidHandle {
    PID_ALLOCATOR.borrow_mut().allocate()
}
