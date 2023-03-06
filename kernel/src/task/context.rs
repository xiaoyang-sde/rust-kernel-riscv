#[repr(C)]
#[derive(Default, Copy, Clone)]
pub struct TaskContext {
    pub ra: usize,
    pub sp: usize,
    pub s: [usize; 12],
}

#[derive(PartialEq, Default, Copy, Clone)]
pub enum TaskStatus {
    #[default]
    Uninitialized,
    Idle,
    Running,
    Stopped,
}

#[derive(Default, Copy, Clone)]
pub struct TaskControlBlock {
    pub task_status: TaskStatus,
    pub task_context: TaskContext,
}
