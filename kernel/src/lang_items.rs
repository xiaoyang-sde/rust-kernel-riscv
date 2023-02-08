//! The `lang_items` module contains Rust lang items.
//! Rust lang items are functionalities that isn't hard-coded into the language,
//! but is implemented in libraries, with a special marker to tell the compiler it exists.
//! Since the kernel doesn't depend on the `std` crate, it has to implement some
//! lang items, such as the `panic_handler`.

use log::error;

use core::panic::PanicInfo;

use crate::sbi;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    if let Some(location) = info.location() {
        error!(
            " panic at {}:{}: {}",
            location.file(),
            location.line(),
            info.message().unwrap()
        );
    } else {
        error!("panic: {}", info.message().unwrap());
    }
    sbi::shutdown();
}
