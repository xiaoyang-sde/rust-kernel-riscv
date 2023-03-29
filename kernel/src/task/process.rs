use alloc::{
    collections::BTreeMap,
    string::String,
    sync::{Arc, Weak},
    vec::Vec,
};

use lazy_static::lazy_static;
use log::info;

use crate::{
    executor::spawn_thread,
    file::get_bin,
    mem::PageSet,
    sync::{Event, EventBus, Mutex, MutexGuard},
    task::{
        pid::{allocate_pid, Pid, PidHandle},
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
    pub fn new(bin_name: &str) -> Arc<Self> {
        let elf_data = get_bin(bin_name).unwrap();
        let (page_set, user_stack_base, entry_point) = PageSet::from_elf(elf_data);

        let pid_handle = allocate_pid();
        let process = Arc::new(Self {
            pid_handle,
            state: Mutex::new(ProcessState::new(page_set, None)),
            event_bus: EventBus::new(),
        });

        let thread = Arc::new(Thread::new(process.clone(), user_stack_base, true));
        let trap_context = thread.state().kernel_trap_context_mut();
        trap_context.set_user_register(2, usize::from(thread.state().user_stack_top()));
        trap_context.set_user_sepc(usize::from(entry_point));
        process.state().thread_list_mut().push(thread.clone());

        insert_process(process.pid(), process.clone());
        spawn_thread(thread);
        process
    }

    pub fn fork(self: &Arc<Self>) -> Arc<Self> {
        let pid_handle = allocate_pid();
        let page_set = PageSet::clone_from(self.state().page_set());
        let child_process = Arc::new(Self {
            pid_handle,
            state: Mutex::new(ProcessState::new(page_set, Some(Arc::downgrade(self)))),
            event_bus: EventBus::new(),
        });
        self.state().child_list_mut().push(child_process.clone());

        let thread = Arc::new(Thread::new(
            child_process.clone(),
            self.state().main_thread().user_stack_base(),
            false,
        ));
        let trap_context = thread.state().kernel_trap_context_mut();
        trap_context.set_user_register(10, 0);
        child_process.state().thread_list_mut().push(thread.clone());

        insert_process(child_process.pid(), child_process.clone());
        spawn_thread(thread);
        child_process
    }

    pub fn exec(self: &Arc<Self>, bin_name: &str, _argument_list: Vec<String>) {
        let elf_data = get_bin(bin_name).unwrap();
        let (page_set, user_stack_base, entry_point) = PageSet::from_elf(elf_data);
        self.state().set_page_set(page_set);

        let thread = self.state().main_thread_mut().clone();
        thread.reallocate_resource(user_stack_base);

        let trap_context = thread.state().kernel_trap_context_mut();
        trap_context.set_user_register(2, usize::from(thread.state().user_stack_top()));
        trap_context.set_user_sepc(usize::from(entry_point));
    }

    pub fn exit(&self, exit_code: usize) {
        self.state().set_status(Status::Zombie);
        self.state().set_exit_code(exit_code);
        if self.pid() != 0 {
            for child_process in self.state().child_list_mut() {
                let init_process = get_process(0).unwrap();
                init_process
                    .state()
                    .child_list_mut()
                    .push(child_process.clone());
            }
        }
        self.state().thread_list_mut().clear();
        self.state().child_list_mut().clear();

        if let Some(parent) = self.state().parent() {
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

    pub fn state(&self) -> MutexGuard<'_, ProcessState> {
        self.state.lock()
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

    pub fn thread_list(&self) -> &Vec<Arc<Thread>> {
        &self.thread_list
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
