use alloc::vec::Vec;

pub type Tid = usize;

pub struct TidHandle {
    tid: Tid,
}

impl TidHandle {
    pub fn new(tid: Tid) -> Self {
        Self { tid }
    }

    pub fn tid(&self) -> Tid {
        self.tid
    }
}

pub struct TidAllocator {
    state: Tid,
    deallocated_tid: Vec<Tid>,
}

impl TidAllocator {
    pub fn new() -> Self {
        TidAllocator {
            state: 0,
            deallocated_tid: Vec::new(),
        }
    }

    pub fn allocate(&mut self) -> TidHandle {
        if let Some(tid) = self.deallocated_tid.pop() {
            TidHandle::new(tid)
        } else {
            let tid_handle = TidHandle::new(self.state);
            self.state += 1;
            tid_handle
        }
    }

    pub fn deallocate(&mut self, tid: Tid) {
        self.deallocated_tid.push(tid);
    }
}
