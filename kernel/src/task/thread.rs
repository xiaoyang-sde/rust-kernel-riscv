use alloc::sync::{Arc, Weak};

use crate::{
    constant::{PAGE_SIZE, TRAP_CONTEXT_BASE, USER_STACK_SIZE},
    executor::TrapContext,
    mem::{FrameNumber, MapPermission, PageNumber, VirtualAddress},
    sync::{Mutex, MutexGuard},
    task::{tid::Tid, Process},
};

pub struct Thread {
    tid: Tid,
    process: Weak<Process>,
    user_stack_base: VirtualAddress,

    state: Mutex<ThreadState>,
}

impl Thread {
    pub fn new(
        process: Arc<Process>,
        user_stack_base: VirtualAddress,
        allocate_resource: bool,
    ) -> Self {
        let tid = process.state().allocate_tid();

        let user_stack_bottom = user_stack_base + tid * (PAGE_SIZE + USER_STACK_SIZE);
        let user_stack_top = user_stack_bottom + USER_STACK_SIZE;
        if allocate_resource {
            process.state().page_set_mut().insert_frame(
                user_stack_bottom,
                user_stack_top,
                MapPermission::R | MapPermission::W | MapPermission::U,
            );
        }

        let trap_context_bottom = VirtualAddress::from(TRAP_CONTEXT_BASE) + tid * PAGE_SIZE;
        let trap_context_top = trap_context_bottom + PAGE_SIZE;
        if allocate_resource {
            process.state().page_set_mut().insert_frame(
                trap_context_bottom,
                trap_context_top,
                MapPermission::R | MapPermission::W,
            );
        }
        let trap_context_page = PageNumber::from(trap_context_bottom);
        let trap_context_frame = process
            .state()
            .page_set()
            .translate(trap_context_page)
            .unwrap()
            .frame_number();

        Self {
            tid,
            process: Arc::downgrade(&process),
            user_stack_base,
            state: Mutex::new(ThreadState::new(
                trap_context_page,
                trap_context_frame,
                user_stack_bottom,
            )),
        }
    }

    pub fn reallocate_resource(&self, user_stack_base: VirtualAddress) {
        let user_stack_bottom = user_stack_base + self.tid() * (PAGE_SIZE + USER_STACK_SIZE);
        let user_stack_top = user_stack_bottom + USER_STACK_SIZE;
        self.process().state().page_set_mut().insert_frame(
            user_stack_bottom,
            user_stack_top,
            MapPermission::R | MapPermission::W | MapPermission::U,
        );
        self.state().set_user_stack_bottom(user_stack_bottom);

        let trap_context_bottom = VirtualAddress::from(TRAP_CONTEXT_BASE) + self.tid() * PAGE_SIZE;
        let trap_context_top = trap_context_bottom + PAGE_SIZE;
        self.process().state().page_set_mut().insert_frame(
            trap_context_bottom,
            trap_context_top,
            MapPermission::R | MapPermission::W,
        );
        let trap_context_page = PageNumber::from(trap_context_bottom);
        let trap_context_frame = self
            .process()
            .state()
            .page_set()
            .translate(trap_context_page)
            .unwrap()
            .frame_number();
        self.state().set_trap_context_frame(trap_context_frame);
    }

    pub fn tid(&self) -> Tid {
        self.tid
    }

    pub fn user_stack_base(&self) -> VirtualAddress {
        self.user_stack_base
    }

    pub fn process(&self) -> Arc<Process> {
        self.process.upgrade().unwrap()
    }

    pub fn state(&self) -> MutexGuard<'_, ThreadState> {
        self.state.lock()
    }

    pub fn satp(&self) -> usize {
        self.process().state().page_set().satp()
    }

    pub fn exit(&self, exit_code: usize) {
        self.process().exit(exit_code);
    }

    fn deallocate_user_stack(&self) {
        let user_stack_bottom = self.user_stack_base + self.tid() * (PAGE_SIZE + USER_STACK_SIZE);
        self.process()
            .state()
            .page_set_mut()
            .remove_segment(user_stack_bottom);
    }

    fn deallocate_trap_context(&self) {
        let trap_context_bottom = VirtualAddress::from(TRAP_CONTEXT_BASE) + self.tid() * PAGE_SIZE;
        self.process()
            .state()
            .page_set_mut()
            .remove_segment(trap_context_bottom);
    }
}

impl Drop for Thread {
    fn drop(&mut self) {
        self.deallocate_user_stack();
        self.deallocate_trap_context();
        self.process().state().deallocated_tid(self.tid());
    }
}

pub struct ThreadState {
    trap_context_page: PageNumber,
    trap_context_frame: FrameNumber,
    user_stack_bottom: VirtualAddress,
}

impl ThreadState {
    pub fn new(
        trap_context_page: PageNumber,
        trap_context_frame: FrameNumber,
        user_stack_bottom: VirtualAddress,
    ) -> Self {
        Self {
            trap_context_page,
            trap_context_frame,
            user_stack_bottom,
        }
    }

    pub fn set_user_stack_bottom(&mut self, user_stack_bottom: VirtualAddress) {
        self.user_stack_bottom = user_stack_bottom;
    }

    pub fn user_stack_top(&self) -> VirtualAddress {
        self.user_stack_bottom + USER_STACK_SIZE
    }

    pub fn set_trap_context_frame(&mut self, trap_context_frame: FrameNumber) {
        self.trap_context_frame = trap_context_frame;
    }

    pub fn kernel_trap_context_mut(&self) -> &'static mut TrapContext {
        self.trap_context_frame.as_trap_context_mut()
    }

    pub fn user_trap_context_mut(&self) -> &'static mut TrapContext {
        self.trap_context_page.as_trap_context_mut()
    }
}
