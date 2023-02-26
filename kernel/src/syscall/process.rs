use log::info;

use crate::batch::runtime;

pub fn sys_exit(exit_code: i32) -> ! {
    info!("exited with {}", exit_code);
    runtime::load_next_bin();
}
