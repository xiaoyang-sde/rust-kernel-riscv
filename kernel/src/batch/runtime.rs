const MAX_BIN_NUM: usize = 42;
const BIN_BASE_ADDRESS: usize = 0x80400000;
const BIN_SIZE_LIMIT: usize = 0x20000;

use crate::{
    batch::stack::{KERNEL_STACK, USER_STACK},
    sbi,
    sync::SharedRef,
    trap::TrapContext,
};
use core::{arch::asm, slice};
use lazy_static::lazy_static;
use log::info;

extern "C" {
    fn _bin_address_size();
    fn _bin_address();
    fn _restore(context: usize);
}

lazy_static! {
    static ref BATCH_RUNTIME: SharedRef<BatchRuntime> = unsafe {
        SharedRef::new({
            let bin_address_size_pointer = _bin_address_size as *const usize;
            let bin_address_pointer = _bin_address as *const usize;

            let bin_address_size = bin_address_size_pointer.read_volatile();
            let mut bin_address: [(usize, usize); MAX_BIN_NUM] = [(0, 0); MAX_BIN_NUM];
            for (i, bin) in bin_address.iter_mut().enumerate().take(bin_address_size) {
                *bin = (
                    bin_address_pointer.add(i * 2).read_volatile(),
                    bin_address_pointer.add(i * 2 + 1).read_volatile(),
                );
            }

            BatchRuntime {
                next_bin: 0,
                bin_address,
                bin_address_size,
            }
        })
    };
}

struct BatchRuntime {
    next_bin: usize,
    bin_address: [(usize, usize); MAX_BIN_NUM],
    bin_address_size: usize,
}

impl BatchRuntime {
    pub fn print_bin_address(&self) {
        for i in 0..self.bin_address_size {
            let (bin_address_start, bin_address_end) = self.bin_address[i];
            info!(
                "bin_{} [{:#x}, {:#x}]",
                i, bin_address_start, bin_address_end
            );
        }
    }

    pub fn load_next_bin(&mut self) {
        if self.next_bin == self.bin_address_size {
            info!("all binaries are loaded");
            sbi::shutdown();
        }

        unsafe {
            self.load_bin(self.next_bin);
        }
        self.next_bin += 1;
    }

    unsafe fn load_bin(&self, bin_id: usize) {
        // `fence.i` clears the CPU instruction cache to ensure that
        // the CPU executes the instructions in the latest `bin_image`
        asm!("fence.i");

        slice::from_raw_parts_mut(BIN_BASE_ADDRESS as *mut u8, BIN_SIZE_LIMIT).fill(0);
        let (bin_address_start, bin_address_end) = self.bin_address[bin_id];
        let bin_image = slice::from_raw_parts(
            bin_address_start as *const u8,
            bin_address_end - bin_address_start,
        );
        let bin_text_segment =
            slice::from_raw_parts_mut(BIN_BASE_ADDRESS as *mut u8, bin_image.len());
        bin_text_segment.clone_from_slice(bin_image);
    }
}

pub fn init() {
    BATCH_RUNTIME.borrow_mut().print_bin_address();
}

pub fn load_next_bin() -> ! {
    let mut batch_runtime = BATCH_RUNTIME.borrow_mut();
    batch_runtime.load_next_bin();
    drop(batch_runtime);

    unsafe {
        let context = TrapContext::init_context(BIN_BASE_ADDRESS, USER_STACK.get_stack_pointer());
        _restore(KERNEL_STACK.push_context(context) as *const _ as usize)
    }
    panic!("unreachable code in batch::load_next_bin");
}
