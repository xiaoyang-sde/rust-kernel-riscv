//! The `fs` module provides system calls to interact with the file system.

use core::str;
use log::error;

use crate::{mem::translate_buffer, print, task::satp};

const STDOUT: usize = 1;

pub fn sys_read(fd: usize, buffer: *const u8, length: usize) -> isize {
    -1
}

/// Writes the contents of a buffer to a file descriptor.
pub fn sys_write(fd: usize, buffer: *const u8, length: usize) -> isize {
    match fd {
        STDOUT => {
            for buffer in translate_buffer(satp(), buffer, length) {
                print!("{}", str::from_utf8(buffer).unwrap());
            }
            length as isize
        }
        _ => {
            error!("the file descriptor {} is not supported in 'sys_write'", fd);
            -1
        }
    }
}
