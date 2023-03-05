use crate::{
    constant::{BIN_BASE_ADDRESS, BIN_SIZE_LIMIT},
    file::stack::{KERNEL_STACK, USER_STACK},
    sbi,
    trap::TrapContext,
};
use core::{arch::asm, slice};

extern "C" {
    fn _bin_count();
    fn _bin_address();
}

pub unsafe fn get_bin_count() -> usize {
    (_bin_count as *const usize).read_volatile()
}

pub fn load_bin() {
    let bin_address_pointer = _bin_address as *const usize;

    unsafe {
        asm!("fence.i");
        let bin_count = get_bin_count();
        if bin_count == 0 {
            sbi::shutdown();
        }

        for bin_index in 0..bin_count {
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
    }
}

pub fn init_bin_context(bin_index: usize) -> usize {
    let bin_text_address = BIN_BASE_ADDRESS + bin_index * BIN_SIZE_LIMIT;
    KERNEL_STACK[bin_index].push_context(TrapContext::init_context(
        bin_text_address,
        USER_STACK[bin_index].get_stack_pointer(),
    )) as usize
}
