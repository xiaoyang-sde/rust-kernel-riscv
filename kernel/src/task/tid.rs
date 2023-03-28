use alloc::vec::Vec;

pub type Tid = usize;

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

    pub fn allocate(&mut self) -> Tid {
        if let Some(tid) = self.deallocated_tid.pop() {
            tid
        } else {
            let tid = self.state;
            self.state += 1;
            tid
        }
    }

    pub fn deallocate(&mut self, tid: Tid) {
        self.deallocated_tid.push(tid);
    }
}
