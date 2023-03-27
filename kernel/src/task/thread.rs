use alloc::sync::{Arc, Weak};
use core::cell::RefMut;

use crate::{
    constant::{PAGE_SIZE, TRAP_CONTEXT_BASE, USER_STACK_SIZE},
    executor::TrapContext,
    mem::{FrameNumber, MapPermission, PageNumber, VirtualAddress},
    sync::SharedRef,
    task::{Process, TidHandle},
};

pub struct Thread {
    tid_handle: TidHandle,
    process: Weak<Process>,
    user_stack_base: VirtualAddress,

    state: SharedRef<ThreadState>,
}

impl Thread {
    pub fn new(
        process: Arc<Process>,
        user_stack_base: VirtualAddress,
        allocate_resource: bool,
    ) -> Self {
        let tid_handle = process.state().allocate_tid();

        let user_stack_bottom = user_stack_base + tid_handle.tid() * (PAGE_SIZE + USER_STACK_SIZE);
        let user_stack_top = user_stack_bottom + USER_STACK_SIZE;
        if allocate_resource {
            process.state().page_set_mut().insert_frame(
                user_stack_bottom,
                user_stack_top,
                MapPermission::R | MapPermission::W | MapPermission::U,
            );
        }

        let trap_context_bottom =
            VirtualAddress::from(TRAP_CONTEXT_BASE) - tid_handle.tid() * PAGE_SIZE;
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
            tid_handle,
            process: Arc::downgrade(&process),
            user_stack_base,
            state: unsafe {
                SharedRef::new(ThreadState::new(
                    trap_context_page,
                    trap_context_frame,
                    user_stack_bottom,
                ))
            },
        }
    }

    pub fn user_stack_base(&self) -> VirtualAddress {
        self.user_stack_base
    }

    pub fn process(&self) -> Arc<Process> {
        self.process.upgrade().unwrap()
    }

    pub fn state(&self) -> RefMut<'_, ThreadState> {
        self.state.borrow_mut()
    }

    pub fn satp(&self) -> usize {
        self.process().state().page_set().satp()
    }

    fn deallocate_user_stack(&mut self) {
        let user_stack_bottom =
            self.user_stack_base + self.tid_handle.tid() * (PAGE_SIZE + USER_STACK_SIZE);
        self.process()
            .state()
            .page_set_mut()
            .remove_segment(user_stack_bottom);
    }

    fn deallocate_trap_context(&mut self) {
        let trap_context_bottom =
            VirtualAddress::from(TRAP_CONTEXT_BASE) - self.tid_handle.tid() * PAGE_SIZE;
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
        self.process()
            .state()
            .deallocated_tid(self.tid_handle.tid());
    }
}

pub struct ThreadState {
    trap_context_page: PageNumber,
    trap_context_frame: FrameNumber,
    user_stack_bottom: VirtualAddress,
    exit_code: Option<i32>,
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
            exit_code: None,
        }
    }

    pub fn exit_code(&self) -> Option<i32> {
        self.exit_code
    }

    pub fn set_exit_code(&mut self, exit_code: i32) {
        self.exit_code = Some(exit_code);
    }

    pub fn user_stack_bottom(&self) -> VirtualAddress {
        self.user_stack_bottom
    }

    pub fn user_stack_top(&self) -> VirtualAddress {
        self.user_stack_bottom + USER_STACK_SIZE
    }

    pub fn kernel_trap_context_mut(&self) -> &'static mut TrapContext {
        self.trap_context_frame.as_trap_context_mut()
    }

    pub fn user_trap_context_mut(&self) -> &'static mut TrapContext {
        self.trap_context_page.as_trap_context_mut()
    }
}
