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
    static ref BIN_RUNTIME: SharedRef<BinRuntime> = unsafe {
        let bin_address_size_pointer = _bin_address_size as *const usize;
        let bin_address_size = bin_address_size_pointer.read_volatile();
        SharedRef::new({
            BinRuntime {
                bin_address_size,
                bin_state: None,
            }
        })
    };
}

struct BinRuntime {
    bin_address_size: usize,
    bin_state: Option<usize>,
}

impl BinRuntime {
    pub fn execute_next_bin(&mut self) -> Result<usize, ()> {
        let next_bin = match self.bin_state {
            Some(bin_state) => bin_state + 1,
            None => 0,
        };

        if next_bin == self.bin_address_size {
            Err(())
        } else {
            self.bin_state = Some(next_bin);
            Ok(BIN_BASE_ADDRESS + next_bin * BIN_SIZE_LIMIT)
        }
    }
}

pub fn load_bin() {
    let bin_address_size_pointer = _bin_address_size as *const usize;
    let bin_address_pointer = _bin_address as *const usize;

    unsafe {
        asm!("fence.i");
        let bin_address_size = bin_address_size_pointer.read_volatile();
        for bin_index in 0..bin_address_size {
            let bin_address_start = bin_address_pointer.add(bin_index * 2).read_volatile();
            let bin_address_end = bin_address_pointer.add(bin_index * 2 + 1).read_volatile();
            let bin_image = slice::from_raw_parts(
                bin_address_start as *const u8,
                bin_address_end - bin_address_start,
            );

            let bin_text_address = BIN_BASE_ADDRESS + bin_index * BIN_SIZE_LIMIT;
            slice::from_raw_parts_mut(bin_text_address as *mut u8, BIN_SIZE_LIMIT).fill(0);

            let bin_text_segment =
                slice::from_raw_parts_mut(bin_text_address as *mut u8, bin_image.len());
            bin_text_segment.clone_from_slice(bin_image);
        }

        let mut bin_address: [(usize, usize); MAX_BIN_NUM] = [(0, 0); MAX_BIN_NUM];
        for (i, bin) in bin_address.iter_mut().enumerate().take(bin_address_size) {
            *bin = (
                bin_address_pointer.add(i * 2).read_volatile(),
                bin_address_pointer.add(i * 2 + 1).read_volatile(),
            );
        }
    }
}

pub fn execute_next_bin() -> ! {
    let mut bin_runtime = BIN_RUNTIME.borrow_mut();
    let bin_text_address = bin_runtime.execute_next_bin().unwrap_or_else(|_| {
        info!("all binaries have been executed");
        sbi::shutdown();
    });
    drop(bin_runtime);

    unsafe {
        let context = TrapContext::init_context(bin_text_address, USER_STACK.get_stack_pointer());
        _restore(KERNEL_STACK.push_context(context) as *const _ as usize)
    }
    panic!("unreachable code in batch::load_next_bin");
}
