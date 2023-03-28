use alloc::vec::Vec;

use lazy_static::lazy_static;

use crate::sync::Mutex;

pub type Pid = usize;

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
        PID_ALLOCATOR.lock().deallocate(self.pid);
    }
}

pub struct PidAllocator {
    state: Pid,
    deallocated_pid: Vec<Pid>,
}

impl PidAllocator {
    pub fn new() -> Self {
        PidAllocator {
            state: 0,
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
    static ref PID_ALLOCATOR: Mutex<PidAllocator> = Mutex::new(PidAllocator::new());
}

pub fn allocate_pid() -> PidHandle {
    PID_ALLOCATOR.lock().allocate()
}
