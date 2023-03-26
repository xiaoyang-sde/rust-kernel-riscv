use alloc::sync::{Arc, Weak};
use core::cell::RefMut;

use crate::constant::{PAGE_SIZE, TRAP_CONTEXT_BASE, USER_STACK_SIZE};
use crate::mem::{MapPermission, PageNumber};
use crate::sync::SharedRef;
use crate::task::{Process, TidHandle};
use crate::{
    executor::TrapContext,
    mem::{FrameNumber, VirtualAddress},
};

pub struct Thread {
    tid_handle: TidHandle,
    process: Weak<Process>,
    user_stack_base: VirtualAddress,

    state: SharedRef<ThreadState>,
}

impl Thread {
    pub fn new(process: Arc<Process>, user_stack_base: VirtualAddress) -> Self {
        let tid_handle = process.state().allocate_tid();

        Self {
            tid_handle,
            process: Arc::downgrade(&process),
            user_stack_base,
            state: unsafe { SharedRef::new(ThreadState::new()) },
        }
    }

    pub fn user_stack_base(&self) -> VirtualAddress {
        self.user_stack_base
    }

    pub fn process(&self) -> Arc<Process> {
        self.process.upgrade().unwrap()
    }

    pub fn set_user_stack(&mut self) {
        let user_stack_bottom =
            self.user_stack_base + self.tid_handle.tid() * (PAGE_SIZE + USER_STACK_SIZE);
        self.state().set_user_stack_bottom(user_stack_bottom);
    }

    pub fn allocate_user_stack(&mut self) {
        let user_stack_bottom =
            self.user_stack_base + self.tid_handle.tid() * (PAGE_SIZE + USER_STACK_SIZE);
        let user_stack_top = user_stack_bottom + USER_STACK_SIZE;
        self.process().state().page_set_mut().insert_frame(
            user_stack_bottom,
            user_stack_top,
            MapPermission::R | MapPermission::W | MapPermission::U,
        );
        self.state().set_user_stack_bottom(user_stack_bottom);
    }

    fn deallocate_user_stack(&mut self) {
        let user_stack_bottom =
            self.user_stack_base + self.tid_handle.tid() * (PAGE_SIZE + USER_STACK_SIZE);
        self.process()
            .state()
            .page_set_mut()
            .remove_segment(user_stack_bottom);
    }

    pub fn set_trap_context(&mut self) {
        let trap_context_bottom =
            VirtualAddress::from(TRAP_CONTEXT_BASE) - self.tid_handle.tid() * PAGE_SIZE;
        let trap_context_page = PageNumber::from(trap_context_bottom);

        let trap_context_frame = self
            .process()
            .state()
            .page_set()
            .translate(trap_context_page)
            .unwrap()
            .frame_number();
        self.state().set_trap_context_page(trap_context_page);
        self.state().set_trap_context_frame(trap_context_frame);
    }

    pub fn allocate_trap_context(&mut self) {
        let trap_context_bottom =
            VirtualAddress::from(TRAP_CONTEXT_BASE) - self.tid_handle.tid() * PAGE_SIZE;

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
        self.state().set_trap_context_page(trap_context_page);
        self.state().set_trap_context_frame(trap_context_frame);
    }

    fn deallocate_trap_context(&mut self) {
        let trap_context_bottom =
            VirtualAddress::from(TRAP_CONTEXT_BASE) - self.tid_handle.tid() * PAGE_SIZE;
        self.process()
            .state()
            .page_set_mut()
            .remove_segment(trap_context_bottom);
    }

    pub fn state(&self) -> RefMut<'_, ThreadState> {
        self.state.borrow_mut()
    }

    pub fn satp(&self) -> usize {
        self.process().state().page_set().satp()
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
    trap_context_page: Option<PageNumber>,
    trap_context_frame: Option<FrameNumber>,
    user_stack_bottom: Option<VirtualAddress>,
    exit_code: Option<i32>,
}

impl ThreadState {
    pub fn new() -> Self {
        Self {
            trap_context_page: None,
            trap_context_frame: None,
            user_stack_bottom: None,
            exit_code: None,
        }
    }

    pub fn exit_code(&self) -> Option<i32> {
        self.exit_code
    }

    pub fn set_exit_code(&mut self, exit_code: i32) {
        self.exit_code = Some(exit_code);
    }

    pub fn user_stack_bottom(&self) -> Option<VirtualAddress> {
        self.user_stack_bottom
    }

    pub fn user_stack_top(&self) -> Option<VirtualAddress> {
        self.user_stack_bottom
            .map(|user_stack_bottom| user_stack_bottom + USER_STACK_SIZE)
    }

    pub fn set_trap_context_frame(&mut self, trap_context_frame: FrameNumber) {
        self.trap_context_frame = Some(trap_context_frame);
    }

    pub fn set_user_stack_bottom(&mut self, user_stack_bottom: VirtualAddress) {
        self.user_stack_bottom = Some(user_stack_bottom);
    }

    pub fn set_trap_context_page(&mut self, trap_context_page: PageNumber) {
        self.trap_context_page = Some(trap_context_page);
    }

    pub fn kernel_trap_context_mut(&self) -> Option<&'static mut TrapContext> {
        self.trap_context_frame
            .map(|trap_context_frame| trap_context_frame.as_trap_context_mut())
    }

    pub fn user_trap_context_mut(&self) -> Option<&'static mut TrapContext> {
        self.trap_context_page
            .map(|trap_context_page| trap_context_page.as_trap_context_mut())
    }
}
