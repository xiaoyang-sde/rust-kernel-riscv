use crate::{
    file::{get_bin_count, get_bin_data},
    sbi,
    sync::SharedRef,
    task::{TaskContext, TaskControlBlock, TaskStatus},
    trap::TrapContext,
};
use alloc::vec::Vec;
use core::arch::global_asm;
use lazy_static::lazy_static;

global_asm!(include_str!("switch.asm"));

extern "C" {
    fn _switch(task_context: *mut TaskContext, next_task_context: *const TaskContext);
}

struct TaskRuntimeState {
    task_index: Option<usize>,
    task_list: Vec<TaskControlBlock>,
}

pub struct TaskRuntime {
    bin_count: usize,
    state: SharedRef<TaskRuntimeState>,
}

impl TaskRuntime {
    fn run_init_task(&self) -> ! {
        let mut state = self.state.borrow_mut();

        let init_task = &mut state.task_list[0];
        init_task.set_task_status(TaskStatus::Running);
        let next_task_context = init_task.task_context_ptr();

        state.task_index = Some(0);
        drop(state);

        let void_task_context = &mut TaskContext::default() as *mut TaskContext;
        unsafe {
            _switch(void_task_context, next_task_context);
        }
        panic!("unreachable code in TaskRuntime::run_init_task")
    }

    fn set_task_status(&self, task_status: TaskStatus) {
        let mut state = self.state.borrow_mut();
        if let Some(task_index) = state.task_index {
            state.task_list[task_index].set_task_status(task_status);
        }
    }

    fn find_idle_task(&self) -> Option<usize> {
        let state = self.state.borrow_mut();
        if let Some(task_index) = state.task_index {
            (task_index + 1..task_index + self.bin_count + 1)
                .map(|task_index| task_index % self.bin_count)
                .find(|index| state.task_list[*index].task_status() == TaskStatus::Idle)
        } else {
            None
        }
    }

    fn run_idle_task(&self) {
        if let Some(next_task_index) = self.find_idle_task() {
            let mut state = self.state.borrow_mut();
            if let Some(task_index) = state.task_index {
                let task = &mut state.task_list[task_index];
                let task_context = task.task_context_ptr_mut();

                let next_task = &mut state.task_list[next_task_index];
                next_task.set_task_status(TaskStatus::Running);
                let next_task_context = next_task.task_context_ptr();

                state.task_index = Some(next_task_index);
                drop(state);

                unsafe {
                    _switch(task_context, next_task_context);
                }
            }
        } else {
            sbi::shutdown();
        }
    }

    fn sapt(&self) -> usize {
        let state = self.state.borrow_mut();
        let task_index = state.task_index.unwrap();
        state.task_list[task_index].satp()
    }

    fn trap_context(&self) -> &mut TrapContext {
        let state = self.state.borrow_mut();
        let task_index = state.task_index.unwrap();
        state.task_list[task_index].trap_context()
    }
}

lazy_static! {
    pub static ref TASK_RUNTIME: TaskRuntime = unsafe {
        let bin_count = get_bin_count();
        let mut task_list = Vec::new();
        for i in 0..bin_count {
            task_list.push(TaskControlBlock::new(get_bin_data(i), i));
        }

        TaskRuntime {
            bin_count,
            state: SharedRef::new({
                TaskRuntimeState {
                    task_index: None,
                    task_list,
                }
            }),
        }
    };
}

pub fn run_init_task() {
    TASK_RUNTIME.run_init_task();
}

pub fn suspend_task() {
    TASK_RUNTIME.set_task_status(TaskStatus::Idle);
    TASK_RUNTIME.run_idle_task();
}

pub fn exit_task() {
    TASK_RUNTIME.set_task_status(TaskStatus::Stopped);
    TASK_RUNTIME.run_idle_task();
}

pub fn satp() -> usize {
    TASK_RUNTIME.sapt()
}

pub fn trap_context() -> &'static mut TrapContext {
    TASK_RUNTIME.trap_context()
}
