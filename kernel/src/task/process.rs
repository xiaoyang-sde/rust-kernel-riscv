use alloc::{
    collections::BTreeMap,
    string::String,
    sync::{Arc, Weak},
    vec::Vec,
};

use lazy_static::lazy_static;
use log::info;

use crate::{
    executor,
    file,
    mem::PageSet,
    sync::{Event, EventBus, Mutex},
    task::{
        pid::{self, Pid, PidHandle},
        thread::Thread,
        tid::{Tid, TidAllocator},
    },
};

lazy_static! {
    static ref PROCESS_MAP: Mutex<BTreeMap<Pid, Arc<Process>>> = Mutex::new(BTreeMap::new());
}

fn get_process(pid: Pid) -> Option<Arc<Process>> {
    PROCESS_MAP.lock().get(&pid).cloned()
}

fn insert_process(pid: Pid, process: Arc<Process>) {
    PROCESS_MAP.lock().insert(pid, process);
}

fn remove_process(pid: Pid) {
    PROCESS_MAP.lock().remove(&pid);
}

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum Status {
    Runnable,
    Zombie,
}

pub struct Process {
    pid_handle: PidHandle,

    state: Mutex<ProcessState>,
    event_bus: Arc<Mutex<EventBus>>,
}

pub struct ProcessState {
    status: Status,
    exit_code: usize,
    page_set: PageSet,
    tid_allocator: TidAllocator,
    parent: Option<Weak<Process>>,
    child_list: Vec<Arc<Process>>,
    thread_list: Vec<Arc<Thread>>,
}

impl Process {
    /// Creates a process with a main thread that runs a specific executable file.
    pub fn new(bin_name: &str) -> Arc<Self> {
        let elf_data = file::get_bin(bin_name).unwrap();
        let (page_set, user_stack_base, entry_point) = PageSet::from_elf(elf_data);

        let pid_handle = pid::allocate_pid();
        let process = Arc::new(Self {
            pid_handle,
            state: Mutex::new(ProcessState::new(page_set, None)),
            event_bus: EventBus::new(),
        });

        let thread = Arc::new(Thread::new(process.clone(), user_stack_base, true));
        let thread_state = thread.state().lock();
        let trap_context = thread_state.kernel_trap_context_mut();
        trap_context.set_user_register(2, usize::from(thread_state.user_stack_top()));
        trap_context.set_user_sepc(usize::from(entry_point));
        drop(thread_state);

        process
            .state()
            .lock()
            .thread_list_mut()
            .push(thread.clone());
        insert_process(process.pid(), process.clone());
        executor::spawn_thread(thread);
        process
    }

    /// Forks the current process and create a new child process.
    pub fn fork(self: &Arc<Self>) -> Arc<Self> {
        let pid_handle = pid::allocate_pid();

        let mut process_state = self.state().lock();
        let page_set = PageSet::clone_from(process_state.page_set_mut());

        let child_process = Arc::new(Self {
            pid_handle,
            state: Mutex::new(ProcessState::new(page_set, Some(Arc::downgrade(self)))),
            event_bus: EventBus::new(),
        });
        process_state.child_list_mut().push(child_process.clone());

        let user_stack_base = process_state.main_thread().user_stack_base();
        drop(process_state);

        let thread = Arc::new(Thread::new(child_process.clone(), user_stack_base, false));
        let trap_context = thread.state().lock().kernel_trap_context_mut();
        trap_context.set_user_register(10, 0);
        child_process
            .state()
            .lock()
            .thread_list_mut()
            .push(thread.clone());

        insert_process(child_process.pid(), child_process.clone());
        executor::spawn_thread(thread);
        child_process
    }

    /// Replaces the current process with a new process loaded from the executable file with a given
    /// name.
    pub fn exec(self: &Arc<Self>, bin_name: &str, _argument_list: Vec<String>) {
        let elf_data = file::get_bin(bin_name).unwrap();
        let (page_set, user_stack_base, entry_point) = PageSet::from_elf(elf_data);

        let mut process_state = self.state().lock();
        process_state.set_page_set(page_set);
        let thread = process_state.main_thread_mut().clone();
        drop(process_state);

        thread.reallocate_resource(user_stack_base);
        let thread_state = thread.state().lock();
        let trap_context = thread_state.kernel_trap_context_mut();
        trap_context.set_user_register(2, usize::from(thread_state.user_stack_top()));
        trap_context.set_user_sepc(usize::from(entry_point));
    }

    /// Terminates the current thread with the given exit code.
    pub fn exit(&self, exit_code: usize) {
        let mut process_state = self.state().lock();
        process_state.set_status(Status::Zombie);
        process_state.set_exit_code(exit_code);
        if self.pid() != 0 {
            let init_process = get_process(0).unwrap();
            let mut init_process_state = init_process.state().lock();
            for child_process in process_state.child_list_mut() {
                init_process_state
                    .child_list_mut()
                    .push(child_process.clone());
            }
        }
        process_state.thread_list_mut().clear();
        process_state.child_list_mut().clear();

        if let Some(parent) = process_state.parent() {
            if let Some(parent) = parent.upgrade() {
                parent.event_bus().lock().push(Event::CHILD_PROCESS_QUIT);
            }
        }

        remove_process(self.pid());
        info!("process {} exited with {}", self.pid(), exit_code);
    }

    pub fn pid(&self) -> Pid {
        self.pid_handle.pid()
    }

    pub fn state(&self) -> &Mutex<ProcessState> {
        &self.state
    }

    pub fn event_bus(&self) -> Arc<Mutex<EventBus>> {
        self.event_bus.clone()
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
            exit_code: 0,
            status: Status::Runnable,
        }
    }

    pub fn parent(&self) -> Option<Weak<Process>> {
        self.parent.clone()
    }

    pub fn status(&self) -> Status {
        self.status
    }

    pub fn set_status(&mut self, status: Status) {
        self.status = status;
    }

    pub fn exit_code(&self) -> usize {
        self.exit_code
    }

    pub fn set_exit_code(&mut self, exit_code: usize) {
        self.exit_code = exit_code;
    }

    pub fn child_list_mut(&mut self) -> &mut Vec<Arc<Process>> {
        &mut self.child_list
    }

    pub fn thread_list_mut(&mut self) -> &mut Vec<Arc<Thread>> {
        &mut self.thread_list
    }

    pub fn main_thread(&self) -> &Arc<Thread> {
        &self.thread_list[0]
    }

    pub fn main_thread_mut(&mut self) -> &mut Arc<Thread> {
        &mut self.thread_list[0]
    }

    pub fn page_set(&self) -> &PageSet {
        &self.page_set
    }

    pub fn page_set_mut(&mut self) -> &mut PageSet {
        &mut self.page_set
    }

    pub fn set_page_set(&mut self, page_set: PageSet) {
        self.page_set = page_set;
    }

    pub fn allocate_tid(&mut self) -> Tid {
        self.tid_allocator.allocate()
    }

    pub fn deallocated_tid(&mut self, tid: Tid) {
        self.tid_allocator.deallocate(tid);
    }
}
