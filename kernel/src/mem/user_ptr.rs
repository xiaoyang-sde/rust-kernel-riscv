use alloc::{string::String, vec::Vec};
use core::{
    marker::PhantomData,
    ops::{Deref, DerefMut},
};

use crate::{
    constant::PAGE_SIZE,
    mem::{address::PageRange, page_table::PageTable, PageNumber, VirtualAddress},
};

#[repr(C)]
#[derive(Clone, Copy)]
pub struct UserPtr<T> {
    satp: usize,
    ptr: *mut T,
    phantom: PhantomData<T>,
}

unsafe impl<T: Clone + Copy + 'static> Send for UserPtr<T> {}
unsafe impl<T: Clone + Copy + 'static> Sync for UserPtr<T> {}

impl<T> UserPtr<T> {
    pub fn new(satp: usize, ptr: usize) -> Self {
        Self {
            satp,
            ptr: ptr as *mut T,
            phantom: PhantomData,
        }
    }

    pub fn as_string(&self) -> String {
        let page_table = PageTable::from_satp(self.satp);
        let mut virtual_address = VirtualAddress::from(self.ptr as usize);
        let mut string = String::new();
        loop {
            let char_pointer = page_table.translate(virtual_address).unwrap().as_ptr() as *const u8;

            let char = unsafe { *char_pointer as char };

            if char == '\0' {
                break;
            }
            string.push(char);
            virtual_address += 1;
        }
        string
    }

    pub fn as_buffer(&self, length: usize) -> Vec<&'static [u8]> {
        let page_table = PageTable::from_satp(self.satp);
        let mut translated_buffer = Vec::new();

        let buffer_address_start = VirtualAddress::from(self.ptr as usize);
        let buffer_address_end = buffer_address_start + length;

        let page_range = PageRange::new(
            PageNumber::from(buffer_address_start),
            PageNumber::from(buffer_address_end).offset(1),
        );

        for (index, page_number) in page_range.iter().enumerate() {
            let frame_number = page_table
                .translate_page(page_number)
                .unwrap()
                .frame_number();
            let lower_bound = {
                if index == 0 {
                    buffer_address_start.page_offset()
                } else {
                    0
                }
            };

            let upper_bound = {
                if index == page_range.len() - 1 {
                    buffer_address_end.page_offset()
                } else {
                    PAGE_SIZE
                }
            };
            translated_buffer.push(&frame_number.as_bytes()[lower_bound..upper_bound]);
        }

        translated_buffer
    }
}

impl<T: 'static> Deref for UserPtr<T> {
    type Target = T;

    fn deref(&self) -> &T {
        let page_table = PageTable::from_satp(self.satp);
        let virtual_address = VirtualAddress::from(self.ptr as usize);
        page_table.translate(virtual_address).unwrap().as_ref()
    }
}

impl<T: 'static> DerefMut for UserPtr<T> {
    fn deref_mut(&mut self) -> &mut T {
        let page_table = PageTable::from_satp(self.satp);
        let virtual_address = VirtualAddress::from(self.ptr as usize);
        page_table.translate(virtual_address).unwrap().as_mut()
    }
}
