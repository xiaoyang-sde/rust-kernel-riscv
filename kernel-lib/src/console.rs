//! The `console` module contains functions that interacts with the debug console.
//! It exports useful macros such as `print!` and `println!`.

use crate::write;
use core::fmt::{self, Write};

const STDOUT: usize = 1;

struct Console;

impl Write for Console {
    fn write_str(&mut self, string: &str) -> fmt::Result {
        write(STDOUT, string.as_bytes());
        Ok(())
    }
}

pub fn print(args: fmt::Arguments) {
    Console.write_fmt(args).unwrap();
}

/// Print to the debug console.
#[macro_export]
macro_rules! print {
    ($fmt: literal $(, $($arg: tt)+)?) => {
        $crate::console::print(format_args!($fmt $(, $($arg)+)?));
    }
}

/// Print to the debug console, with a newline.
#[macro_export]
macro_rules! println {
    ($fmt: literal $(, $($arg: tt)+)?) => {
        $crate::console::print(format_args!(concat!($fmt, '\n') $(, $($arg)+)?));
    }
}
