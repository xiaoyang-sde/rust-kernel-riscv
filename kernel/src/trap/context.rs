use riscv::register::sstatus::{self, Sstatus, SPP};

#[repr(C)]
pub struct TrapContext {
    pub register: [usize; 32],
    pub sstatus: Sstatus,
    pub sepc: usize,
}

impl TrapContext {
    pub fn set_stack_pointer(&mut self, stack_pointer: usize) {
        self.register[2] = stack_pointer;
    }

    pub fn init_context(sepc: usize, stack_pointer: usize) -> Self {
        unsafe {
            sstatus::set_spp(SPP::User);
        }

        let mut context = Self {
            register: [0; 32],
            sstatus: sstatus::read(),
            sepc,
        };
        context.set_stack_pointer(stack_pointer);
        context
    }
}
