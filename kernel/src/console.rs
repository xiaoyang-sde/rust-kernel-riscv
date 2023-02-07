use crate::sbi::console_putchar;
use core::fmt::{self, Write};

struct Console;

impl Write for Console {
    fn write_str(&mut self, string: &str) -> fmt::Result {
        for char in string.bytes() {
            console_putchar(char as usize);
        }
        Ok(())
    }
}

pub fn print(args: fmt::Arguments) {
    Console.write_fmt(args).unwrap();
}

#[macro_export]
macro_rules! print {
    ($fmt: literal $(, $($arg: tt)+)?) => {
        $crate::console::print(format_args!($fmt $(, $($arg)+)?));
    }
}

#[macro_export]
macro_rules! println {
    ($fmt: literal $(, $($arg: tt)+)?) => {
        $crate::console::print(format_args!(concat!($fmt, "\n") $(, $($arg)+)?));
    }
}
