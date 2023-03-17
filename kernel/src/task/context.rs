use crate::{
    constant::{KERNEL_STACK_SIZE, PAGE_SIZE, TRAMPOLINE, TRAP_CONTEXT},
    mem::{FrameNumber, MapPermission, PageNumber, PageSet, VirtualAddress, KERNEL_SPACE},
    trap::{trap_handler, trap_return, TrapContext},
};

#[repr(C)]
#[derive(Default)]
pub struct TaskContext {
    ra: usize,
    sp: usize,
    s: [usize; 12],
}

impl TaskContext {
    pub fn new(ra: usize, sp: usize, s: [usize; 12]) -> Self {
        Self { ra, sp, s }
    }
}

#[derive(PartialEq, Default, Copy, Clone)]
pub enum TaskStatus {
    #[default]
    Idle,
    Running,
    Stopped,
}

pub fn kernel_stack_address(bin_index: usize) -> (VirtualAddress, VirtualAddress) {
    let kernel_stack_top =
        VirtualAddress::from(TRAMPOLINE - bin_index * (KERNEL_STACK_SIZE + PAGE_SIZE));
    let kernel_stack_bottom = kernel_stack_top - KERNEL_STACK_SIZE;
    (kernel_stack_bottom, kernel_stack_top)
}

pub struct ProcessControlBlock {
    task_status: TaskStatus,
    task_context: TaskContext,
    page_set: PageSet,
    trap_context_frame: FrameNumber,
}

impl ProcessControlBlock {
    pub fn new(elf_data: &[u8], bin_index: usize) -> Self {
        let (page_set, user_stack_top, entry_point) = PageSet::from_elf(elf_data);
        let trap_context_frame = page_set
            .translate(PageNumber::from(VirtualAddress::from(TRAP_CONTEXT)))
            .unwrap()
            .frame_number();

        let (kernel_stack_bottom, kernel_stack_top) = kernel_stack_address(bin_index);
        KERNEL_SPACE.borrow_mut().insert_frame(
            kernel_stack_bottom,
            kernel_stack_top,
            MapPermission::R | MapPermission::W,
        );

        let task_control_block = Self {
            task_status: TaskStatus::Idle,
            task_context: TaskContext::new(
                trap_return as usize,
                usize::from(kernel_stack_top),
                [0; 12],
            ),
            page_set,
            trap_context_frame,
        };

        *task_control_block.trap_context() = TrapContext::init_context(
            usize::from(entry_point),
            usize::from(user_stack_top),
            trap_handler as usize,
            KERNEL_SPACE.borrow_mut().satp(),
            usize::from(kernel_stack_top),
        );
        task_control_block
    }

    pub fn trap_context(&self) -> &'static mut TrapContext {
        self.trap_context_frame.as_trap_context_mut()
    }

    pub fn satp(&self) -> usize {
        self.page_set.satp()
    }

    pub fn task_status(&self) -> TaskStatus {
        self.task_status
    }

    pub fn set_task_status(&mut self, task_status: TaskStatus) {
        self.task_status = task_status
    }

    pub fn task_context_ptr(&self) -> *const TaskContext {
        &self.task_context as *const TaskContext
    }

    pub fn task_context_ptr_mut(&mut self) -> *mut TaskContext {
        &mut self.task_context as *mut TaskContext
    }
}
