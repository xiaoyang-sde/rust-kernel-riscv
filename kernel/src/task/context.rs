use crate::{
    constant::{KERNEL_STACK_SIZE, PAGE_SIZE, TRAMPOLINE, TRAP_CONTEXT},
    mem::{FrameNumber, MapPermission, PageSet, VirtualAddress, KERNEL_SPACE},
    trap::{trap_handler, trap_return, TrapContext},
};

#[repr(C)]
#[derive(Default)]
pub struct TaskContext {
    pub ra: usize,
    pub sp: usize,
    pub s: [usize; 12],
}

#[derive(PartialEq, Default, Copy, Clone)]
pub enum TaskStatus {
    #[default]
    Idle,
    Running,
    Stopped,
}

pub fn kernel_stack_position(bin_index: usize) -> (usize, usize) {
    let top = TRAMPOLINE - bin_index * (KERNEL_STACK_SIZE + PAGE_SIZE);
    let bottom = top - KERNEL_STACK_SIZE;
    (bottom, top)
}

pub struct TaskControlBlock {
    pub task_status: TaskStatus,
    pub task_context: TaskContext,
    pub page_set: PageSet,
    pub trap_context_frame: FrameNumber,
    pub base_size: usize,
}

impl TaskControlBlock {
    pub fn new(elf_data: &[u8], bin_index: usize) -> Self {
        let (page_set, user_stack_pointer, entry_point) = PageSet::from_elf(elf_data);
        let trap_context_frame = page_set
            .translate(VirtualAddress::from(TRAP_CONTEXT).into())
            .unwrap()
            .frame_number();

        let (kernel_stack_bottom, kernel_stack_top) = kernel_stack_position(bin_index);
        KERNEL_SPACE.borrow_mut().insert_frame(
            kernel_stack_bottom.into(),
            kernel_stack_top.into(),
            MapPermission::R | MapPermission::W,
        );

        let task_control_block = Self {
            task_status: TaskStatus::Idle,
            task_context: TaskContext {
                ra: trap_return as usize,
                sp: kernel_stack_top,
                s: [0; 12],
            },
            page_set,
            trap_context_frame,
            base_size: user_stack_pointer.into(),
        };

        let trap_context = task_control_block.get_trap_context();
        *trap_context = TrapContext::init_context(
            entry_point.into(),
            user_stack_pointer.into(),
            trap_handler as usize,
            KERNEL_SPACE.borrow_mut().get_satp(),
            kernel_stack_top,
        );
        task_control_block
    }

    pub fn get_trap_context(&self) -> &'static mut TrapContext {
        self.trap_context_frame.get_trap_context()
    }

    pub fn get_satp(&self) -> usize {
        self.page_set.get_satp()
    }
}
