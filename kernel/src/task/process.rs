use alloc::{
    collections::BTreeMap,
    sync::{Arc, Weak},
    vec::Vec,
};
use core::{cell::RefMut, sync::atomic::AtomicI32};

use lazy_static::lazy_static;

use crate::task::{
    pid::{allocate_pid, Pid},
    thread::Thread,
    tid::{Tid, TidAllocator, TidHandle},
};
use crate::{
    executor::spawn_thread, file::get_bin, mem::PageSet, sync::SharedRef, task::pid::PidHandle,
};

lazy_static! {
    static ref PROCESS_MAP: SharedRef<BTreeMap<Pid, Arc<Process>>> =
        unsafe { SharedRef::new(BTreeMap::new()) };
}

pub fn get_process(pid: Pid) -> Option<Arc<Process>> {
    PROCESS_MAP.borrow_mut().get(&pid).cloned()
}

pub fn insert_process(pid: Pid, process: Arc<Process>) {
    PROCESS_MAP.borrow_mut().insert(pid, process);
}

pub struct Process {
    pid_handle: PidHandle,

    exit_code: AtomicI32,
    state: SharedRef<ProcessState>,
}

pub struct ProcessState {
    page_set: PageSet,
    tid_allocator: TidAllocator,
    parent: Option<Weak<Process>>,
    child_list: Vec<Arc<Process>>,
    thread_list: Vec<Arc<Thread>>,
}

impl Process {
    pub fn new(bin_name: &str) -> Arc<Self> {
        let elf_data = get_bin(bin_name).unwrap();
        let (page_set, user_stack_base, entry_point) = PageSet::from_elf(elf_data);

        let pid_handle = allocate_pid();
        let process = Arc::new(Self {
            pid_handle,
            exit_code: 0.into(),
            state: unsafe { SharedRef::new(ProcessState::new(page_set, None)) },
        });

        let thread = Arc::new({
            let mut thread = Thread::new(process.clone(), user_stack_base);
            thread.allocate_trap_context();
            thread.allocate_user_stack();
            thread
        });

        let trap_context = thread.state().kernel_trap_context_mut().unwrap();
        trap_context.set_user_register(2, usize::from(thread.state().user_stack_top().unwrap()));
        trap_context.set_user_sepc(usize::from(entry_point));
        process.state().thread_list_mut().push(thread.clone());

        insert_process(process.pid(), process.clone());
        spawn_thread(thread);
        process
    }

    pub fn fork(self: &Arc<Self>) -> Arc<Self> {
        let pid_handle = allocate_pid();
        let page_set = PageSet::clone_from(self.state().page_set());
        let process = Arc::new(Self {
            pid_handle,
            exit_code: 0.into(),
            state: unsafe {
                SharedRef::new(ProcessState::new(page_set, Some(Arc::downgrade(self))))
            },
        });
        self.state().child_list_mut().push(process.clone());

        let thread = Arc::new({
            let mut thread = Thread::new(
                process.clone(),
                self.state().thread_list()[0].user_stack_base(),
            );
            thread.set_user_stack();
            thread.set_trap_context();

            let trap_context = thread.state().kernel_trap_context_mut().unwrap();
            trap_context.set_user_register(10, 0);
            thread
        });
        process.state().thread_list_mut().push(thread.clone());

        insert_process(process.pid(), process.clone());
        spawn_thread(thread);
        process
    }

    pub fn pid(&self) -> Pid {
        self.pid_handle.pid()
    }

    pub fn state(&self) -> RefMut<'_, ProcessState> {
        self.state.borrow_mut()
    }
}

impl ProcessState {
    pub fn new(page_set: PageSet, parent: Option<Weak<Process>>) -> Self {
        Self {
            page_set,
            parent,
            tid_allocator: TidAllocator::new(),
            child_list: Vec::new(),
            thread_list: Vec::new(),
        }
    }

    pub fn child_list_mut(&mut self) -> &mut Vec<Arc<Process>> {
        &mut self.child_list
    }

    pub fn thread_list(&self) -> &Vec<Arc<Thread>> {
        &self.thread_list
    }

    pub fn thread_list_mut(&mut self) -> &mut Vec<Arc<Thread>> {
        &mut self.thread_list
    }

    pub fn page_set(&self) -> &PageSet {
        &self.page_set
    }

    pub fn page_set_mut(&mut self) -> &mut PageSet {
        &mut self.page_set
    }

    pub fn allocate_tid(&mut self) -> TidHandle {
        self.tid_allocator.allocate()
    }

    pub fn deallocated_tid(&mut self, tid: Tid) {
        self.tid_allocator.deallocate(tid);
    }
}
