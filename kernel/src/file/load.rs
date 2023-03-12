use core::slice;

extern "C" {
    fn _bin_count();
    fn _bin_address();
}

pub unsafe fn get_bin_count() -> usize {
    (_bin_count as *const usize).read_volatile()
}

pub fn get_bin_data(bin_index: usize) -> &'static [u8] {
    let bin_address_pointer = _bin_address as *const usize;

    unsafe {
        let bin_address_start = bin_address_pointer.add(bin_index * 2).read_volatile();
        let bin_address_end = bin_address_pointer.add(bin_index * 2 + 1).read_volatile();
        slice::from_raw_parts(
            bin_address_start as *const u8,
            bin_address_end - bin_address_start,
        )
    }
}
