const MAX_BIN_NUM: usize = 42;
const BIN_BASE_ADDRESS: usize = 0x80400000;
const BIN_SIZE_LIMIT: usize = 0x20000;

use core::{arch::asm, slice};

use crate::{sbi, sync::shared_ref::SharedRef};
use lazy_static::lazy_static;
use log::info;

extern "C" {
    fn _bin_address_size();
    fn _bin_address();
}

lazy_static! {
    static ref BATCH_MANAGER: SharedRef<BatchManager> = unsafe {
        SharedRef::new({
            let bin_address_size_pointer = _bin_address_size as *const usize;
            let bin_address_pointer = _bin_address as *const usize;

            let bin_address_size = bin_address_size_pointer.read_volatile();
            let mut bin_address: [(usize, usize); MAX_BIN_NUM] = [(0, 0); MAX_BIN_NUM];
            for i in 0..bin_address_size {
                bin_address[i] = (
                    bin_address_pointer.add(i * 2).read_volatile(),
                    bin_address_pointer.add(i * 2 + 1).read_volatile(),
                );
            }
            BatchManager {
                bin_state: 0,
                bin_address,
                bin_address_size,
            }
        })
    };
}

struct BatchManager {
    bin_state: usize,
    bin_address: [(usize, usize); MAX_BIN_NUM],
    bin_address_size: usize,
}

impl BatchManager {
    pub fn bin_state(&self) -> usize {
        self.bin_state
    }

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
        self.bin_state += 1;
        if self.bin_state == self.bin_address_size {
            info!("all binaries are loaded");
            sbi::shutdown();
        }

        unsafe {
            self.load_bin(self.bin_state);
        }
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
    BATCH_MANAGER.borrow_mut().print_bin_address();
}
