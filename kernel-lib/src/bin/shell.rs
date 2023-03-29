#![no_std]
#![no_main]

use alloc::string::String;

use kernel_lib::{console::getchar, exec, fork, waitpid};

extern crate alloc;
#[macro_use]
extern crate kernel_lib;

const LF: u8 = 0x0au8;
const CR: u8 = 0x0du8;
const DL: u8 = 0x7fu8;
const BS: u8 = 0x08u8;

#[no_mangle]
fn main() -> i32 {
    print!("$ ");

    let mut line = String::new();
    loop {
        let char = getchar();
        match char {
            LF | CR => {
                println!("");
                if line.is_empty() {
                    continue;
                }
                line.push('\0');

                let pid = fork() as usize;
                if pid == 0 {
                    if exec(line.as_str()) == -1 {
                        return -4;
                    }
                } else {
                    let mut exit_code = 0;
                    waitpid(pid, &mut exit_code);
                }
                line.clear();
                print!("$ ");
            }
            BS | DL => {
                if line.is_empty() {
                    continue;
                }
                print!("{0} {0}", BS as char);
                line.pop();
            }
            _ => {
                print!("{}", char as char);
                line.push(char as char);
            }
        }
    }
}
