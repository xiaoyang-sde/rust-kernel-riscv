//! The `console` module contains functions that interacts with the debug console.
//! It exports useful macros such as `print!` and `println!`.

use core::fmt::{self, Write};

use crate::{read, write};

const STDIN: usize = 0;
const STDOUT: usize = 1;

struct Console;

impl Write for Console {
    fn write_str(&mut self, string: &str) -> fmt::Result {
        write(STDOUT, string.as_bytes());
        Ok(())
    }
}

pub fn getchar() -> u8 {
    let mut char = [0; 1];
    read(STDIN, &mut char);
    char[0]
}

pub fn print(args: fmt::Arguments) {
    Console.write_fmt(args).unwrap();
}

/// Print to the debug console.
#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::console::print(format_args!($($arg)*)));
}

/// Print to the debug console, with a newline.
#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}
