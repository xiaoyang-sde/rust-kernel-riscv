//! The `fs` module provides system calls to interact with the file system.

use log::error;

use crate::print;
use core::{slice, str};

const STDOUT: usize = 1;

/// Write the contents of a buffer to a file descriptor.
pub fn sys_write(fd: usize, buffer: *const u8, length: usize) -> isize {
    match fd {
        STDOUT => {
            let slice = unsafe { slice::from_raw_parts(buffer, length) };
            print!("{}", str::from_utf8(slice).unwrap());
            length as isize
        }
        _ => {
            error!("the file descriptor {} is not supported in 'sys_write'", fd);
            -1
        }
    }
}
