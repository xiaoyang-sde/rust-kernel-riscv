use alloc::vec::Vec;
use core::{slice, str};
use lazy_static::lazy_static;

extern "C" {
    fn _bin_count();
    fn _bin_address();
    fn _bin_name();
}

lazy_static! {
    static ref BIN_NAME_LIST: Vec<&'static str> = {
        let mut bin_name_list = Vec::new();
        let mut bin_name_pointer = _bin_name as usize as *const u8;
        let bin_count = get_bin_count();
        unsafe {
            for _ in 0..bin_count {
                let mut bin_name_length = 0;
                while bin_name_pointer.add(bin_name_length).read_volatile() != b'\0' {
                    bin_name_length += 1;
                }
                let bin_name_slice = slice::from_raw_parts(bin_name_pointer, bin_name_length);
                bin_name_list.push(str::from_utf8(bin_name_slice).unwrap());
                bin_name_pointer = bin_name_pointer.add(bin_name_length).add(1);
            }
        }
        bin_name_list
    };
}

pub fn get_bin_count() -> usize {
    unsafe { (_bin_count as *const usize).read_volatile() }
}

pub fn get_bin(name: &str) -> Option<&'static [u8]> {
    let bin_count = get_bin_count();
    (0..bin_count)
        .find(|&bin_index| BIN_NAME_LIST[bin_index] == name)
        .map(|bin_index| get_bin_data(bin_index))
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
