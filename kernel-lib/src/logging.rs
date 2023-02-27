//! The `logging` module implements the [log::Log] trait.

use crate::println;
use log::{self, Level, LevelFilter, Metadata, Record};

struct KernelLogger;

impl log::Log for KernelLogger {
    fn enabled(&self, _: &Metadata) -> bool {
        true
    }

    fn log(&self, record: &Record) {
        if !self.enabled(record.metadata()) {
            return;
        }

        let color: u8 = match record.level() {
            Level::Error => 31,
            Level::Warn => 93,
            Level::Info => 34,
            Level::Debug => 32,
            Level::Trace => 90,
        };
        println!(
            "\u{1B}[{}m[U][{}] {}\u{1B}[0m",
            color,
            record.level(),
            record.args()
        );
    }

    fn flush(&self) {}
}

/// Initialize the kernel logger with the specified log level,
/// or defaults to LevelFilter::Info
pub fn init() {
    static LOGGER: KernelLogger = KernelLogger;
    log::set_logger(&LOGGER).unwrap();
    log::set_max_level(match option_env!("LOG_LEVEL") {
        Some("error") => LevelFilter::Error,
        Some("warn") => LevelFilter::Warn,
        Some("info") => LevelFilter::Info,
        Some("debug") => LevelFilter::Debug,
        Some("trace") => LevelFilter::Trace,
        _ => LevelFilter::Info,
    });
}
