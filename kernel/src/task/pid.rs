use core::{
    mem,
    ops::{Add, AddAssign},
};

use alloc::vec::Vec;
use lazy_static::lazy_static;

use crate::{
    constant::{KERNEL_STACK_SIZE, PAGE_SIZE, TRAMPOLINE},
    mem::{MapPermission, VirtualAddress, KERNEL_SPACE},
    sync::SharedRef,
};

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

pub struct PidAllocator {
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

pub struct KernelStack {
    pid: Pid,
    kernel_stack_top: VirtualAddress,
    kernel_stack_bottom: VirtualAddress,
}

impl KernelStack {
    pub fn new(pid_handle: &PidHandle) -> Self {
        let pid = pid_handle.pid();
        let kernel_stack_top =
            VirtualAddress::from(TRAMPOLINE - usize::from(pid) * (KERNEL_STACK_SIZE + PAGE_SIZE));
        let kernel_stack_bottom = kernel_stack_top - KERNEL_STACK_SIZE;

        KERNEL_SPACE.borrow_mut().insert_frame(
            kernel_stack_bottom,
            kernel_stack_top,
            MapPermission::R | MapPermission::W,
        );
        KernelStack {
            pid,
            kernel_stack_top,
            kernel_stack_bottom,
        }
    }

    pub fn top(&self) -> VirtualAddress {
        self.kernel_stack_top
    }

    pub fn bottom(&self) -> VirtualAddress {
        self.kernel_stack_bottom
    }

    pub fn push_top<T: Sized>(&self, value: T) -> *const T {
        let pointer_mut = (usize::from(self.top()) - mem::size_of::<T>()) as *mut T;
        unsafe {
            *pointer_mut = value;
        }
        pointer_mut as *const T
    }

    pub fn push_top_mut<T: Sized>(&self, value: T) -> *mut T {
        let pointer_mut = (usize::from(self.top()) - mem::size_of::<T>()) as *mut T;
        unsafe {
            *pointer_mut = value;
        }
        pointer_mut
    }
}

impl Drop for KernelStack {
    fn drop(&mut self) {
        KERNEL_SPACE.borrow_mut().remove_segment(self.bottom());
    }
}
