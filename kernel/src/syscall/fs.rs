use crate::print;
use core::{slice, str};

const STDOUT: usize = 1;

pub fn sys_write(fd: usize, buffer: *const u8, length: usize) -> isize {
    match fd {
        STDOUT => {
            let slice = unsafe { slice::from_raw_parts(buffer, length) };
            print!("{}", str::from_utf8(slice).unwrap());
            length as isize
        }
        _ => {
            panic!("the fd {} is not supported in 'sys_write'", fd);
        }
    }
}
