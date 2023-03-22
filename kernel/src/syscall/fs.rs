//! The `fs` module provides system calls to interact with the file system.

use core::str;
use log::error;

use crate::{executor::TaskAction, mem::translate_buffer, print};

use super::SystemCall;

const STDOUT: usize = 1;

impl SystemCall<'_> {
    pub fn sys_read(&self, _fd: usize, _buffer: *const u8, _length: usize) -> (isize, TaskAction) {
        (-1, TaskAction::Continue)
    }

    /// Writes the contents of a buffer to a file descriptor.
    pub fn sys_write(&self, fd: usize, buffer: *const u8, length: usize) -> (isize, TaskAction) {
        match fd {
            STDOUT => {
                for buffer in translate_buffer(self.thread.satp(), buffer, length) {
                    print!("{}", str::from_utf8(buffer).unwrap());
                }
                (length as isize, TaskAction::Continue)
            }
            _ => {
                error!("the file descriptor {} is not supported in 'sys_write'", fd);
                (-1, TaskAction::Continue)
            }
        }
    }
}
