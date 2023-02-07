use crate::{println, sbi::shutdown};
use core::panic::PanicInfo;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    if let Some(location) = info.location() {
        println!(
            "panic at {}:{}: {}",
            location.file(),
            location.line(),
            info.message().unwrap()
        );
    } else {
        println!("panic: {}", info.message().unwrap());
    }
    shutdown();
}
